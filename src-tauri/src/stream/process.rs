use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::{Child, Command};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Failed to spawn FFmpeg: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("FFmpeg exited with error: {0}")]
    Exit(String),
    #[error("Video file not found: {0}")]
    VideoNotFound(String),
}

pub struct FFmpegProcess {
    child: Child,
    started_at: Instant,
}

impl FFmpegProcess {
    pub async fn start(
        ffmpeg_path: &Path,
        video_path: &str,
        stream_key: &str,
    ) -> Result<Self, ProcessError> {
        // Validate video file exists
        if !Path::new(video_path).exists() {
            return Err(ProcessError::VideoNotFound(video_path.to_string()));
        }

        let rtmp_url = format!("rtmp://a.rtmp.youtube.com/live2/{}", stream_key);
        
        tracing::info!("Starting FFmpeg stream: {} -> YouTube", video_path);

        // Try hardware encoding first, fallback to software
        let child = Self::try_hardware_encoding(ffmpeg_path, video_path, &rtmp_url).await
            .or_else(|_| Self::start_software(ffmpeg_path, video_path, &rtmp_url))?;

        Ok(Self {
            child,
            started_at: Instant::now(),
        })
    }

    #[cfg(target_os = "windows")]
    async fn try_hardware_encoding(
        ffmpeg_path: &Path,
        video_path: &str,
        rtmp_url: &str,
    ) -> Result<Child, ProcessError> {
        tracing::info!("Trying NVIDIA NVENC hardware encoding...");
        
        // Windows: Try NVENC (NVIDIA GPU) first
        let result = Command::new(ffmpeg_path)
            .arg("-re")
            .arg("-stream_loop").arg("-1")
            .arg("-i").arg(video_path)
            
            // NVIDIA NVENC encoder
            .arg("-c:v").arg("h264_nvenc")
            .arg("-preset").arg("p4")         // Balanced preset for NVENC
            .arg("-tune").arg("ll")           // Low latency tuning
            .arg("-rc").arg("cbr")            // Constant bitrate mode
            
            .arg("-r").arg("30")
            .arg("-g").arg("60")              // GOP = 2 seconds
            .arg("-bf").arg("0")              // No B-frames for low latency
            
            .arg("-b:v").arg("4500k")
            .arg("-maxrate").arg("4500k")
            .arg("-bufsize").arg("9000k")
            
            .arg("-profile:v").arg("high")
            .arg("-pix_fmt").arg("yuv420p")
            
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg("128k")
            .arg("-ar").arg("44100")
            .arg("-ac").arg("2")
            
            .arg("-f").arg("flv")
            .arg("-flvflags").arg("no_duration_filesize")
            .arg(rtmp_url)
            
            .arg("-loglevel").arg("warning")
            .arg("-stats")
            
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            
            .spawn();

        match result {
            Ok(child) => {
                tracing::info!("Using NVIDIA NVENC hardware encoder");
                Ok(child)
            }
            Err(_) => {
                tracing::warn!("NVENC not available, trying Intel QuickSync...");
                Self::try_qsv_encoding(ffmpeg_path, video_path, rtmp_url).await
            }
        }
    }

    #[cfg(target_os = "windows")]
    async fn try_qsv_encoding(
        ffmpeg_path: &Path,
        video_path: &str,
        rtmp_url: &str,
    ) -> Result<Child, ProcessError> {
        // Windows: Try Intel QuickSync
        Command::new(ffmpeg_path)
            .arg("-re")
            .arg("-stream_loop").arg("-1")
            .arg("-i").arg(video_path)
            
            // Intel QuickSync encoder
            .arg("-c:v").arg("h264_qsv")
            .arg("-preset").arg("faster")
            
            .arg("-r").arg("30")
            .arg("-g").arg("60")
            
            .arg("-b:v").arg("4500k")
            .arg("-maxrate").arg("4500k")
            .arg("-bufsize").arg("9000k")
            
            .arg("-profile:v").arg("high")
            .arg("-pix_fmt").arg("yuv420p")
            
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg("128k")
            .arg("-ar").arg("44100")
            .arg("-ac").arg("2")
            
            .arg("-f").arg("flv")
            .arg("-flvflags").arg("no_duration_filesize")
            .arg(rtmp_url)
            
            .arg("-loglevel").arg("warning")
            .arg("-stats")
            
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            
            .spawn()
            .map_err(ProcessError::from)
    }

