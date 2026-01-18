use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use tokio::time::{sleep, Duration};

pub struct Scheduler {
    cancelled: Arc<AtomicBool>,
}

impl Scheduler {
    /// Create a new scheduler that will call the callback after the specified seconds
    pub fn new<F>(seconds: u64, callback: F) -> Self 
    where
        F: FnOnce() + Send + 'static,
    {
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();
        
        tracing::info!("Scheduling stop in {} seconds", seconds);
        
        tokio::spawn(async move {
            // Use high-precision sleep
            let target = tokio::time::Instant::now() + Duration::from_secs(seconds);
            
            // Check cancellation every 100ms for responsive cancellation
            while tokio::time::Instant::now() < target {
                if cancelled_clone.load(Ordering::Relaxed) {
                    tracing::info!("Scheduler cancelled");
                    return;
                }
                
                let remaining = target - tokio::time::Instant::now();
                let sleep_duration = remaining.min(Duration::from_millis(100));
                sleep(sleep_duration).await;
            }
            
            if !cancelled_clone.load(Ordering::Relaxed) {
                tracing::info!("Scheduler firing callback");
                callback();
            }
        });
        
        Self { cancelled }
    }

    /// Cancel the scheduled callback
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Calculate seconds until a specific datetime in a timezone
    pub fn calculate_seconds_until(datetime_str: &str, timezone_str: &str) -> Option<u64> {
        // Parse the timezone
        let tz: Tz = timezone_str.parse().ok()?;
        
        // Parse datetime (expecting format like "2024-01-15T14:30")
        let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M").ok()?;
        
        // Convert to timezone-aware datetime
        let target_local = tz.from_local_datetime(&naive).single()?;
        let target_utc = target_local.with_timezone(&Utc);
        
        // Get current time
        let now_utc = Utc::now();
        
        // Calculate difference
        if target_utc > now_utc {
            Some((target_utc - now_utc).num_seconds() as u64)
        } else {
            // If target is in the past, return 0 (immediate stop)
            Some(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    
    #[tokio::test]
    async fn test_scheduler_fires() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let _scheduler = Scheduler::new(1, move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });
        
        sleep(Duration::from_millis(1500)).await;
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
    
    #[tokio::test]
    async fn test_scheduler_cancel() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let scheduler = Scheduler::new(2, move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });
        
        sleep(Duration::from_millis(500)).await;
        scheduler.cancel();
        sleep(Duration::from_millis(2000)).await;
        
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }
}
