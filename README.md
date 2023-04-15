<h1 align="center">serial-to-tcp</h1>

<p align="center">
<a href="https://github.com/etienne-k/serial-to-tcp">
<img alt="repository" src="https://img.shields.io/badge/repository-serial--to--tcp-8da0cb?style=for-the-badge&logo=github" height="20"/>
</a>

<a href="https://github.com/etienne-k/serial-to-tcp/actions">
<img alt="build" src="https://img.shields.io/github/actions/workflow/status/etienne-k/serial-to-tcp/rust.yml?style=for-the-badge&logo=github" height="20"/>
</a>

<a href="https://github.com/etienne-k/serial-to-tcp/blob/main/LICENSE">
<img alt="license" src="https://img.shields.io/github/license/etienne-k/serial-to-tcp?style=for-the-badge&logo=github&color=blue" height="20"/>
</a>
</p>

An app which reads data from a serial port and serves it on a TCP port.

## How to use

[Download a binary](https://github.com/etienne-k/serial-to-tcp/releases) for your architecture or build the app as outlined below.

The app takes its configuration from the command line. Multiple instances of the app can be executed at once. The app can serve multiple clients at once.

Root privileges are most likely needed to open the serial device (and to open a port under < 1024, if applicable).

### Example

Read from USB device `ttyUSB0` with a baud rate of 115200, and serve the data on TCP port 2022:

```bash
serial-to-tcp -s /dev/ttyUSB0 -b 115200 -p 2022
```

On another client in the network, assuming the server has the IP address `192.168.0.123`, test reading from the TCP port, e.g. using [socat](http://www.dest-unreach.org/socat/):

```bash
socat - TCP4:192.168.0.123:2022
```

### Usage

```
USAGE:
    serial-to-tcp [OPTIONS] --serial-device <path> --baud-rate <number> --port <number>

OPTIONS:
    -n                            (optional) If set, do not poll the serial device: If the device
                                  is/becomes unavailable, terminate immediately.
    -s, --serial-device <path>    The serial device to read from, e.g. /dev/ttyUSB0
    -b, --baud-rate <number>      The serial baud rate to use, e.g. 115200
    -a, --address <ip>            The IP (v4 or v6) address to bind to [default: 0.0.0.0]
    -p, --port <number>           The port to listen on
    -h, --help                    Print help information
    -V, --version                 Print version information
```

### Autostart

An [OpenRC script](https://github.com/etienne-k/serial-to-tcp/tree/main/etc/init.d) is available.

## Building

You'll need a [rust toolchain](https://rustup.rs), a recent stable version is recommended.

In the cloned repository:

```bash
cargo build --release
```

### Cross-building

For targetting another platform, you could manually install the required toolchain and build respectively, e.g.:

```bash
cargo build --target=arm-unknown-linux-musleabihf
```

An easier way, if you have Podman or Docker installed, may be using [cross](https://github.com/cross-rs/cross), e.g.:

```bash
cargo install cross
cross build --target=arm-unknown-linux-musleabihf
```
