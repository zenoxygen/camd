# camd

[![pipeline](https://github.com/zenoxygen/camd/actions/workflows/ci.yaml/badge.svg)](https://github.com/zenoxygen/camd/actions/workflows/ci.yaml)

Stream camera frames from a device to a virtual video device on Linux.

## About

`camd` is a command-line utility for Linux that streams camera frames from a device to a virtual video device.
Using FFmpeg, the application converts the camera frames into a suitable format for the virtual video device.

## Installation

### Requirements

- A virtual video device available
- [FFmpeg](https://github.com/FFmpeg/FFmpeg) (already installed in most of Linux distributions)

### Build

Clone this repository:

```sh
git clone https://github.com/zenoxygen/camd.git
```

Build the binary with optimizations:

```sh
cargo build --release
```

Install the binary on your system:

```sh
sudo install -m 0755 -o root -g root -t /usr/local/bin ./target/release/camd
```

## Usage

```
camd 0.1.0

USAGE:
    camd [OPTIONS]

OPTIONS:
        --host_camera <host_camera>          the host of the camera server [default: 0.0.0.0:4321]
        --host_video <host_video>            the host of the video server [default: 0.0.0.0:1234]
        --virtual_device <virtual_device>    the name of the virtual device [default: /dev/video2]
```

## Debug

Run with the environment variable set:

```sh
RUST_LOG=trace camd
```

## Setup a virtual video device on Linux

1. Install **v4l2loopback**:

```sh
sudo apt-get install v4l2loopback-dkms v4l-utils
```

2. Load the **v4l2loopback** module into the kernel:

```sh
sudo modprobe v4l2loopback
```

3. List available video devices using **v4l2-ctl**:

```sh
v4l2-ctl --list-devices
```
