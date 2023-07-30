#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod camera;
mod ffmpeg;
mod video;

use std::net::SocketAddr;
use std::path::Path;

use anyhow::{anyhow, Result};
use clap::{crate_name, crate_version, App, Arg, ArgMatches};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::camera::camera_server;
use crate::ffmpeg::ffmpeg_converter;
use crate::video::video_server;

const CHANNEL_BUFFER_SIZE: usize = 100000000;

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::with_name("virtual_device")
                .long("virtual_device")
                .help("the name of the virtual device")
                .number_of_values(1)
                .default_value("/dev/video2"),
        )
        .arg(
            Arg::with_name("host_camera")
                .long("host_camera")
                .help("the host of the camera server")
                .number_of_values(1)
                .default_value("0.0.0.0:4321"),
        )
        .arg(
            Arg::with_name("host_video")
                .long("host_video")
                .help("the host of the video server")
                .number_of_values(1)
                .default_value("0.0.0.0:1234"),
        )
        .get_matches()
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let args = parse_args();

    let virtual_device = args.value_of("virtual_device").unwrap().to_string();
    let virtual_device_path = Path::new(&virtual_device);
    if !virtual_device_path.exists() {
        return Err(anyhow!("could not find virtual device"));
    }

    let host_camera = args.value_of("host_camera").unwrap();
    let host_camera: SocketAddr = match host_camera.parse() {
        Ok(host) => host,
        Err(_) => return Err(anyhow!("could not parse camera server host")),
    };

    let host_video = args.value_of("host_video").unwrap();
    let host_video: SocketAddr = match host_video.parse() {
        Ok(host) => host,
        Err(_) => return Err(anyhow!("could not parse video server host")),
    };

    // Create a bounded channel for communicating between asynchronous tasks
    let (chan_tx, mut chan_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel(CHANNEL_BUFFER_SIZE);

    // Start camera server
    tokio::spawn(async move { camera_server(&host_camera, &chan_tx).await });

    // Start video server
    tokio::spawn(async move { video_server(&host_video, &mut chan_rx).await });

    // Start FFMPEG converter
    ffmpeg_converter(&host_video, &virtual_device).await?;

    Ok(())
}
