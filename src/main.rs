#[macro_use]
extern crate lazy_static;

mod app;
mod chunk;
mod config;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use app::App;
use clap::{command, Arg};

use crate::config::Config;

struct CLIContext<'a>
{
	pub command: clap::Command<'a>
}

#[tokio::main]
async fn main()
{
	simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

	let default_panic = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |info| {
		log::error!("A panic occurred, application terminating.");
		default_panic(info);
		std::process::exit(1);
	}));

	log::trace!("Starting application...");

	let mut context = CLIContext { command: command!() };

	match init_cli(&mut context).await {
		Ok(()) => {
			let app = App::instance();
			app.run().await.unwrap();
		}
		Err(e) => {
			log::error!("{}\n", e);
			context.command.print_help().unwrap();
		}
	};
}

async fn init_cli<'a>(context: &mut CLIContext<'a>) -> Result<(), &'a str>
{
	context.command = command!()
		.arg(Arg::new("no-polling").short('n').takes_value(false).display_order(0).help(
			"(optional) If set, do not poll the serial device: \
				If the device is/becomes unavailable, terminate immediately."
		))
		.arg(
			Arg::new("serial-device-path")
				.long("serial-device")
				.short('s')
				.required(true)
				.value_name("path")
				.display_order(1)
				.help("The serial device to read from, e.g. /dev/ttyUSB0")
		)
		.arg(
			Arg::new("serial-baud-rate")
				.long("baud-rate")
				.short('b')
				.required(true)
				.value_name("number")
				.display_order(2)
				.help("The serial baud rate to use, e.g. 115200")
		)
		.arg(
			Arg::new("address")
				.long("address")
				.short('a')
				.default_value("0.0.0.0")
				.value_name("ip")
				.display_order(3)
				.help("The IP (v4 or v6) address to bind to")
		)
		.arg(
			Arg::new("port")
				.long("port")
				.short('p')
				.required(true)
				.value_name("number")
				.display_order(4)
				.help("The port to listen on")
		);

	let matches = context.command.clone().get_matches();

	let serial_device_path = matches.value_of("serial-device-path").ok_or("Missing argument: <serial-device>")?;

	let serial_baud_rate_str = matches.value_of("serial-baud-rate").ok_or("Missing argument: <baud-rate>")?;
	let serial_baud_rate: u32 = serial_baud_rate_str.parse().map_err(|_| "Invalid value for argument: <baud-rate>")?;

	let polling = !matches.is_present("no-polling");

	let address_str = matches.value_of("address").ok_or("Missing argument: <address>")?;
	let address: IpAddr = match address_str.parse::<Ipv4Addr>() {
		Ok(v4_address) => IpAddr::V4(v4_address),
		Err(_) => match address_str.parse::<Ipv6Addr>() {
			Ok(v6_address) => IpAddr::V6(v6_address),
			Err(_) => return Err("Invalid IP address for argument: <address>")
		}
	};

	let port_str = matches.value_of("port").ok_or("Missing argument: <port>")?;
	let port: u16 = port_str.parse().map_err(|_| "Invalid number for argument: <port>")?;

	// A port of 0 could mean that we're asking the OS to select a free port for us.
	// Let's be optimistic and allow it here. Worst case, 0 causes TcpListener to return an Err which will
	// make us panic.

	let mut config = Config::get_mut().await;
	config.serial_device_path = String::from(serial_device_path);
	config.serial_baud_rate = serial_baud_rate;
	config.poll_serial = polling;
	config.bind_address = address;
	config.bind_port = port;

	return Ok(());
}
