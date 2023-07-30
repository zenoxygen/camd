use std::net::SocketAddr;
use std::time::Instant;

use anyhow::{anyhow, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Receiver;

/// Send a video frame to the provided stream in MJPEG stream format.
async fn send_frame(video_stream: &mut TcpStream, frame: &Vec<u8>) -> Result<()> {
    let timestamp = Instant::now();

    let frame_header = format!(
        "--frame\r\n\
        Content-Type: image/jpeg\r\n\
        Content-Length: {}\r\n\
        X-Timestamp: {}\r\n\r\n",
        frame.len(),
        timestamp.elapsed().as_secs_f64()
    );

    info!("[VIDEO SERVER] - Send frame headers");
    if video_stream
        .write_all(frame_header.as_bytes())
        .await
        .is_err()
    {
        return Err(anyhow!("[VIDEO SERVER] - Failed to send frame headers"));
    }

    info!("[VIDEO SERVER] - Send frame");
    if video_stream.write_all(frame).await.is_err() {
        return Err(anyhow!("[VIDEO SERVER] - Failed to send frame"));
    }

    info!("[VIDEO SERVER] - Send frame separator");
    let frame_separator = "\r\n".to_string();
    if video_stream
        .write_all(frame_separator.as_bytes())
        .await
        .is_err()
    {
        return Err(anyhow!("[VIDEO SERVER] - Failed to send frame separator"));
    }

    info!("[VIDEO SERVER] - Flush socket");
    if video_stream.flush().await.is_err() {
        return Err(anyhow!("[VIDEO SERVER] - Failed to send flush socket"));
    }

    Ok(())
}

/// Start a server that receives video frames from a shared channel.
/// Incoming video frames are sent to the client in MJPEG stream format.
pub async fn video_server(host_video: &SocketAddr, chan_rx: &mut Receiver<Vec<u8>>) -> Result<()> {
    loop {
        let video_server = match TcpListener::bind(host_video).await {
            Ok(server) => server,
            Err(_) => {
                error!("[VIDEO SERVER] - Failed to bind server");
                break;
            }
        };

        println!("[VIDEO SERVER] - Listen for client on {host_video}...");
        let (mut video_stream, client_addr) = match video_server.accept().await {
            Ok(tcp) => tcp,
            Err(_) => {
                error!("[VIDEO SERVER] - Failed to accept incoming connection");
                break;
            }
        };
        println!("[VIDEO SERVER] - Client connected: {client_addr}");

        let headers = "HTTP/1.0 200 OK\r\n\
            Connection: close\r\n\
            Max-Age: 0\r\n\
            Expires: 0\r\n\
            Cache-Control: no-cache, private\r\n\
            Pragma: no-cache\r\n\
            Content-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n";

        info!("Initialize MJPEG stream");
        if video_stream.write_all(headers.as_bytes()).await.is_err() {
            error!("[VIDEO SERVER] - Failed to initialize MJPEG stream");
            break;
        }

        while let Some(frame) = chan_rx.recv().await {
            info!("[VIDEO SERVER] - Take frame from channel");
            if let Err(e) = send_frame(&mut video_stream, &frame).await {
                error!("{e}");
                break;
            }
        }
    }

    Ok(())
}
