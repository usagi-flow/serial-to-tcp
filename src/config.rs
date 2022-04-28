use std::net::{IpAddr, Ipv4Addr};

lazy_static! {
	static ref INSTANCE: Config = Config::default();
}

pub struct Config
{
	pub serial_device_path: String,
	pub serial_baud_rate: u32,
	pub poll_serial: bool,
	pub bind_address: IpAddr,
	pub bind_port: u16
}

impl Default for Config
{
    fn default() -> Self
	{
		return Config {
			serial_device_path: String::from("/dev/ttyUSB1"),
			serial_baud_rate: 115200,
			poll_serial: true,
			bind_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
			bind_port: 2022
		};
	}
}

impl Config
{
	pub fn get() -> &'static Self
	{
		return &INSTANCE;
	}
}
