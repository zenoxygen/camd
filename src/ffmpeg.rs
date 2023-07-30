use std::net::SocketAddr;
use std::process::{Command, Stdio};

use anyhow::Result;

/// Convert a video stream from the provided URL to a virtual device using FFmpeg.
pub async fn ffmpeg_converter(host_video: &SocketAddr, virtual_device: &String) -> Result<()> {
    loop {
        let url_video_stream = "http://".to_string() + &host_video.to_string();

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-i")
            .arg(url_video_stream)
            .arg("-vf")
            .arg("format=yuv420p")
            .arg("-f")
            .arg("v4l2")
            .arg(virtual_device)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let mut process = match cmd.spawn() {
            Ok(process) => process,
            Err(_) => {
                error!("[FFMPEG] - Failed to spawn ffmpeg command");
                continue;
            }
        };

        println!("[FFMPEG] - Stream video on virtual device {virtual_device}");

        if process.wait().is_err() {
            error!("[FFMPEG] - Failed to execute ffmpeg command");
            continue;
        }
    }
}
