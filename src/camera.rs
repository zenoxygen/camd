use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;

/// Read a camera frame from the provided stream.
async fn read_frame(camera_stream: &mut TcpStream) -> Result<Vec<u8>> {
    info!("[CAMERA SERVER] - Read frame size");
    let mut frame_size_buf = [0u8; 4];
    if camera_stream.read_exact(&mut frame_size_buf).await.is_err() {
        return Err(anyhow!("[CAMERA SERVER] - Failed to read frame size"));
    }

    // Convert the frame size to u32
    let frame_size: u32 = u32::from_be_bytes(frame_size_buf);

    info!("[CAMERA SERVER] - Read frame");
    let mut frame_buf = vec![0u8; frame_size as usize];
    if camera_stream.read_exact(&mut frame_buf).await.is_err() {
        return Err(anyhow!("[CAMERA SERVER] - Failed to read frame"));
    }

    Ok(frame_buf)
}

/// Start a server that receives camera frames from a camera stream.
/// Incoming camera frames are sent through a shared channel.
pub async fn camera_server(host_camera: &SocketAddr, chan_tx: &Sender<Vec<u8>>) -> Result<()> {
    loop {
        let camera_server = match TcpListener::bind(host_camera).await {
            Ok(server) => server,
            Err(_) => {
                error!("[CAMERA SERVER] - Failed to bind server");
                break;
            }
        };

        println!("[CAMERA SERVER] - Listen for camera on {host_camera}...");
        let (mut camera_stream, camera_addr) = match camera_server.accept().await {
            Ok(tcp) => tcp,
            Err(_) => {
                error!("[CAMERA SERVER] - Failed to accept incoming connection");
                break;
            }
        };
        println!("[CAMERA SERVER] - Camera connected: {camera_addr}");

        loop {
            let frame = match read_frame(&mut camera_stream).await {
                Ok(frame) => frame,
                Err(e) => {
                    camera_stream.shutdown().await?;
                    println!("[CAMERA SERVER] - Camera disconnected");
                    error!("{e}");
                    break;
                }
            };

            // Check if frame is compatible with the MJPEG format
            if !frame.starts_with(&[0xFF, 0xD8]) || !frame.ends_with(&[0xFF, 0xD9]) {
                error!("[CAMERA SERVER] - Invalid frame received from the camera");
                break;
            }

            // If the channel if full, block until there's room for more frames
            if chan_tx.send(frame).await.is_err() {
                camera_stream.shutdown().await?;
                error!("[CAMERA SERVER] - Failed to send frame in channel");
                break;
            }
        }
    }

    Ok(())
}
