use std::path::Path;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use crate::stream::types::{Stream, StreamStatus, ScheduleConfig};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(path: &Path) -> Result<Self, sqlx::Error> {
        let url = format!("sqlite:{}?mode=rwc", path.display());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;

        // Run pragma
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;
        
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        // Create table if not exists
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS streams (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                youtube_key TEXT NOT NULL,
                video_path TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'idle',
                schedule TEXT NOT NULL,
                started_at TEXT,
                stopped_at TEXT,
                created_at TEXT NOT NULL,
                last_elapsed_seconds INTEGER
            )
        "#)
        .execute(&self.pool)
        .await?;
        
        // Add last_elapsed_seconds column if not exists (migration)
        sqlx::query(r#"
            ALTER TABLE streams ADD COLUMN last_elapsed_seconds INTEGER
        "#)
        .execute(&self.pool)
        .await
        .ok(); // Ignore error if column already exists
        
        Ok(())
    }

    pub async fn get_all_streams(&self) -> Result<Vec<Stream>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, youtube_key, video_path, status, schedule, started_at, stopped_at, created_at, last_elapsed_seconds FROM streams ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let streams = rows.iter().map(|row| {
            let schedule_json: String = row.get("schedule");
            let schedule: ScheduleConfig = serde_json::from_str(&schedule_json)
                .unwrap_or_else(|_| ScheduleConfig {
                    schedule_type: crate::stream::types::ScheduleType::Manual,
                    duration: None,
                    absolute: None,
                });
            
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "live" => StreamStatus::Live,
                "scheduled" => StreamStatus::Scheduled,
                "completed" => StreamStatus::Completed,
                "error" => StreamStatus::Error,
                "stopping" => StreamStatus::Stopping,
                _ => StreamStatus::Idle,
            };

            let last_elapsed: Option<i64> = row.get("last_elapsed_seconds");

            Stream {
                id: row.get("id"),
                name: row.get("name"),
                youtube_key: row.get("youtube_key"),
                video_path: row.get("video_path"),
                status,
                schedule,
                started_at: row.get("started_at"),
                stopped_at: row.get("stopped_at"),
                created_at: row.get("created_at"),
                elapsed_seconds: None,
                last_elapsed_seconds: last_elapsed.map(|v| v as u64),
            }
        }).collect();

        Ok(streams)
    }

    pub async fn get_stream(&self, id: &str) -> Result<Option<Stream>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, youtube_key, video_path, status, schedule, started_at, stopped_at, created_at, last_elapsed_seconds FROM streams WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let schedule_json: String = row.get("schedule");
            let schedule: ScheduleConfig = serde_json::from_str(&schedule_json)
                .unwrap_or_else(|_| ScheduleConfig {
                    schedule_type: crate::stream::types::ScheduleType::Manual,
                    duration: None,
                    absolute: None,
                });
            
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "live" => StreamStatus::Live,
                "scheduled" => StreamStatus::Scheduled,
                "completed" => StreamStatus::Completed,
                "error" => StreamStatus::Error,
                "stopping" => StreamStatus::Stopping,
                _ => StreamStatus::Idle,
            };

            let last_elapsed: Option<i64> = row.get("last_elapsed_seconds");

            Stream {
                id: row.get("id"),
                name: row.get("name"),
                youtube_key: row.get("youtube_key"),
                video_path: row.get("video_path"),
                status,
                schedule,
                started_at: row.get("started_at"),
                stopped_at: row.get("stopped_at"),
                created_at: row.get("created_at"),
                elapsed_seconds: None,
                last_elapsed_seconds: last_elapsed.map(|v| v as u64),
            }
        }))
    }

    pub async fn insert_stream(&self, stream: &Stream) -> Result<(), sqlx::Error> {
        let schedule_json = serde_json::to_string(&stream.schedule)
            .unwrap_or_else(|_| "{}".to_string());
        
        let status_str = match stream.status {
            StreamStatus::Idle => "idle",
            StreamStatus::Live => "live",
            StreamStatus::Scheduled => "scheduled",
            StreamStatus::Completed => "completed",
            StreamStatus::Error => "error",
            StreamStatus::Stopping => "stopping",
        };

        sqlx::query(
            "INSERT INTO streams (id, name, youtube_key, video_path, status, schedule, started_at, stopped_at, created_at, last_elapsed_seconds) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&stream.id)
        .bind(&stream.name)
        .bind(&stream.youtube_key)
        .bind(&stream.video_path)
        .bind(status_str)
        .bind(&schedule_json)
        .bind(&stream.started_at)
        .bind(&stream.stopped_at)
        .bind(&stream.created_at)
        .bind(stream.last_elapsed_seconds.map(|v| v as i64))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_stream_status(&self, id: &str, status: StreamStatus) -> Result<(), sqlx::Error> {
        let status_str = match status {
            StreamStatus::Idle => "idle",
            StreamStatus::Live => "live",
            StreamStatus::Scheduled => "scheduled",
            StreamStatus::Completed => "completed",
            StreamStatus::Error => "error",
            StreamStatus::Stopping => "stopping",
        };

        sqlx::query("UPDATE streams SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_stream_started_at(&self, id: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE streams SET started_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_stream_stopped_at(&self, id: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE streams SET stopped_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_stream_last_elapsed(&self, id: &str, seconds: u64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE streams SET last_elapsed_seconds = ? WHERE id = ?")
            .bind(seconds as i64)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_stream(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM streams WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
