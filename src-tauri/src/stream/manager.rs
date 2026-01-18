use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

use crate::db::Database;
use crate::stream::process::FFmpegProcess;
use crate::stream::scheduler::Scheduler;
use crate::stream::types::{Stream, StreamInput, StreamStatus};

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Stream not found: {0}")]
    NotFound(String),
    #[error("Stream already running: {0}")]
    AlreadyRunning(String),
    #[error("Duplicate stream key: {0} - Key đã được sử dụng bởi luồng đang phát")]
    DuplicateKey(String),
    #[error("FFmpeg error: {0}")]
    FFmpeg(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct StreamManager {
    db: Option<Database>,
    processes: Arc<RwLock<HashMap<String, FFmpegProcess>>>,
    schedulers: Arc<RwLock<HashMap<String, Scheduler>>>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            db: None,
            processes: Arc::new(RwLock::new(HashMap::new())),
            schedulers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize(&mut self, instance_id: &str) -> Result<(), ManagerError> {
        let db_path = Self::get_db_path(instance_id);
        tracing::info!("Initializing database at: {:?}", db_path);
        
        let db = Database::new(&db_path).await?;
        db.migrate().await?;
        self.db = Some(db);
        
        // Start process monitor
        self.start_process_monitor();
        
        Ok(())
    }

    fn get_db_path(instance_id: &str) -> PathBuf {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("youtube-live-manager");
        std::fs::create_dir_all(&data_dir).ok();
        data_dir.join(format!("streams_{}.db", &instance_id[..8]))
    }

    fn db(&self) -> Result<&Database, ManagerError> {
        self.db.as_ref().ok_or_else(|| {
            ManagerError::Database(sqlx::Error::Configuration("Database not initialized".into()))
        })
    }

    /// Monitor FFmpeg processes for unexpected exits (YouTube errors)
    fn start_process_monitor(&self) {
        let processes = self.processes.clone();
        let db = self.db.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                
                let mut dead_streams: Vec<(String, u64)> = Vec::new();
                
                // Check for dead processes
                {
                    let mut procs = processes.write().await;
                    let ids: Vec<String> = procs.keys().cloned().collect();
                    
                    for id in ids {
                        if let Some(process) = procs.get_mut(&id) {
                            if !process.is_running() {
                                let elapsed = process.elapsed_seconds();
                                dead_streams.push((id.clone(), elapsed));
                                procs.remove(&id);
                            }
                        }
                    }
                }
                
                // Update status for dead streams
                if let Some(db) = &db {
                    for (id, elapsed) in dead_streams {
                        tracing::warn!("Stream {} died unexpectedly after {}s", id, elapsed);
                        
                        // Mark as error with elapsed time
                        if let Err(e) = db.update_stream_status(&id, StreamStatus::Error).await {
                            tracing::error!("Error updating stream status: {}", e);
                        }
                        if let Err(e) = db.update_stream_stopped_at(&id).await {
                            tracing::error!("Error updating stopped_at: {}", e);
                        }
                        if let Err(e) = db.update_stream_last_elapsed(&id, elapsed).await {
                            tracing::error!("Error updating last_elapsed: {}", e);
                        }
                    }
                }
            }
        });
    }

    pub async fn get_streams(&self) -> Result<Vec<Stream>, ManagerError> {
        let mut streams = self.db()?.get_all_streams().await?;
        let processes = self.processes.read().await;
        
        // Update elapsed time for running streams or show last elapsed for stopped ones
        for stream in &mut streams {
            if let Some(process) = processes.get(&stream.id) {
                // Running stream - show live elapsed time
                stream.elapsed_seconds = Some(process.elapsed_seconds());
            } else if stream.last_elapsed_seconds.is_some() {
                // Stopped stream with recorded elapsed - show it
                stream.elapsed_seconds = stream.last_elapsed_seconds;
            }
        }
        
        Ok(streams)
    }

    pub async fn add_stream(&mut self, input: StreamInput) -> Result<Stream, ManagerError> {
        // Check for duplicate YouTube key on live streams
        let existing_streams = self.db()?.get_all_streams().await?;
        for existing in &existing_streams {
            if existing.youtube_key == input.youtube_key && existing.status == StreamStatus::Live {
                return Err(ManagerError::DuplicateKey(input.youtube_key.clone()));
            }
        }
        
        let start_immediately = input.start_immediately;
        
        let stream = Stream {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            youtube_key: input.youtube_key,
            video_path: input.video_path,
            status: StreamStatus::Idle,
            schedule: input.schedule,
            started_at: None,
            stopped_at: None,
            created_at: input.created_at,
            elapsed_seconds: None,
            last_elapsed_seconds: None,
        };
        
        self.db()?.insert_stream(&stream).await?;
        
        // Auto-start if requested
        if start_immediately {
            let id = stream.id.clone();
            if let Err(e) = self.start_stream(&id).await {
                tracing::error!("Failed to auto-start stream: {}", e);
            }
        }
        
        // Return fresh stream data
        let updated = self.db()?.get_stream(&stream.id).await?
            .unwrap_or(stream);
        
        Ok(updated)
    }

    pub async fn start_stream(&mut self, id: &str) -> Result<(), ManagerError> {
        let stream = self.db()?.get_stream(id).await?
            .ok_or_else(|| ManagerError::NotFound(id.to_string()))?;

        if stream.status == StreamStatus::Live {
            return Err(ManagerError::AlreadyRunning(id.to_string()));
        }

        // Check for duplicate YouTube key on other live streams
        {
            let processes = self.processes.read().await;
            let all_streams = self.db()?.get_all_streams().await?;
            for other in &all_streams {
                if other.id != id 
                    && other.youtube_key == stream.youtube_key 
                    && processes.contains_key(&other.id) 
                {
                    return Err(ManagerError::DuplicateKey(stream.youtube_key.clone()));
                }
            }
        }

        // Get FFmpeg path
        let ffmpeg_path = Self::get_ffmpeg_path();
        
        // Start FFmpeg process
        let process = FFmpegProcess::start(
            &ffmpeg_path,
            &stream.video_path,
            &stream.youtube_key,
        ).await.map_err(|e| ManagerError::FFmpeg(e.to_string()))?;

        // Store process temporarily
        let process_id = id.to_string();
        {
            let mut processes = self.processes.write().await;
            processes.insert(process_id.clone(), process);
        }

        // Wait a moment and verify FFmpeg is still running
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Check if process is still alive
        let is_running = {
            let mut processes = self.processes.write().await;
            if let Some(proc) = processes.get_mut(&process_id) {
                proc.is_running()
            } else {
                false
            }
        };

        if !is_running {
            // Process died - remove it and report error
            {
                let mut processes = self.processes.write().await;
                processes.remove(&process_id);
            }
            self.db()?.update_stream_status(id, StreamStatus::Error).await?;
            return Err(ManagerError::FFmpeg("FFmpeg process exited immediately - check video file or stream key".into()));
        }

        // Process is running - update stream status to Live
        self.db()?.update_stream_status(id, StreamStatus::Live).await?;
        self.db()?.update_stream_started_at(id).await?;

        // Setup scheduler if needed
        self.setup_scheduler(id, &stream).await?;

        Ok(())
    }

    pub async fn stop_stream(&mut self, id: &str) -> Result<(), ManagerError> {
        self.stop_stream_with_status(id, StreamStatus::Completed).await
    }

    async fn stop_stream_with_status(&mut self, id: &str, final_status: StreamStatus) -> Result<(), ManagerError> {
        // Get elapsed before stopping
        let elapsed = {
            let processes = self.processes.read().await;
            processes.get(id).map(|p| p.elapsed_seconds())
        };

        // Cancel scheduler first
        {
            let mut schedulers = self.schedulers.write().await;
            if let Some(scheduler) = schedulers.remove(id) {
                scheduler.cancel();
            }
        }

        // Update status to stopping
        self.db()?.update_stream_status(id, StreamStatus::Stopping).await?;

        // Stop FFmpeg process
        {
            let mut processes = self.processes.write().await;
            if let Some(mut process) = processes.remove(id) {
                process.stop().await.map_err(|e| ManagerError::FFmpeg(e.to_string()))?;
            }
        }

        // Update stream status and store elapsed
        self.db()?.update_stream_status(id, final_status).await?;
        self.db()?.update_stream_stopped_at(id).await?;
        
        if let Some(secs) = elapsed {
            self.db()?.update_stream_last_elapsed(id, secs).await?;
        }

        Ok(())
    }

    pub async fn delete_stream(&mut self, id: &str) -> Result<(), ManagerError> {
        // Make sure stream is stopped first
        let stream = self.db()?.get_stream(id).await?;
        if let Some(s) = stream {
            if s.status == StreamStatus::Live {
                self.stop_stream(id).await?;
            }
        }
        
        self.db()?.delete_stream(id).await?;
        Ok(())
    }

    async fn setup_scheduler(&self, id: &str, stream: &Stream) -> Result<(), ManagerError> {
        use crate::stream::types::ScheduleType;

        let stop_after_seconds = match &stream.schedule.schedule_type {
            ScheduleType::Duration => {
                stream.schedule.duration.as_ref().map(|d| d.to_seconds())
            }
            ScheduleType::Absolute => {
                stream.schedule.absolute.as_ref().and_then(|abs| {
                    Scheduler::calculate_seconds_until(&abs.datetime, &abs.timezone)
                })
            }
            ScheduleType::Manual => None,
        };

        if let Some(seconds) = stop_after_seconds {
            let id_for_scheduler = id.to_string();
            let id_for_insert = id.to_string();
            let processes = self.processes.clone();
            let db = self.db.clone();
            let schedulers = self.schedulers.clone();

            let scheduler = Scheduler::new(seconds, move || {
                let id = id_for_scheduler.clone();
                let processes = processes.clone();
                let db = db.clone();
                let schedulers = schedulers.clone();
                
                tokio::spawn(async move {
                    tracing::info!("Scheduled stop triggered for stream: {}", id);
                    
                    // Get elapsed before stopping
                    let elapsed = {
                        let procs = processes.read().await;
                        procs.get(&id).map(|p| p.elapsed_seconds())
                    };
                    
                    // Remove scheduler
                    {
                        let mut scheds = schedulers.write().await;
                        scheds.remove(&id);
                    }
                    
                    // Stop process
                    {
                        let mut procs = processes.write().await;
                        if let Some(mut process) = procs.remove(&id) {
                            if let Err(e) = process.stop().await {
                                tracing::error!("Error stopping process: {}", e);
                            }
                        }
                    }
                    
                    // Update DB - mark as Completed (scheduled stop)
                    if let Some(db) = db {
                        if let Err(e) = db.update_stream_status(&id, StreamStatus::Completed).await {
                            tracing::error!("Error updating stream status: {}", e);
                        }
                        if let Err(e) = db.update_stream_stopped_at(&id).await {
                            tracing::error!("Error updating stopped_at: {}", e);
                        }
                        if let Some(secs) = elapsed {
                            if let Err(e) = db.update_stream_last_elapsed(&id, secs).await {
                                tracing::error!("Error updating last_elapsed: {}", e);
                            }
                        }
                    }
                });
            });

            let mut schedulers = self.schedulers.write().await;
            schedulers.insert(id_for_insert, scheduler);
        }

        Ok(())
    }

    fn get_ffmpeg_path() -> PathBuf {
        // Check bundled binary first
        if let Ok(exe_path) = std::env::current_exe() {
            let resource_dir = exe_path.parent().unwrap_or(&exe_path);
            
            #[cfg(windows)]
            let bundled = resource_dir.join("binaries").join("ffmpeg.exe");
            
            #[cfg(not(windows))]
            let bundled = resource_dir.join("binaries").join("ffmpeg");
            
            if bundled.exists() {
                return bundled;
            }
            
            // Also check Resources folder on macOS
            #[cfg(target_os = "macos")]
            {
                let macos_resources = resource_dir
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("Resources").join("binaries").join("ffmpeg"));
                if let Some(path) = macos_resources {
                    if path.exists() {
                        return path;
                    }
                }
            }
        }

        // Fallback to system ffmpeg
        #[cfg(windows)]
        return PathBuf::from("ffmpeg.exe");
        
        #[cfg(not(windows))]
        PathBuf::from("ffmpeg")
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}
