use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StreamStatus {
    Idle,       // Draft - not started
    Live,       // Currently streaming
    Scheduled,  // Scheduled to start later
    Completed,  // Finished successfully (user stop or timer)
    Error,      // Failed (YouTube error, network, etc.)
    Stopping,   // In process of stopping
}

impl Default for StreamStatus {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleType {
    Manual,
    Duration,
    Absolute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationConfig {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl DurationConfig {
    pub fn to_seconds(&self) -> u64 {
        (self.hours as u64 * 3600) + (self.minutes as u64 * 60) + self.seconds as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsoluteConfig {
    pub datetime: String, // ISO format
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    #[serde(rename = "type")]
    pub schedule_type: ScheduleType,
    pub duration: Option<DurationConfig>,
    pub absolute: Option<AbsoluteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub id: String,
    pub name: String,
    pub youtube_key: String,
    pub video_path: String,
    pub status: StreamStatus,
    pub schedule: ScheduleConfig,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub elapsed_seconds: Option<u64>,
    #[serde(default)]
    pub last_elapsed_seconds: Option<u64>, // Store elapsed when stopped/errored
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInput {
    pub name: String,
    pub youtube_key: String,
    pub video_path: String,
    pub schedule: ScheduleConfig,
    pub created_at: String,
    #[serde(default)]
    pub start_immediately: bool, // New field: start after save
}