    #[cfg(target_os = "macos")]
    async fn try_hardware_encoding(
        ffmpeg_path: &Path,
        video_path: &str,
        rtmp_url: &str,
    ) -> Result<Child, ProcessError> {
        tracing::info!("Trying VideoToolbox hardware encoding...");
        
        // macOS: Use VideoToolbox
        Command::new(ffmpeg_path)
            .arg("-re")
            .arg("-stream_loop").arg("-1")
            .arg("-i").arg(video_path)
            
            .arg("-c:v").arg("h264_videotoolbox")
            
            .arg("-r").arg("30")
            .arg("-g").arg("60")
            
            .arg("-b:v").arg("4500k")
            .arg("-maxrate").arg("4500k")
            .arg("-bufsize").arg("9000k")
            
            .arg("-profile:v").arg("high")
            .arg("-pix_fmt").arg("yuv420p")
            
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg("128k")
            .arg("-ar").arg("44100")
            .arg("-ac").arg("2")
            
            .arg("-f").arg("flv")
            .arg("-flvflags").arg("no_duration_filesize")
            .arg(rtmp_url)
            
            .arg("-loglevel").arg("warning")
            .arg("-stats")
            
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            
            .spawn()
            .map_err(ProcessError::from)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    async fn try_hardware_encoding(
        _ffmpeg_path: &Path,
        _video_path: &str,
        _rtmp_url: &str,
    ) -> Result<Child, ProcessError> {
        // Linux: Skip to software encoding
        Err(ProcessError::Exit("No hardware encoder on Linux".into()))
    }

    fn start_software(
        ffmpeg_path: &Path,
        video_path: &str,
        rtmp_url: &str,
    ) -> Result<Child, ProcessError> {
        tracing::info!("Using software encoding (libx264)...");
        
        Command::new(ffmpeg_path)
            .arg("-re")
            .arg("-stream_loop").arg("-1")
            .arg("-i").arg(video_path)
            
            // Software encoding - optimized for speed
            .arg("-c:v").arg("libx264")
            .arg("-preset").arg("ultrafast")  // Fastest encoding
            .arg("-tune").arg("zerolatency")  // Low latency
            
            .arg("-r").arg("30")
            .arg("-g").arg("60")
            .arg("-keyint_min").arg("60")
            .arg("-sc_threshold").arg("0")
            
            .arg("-b:v").arg("3000k")         // Lower bitrate for CPU
            .arg("-maxrate").arg("3000k")
            .arg("-bufsize").arg("6000k")
            
            .arg("-profile:v").arg("main")
            .arg("-pix_fmt").arg("yuv420p")
            
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg("128k")
            .arg("-ar").arg("44100")
            .arg("-ac").arg("2")
            
            .arg("-f").arg("flv")
            .arg("-flvflags").arg("no_duration_filesize")
            .arg(rtmp_url)
            
            .arg("-loglevel").arg("warning")
            .arg("-stats")
            
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            
            .spawn()
            .map_err(ProcessError::from)
    }

    pub fn elapsed_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    pub async fn stop(&mut self) -> Result<(), ProcessError> {
        tracing::info!("Stopping FFmpeg process...");
        
        // Get process ID for logging
        let pid = self.child.id();
        tracing::info!("FFmpeg PID: {:?}", pid);
        
        // First, try to kill the process
        if let Err(e) = self.child.kill().await {
            tracing::warn!("First kill attempt failed: {}", e);
        }
        
        // Wait for process to exit with timeout
        let wait_result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            self.child.wait()
        ).await;
        
        match wait_result {
            Ok(Ok(status)) => {
                tracing::info!("FFmpeg exited with status: {}", status);
            }
            Ok(Err(e)) => {
                tracing::warn!("Wait error: {}", e);
            }
            Err(_) => {
                // Timeout - force kill again
                tracing::warn!("Timeout waiting for FFmpeg, force killing...");
                let _ = self.child.kill().await;
                let _ = self.child.wait().await;
            }
        }
        
        tracing::info!("FFmpeg process stopped");
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }
}

impl Drop for FFmpegProcess {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            tracing::info!("Killing FFmpeg on drop");
            let _ = self.child.start_kill();
        }
    }
}
