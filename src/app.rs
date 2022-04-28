use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;
use tokio::sync::watch::error::SendError;
use tokio::time::sleep;
use tokio_serial::SerialPortBuilderExt;

use crate::chunk::Chunk;

lazy_static! {
	static ref INSTANCE: App = {
		return App::new();
	};
}

pub struct App;

impl App
{
	pub fn instance() -> &'static Self
	{
		return &INSTANCE;
	}

	pub fn new() -> Self
	{
		return App {};
	}

	pub async fn run(&'static self) -> Result<(), std::io::Error>
	{
		let port = 2021u16;
		let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
		let listener = TcpListener::bind(address).await?;

		let (sender, receiver) = watch::channel::<Chunk>(Chunk::default());

		tokio::spawn(async move {
			loop {
				self.read_serial(&sender).await.unwrap();

				// Wait for the serial port to become available
				//sleep(std::time::Duration::from_millis(3000)).await;
			}
		});

		loop {
			log::info!("Listening for connections on port {}...", port);

			let (socket, peer_address) = listener.accept().await?;
			let receiver_instance = receiver.clone();
			tokio::spawn(async move {
				self.serve(socket, peer_address, receiver_instance).await.unwrap();
			});
		}
	}

    async fn serve(&self,
		mut socket: TcpStream,
		peer_address: SocketAddr,
		mut receiver: Receiver<Chunk>) -> Result<(), std::io::Error>
    {
		log::info!("{:?} connected", peer_address);

		loop {
			if let Ok(()) = receiver.changed().await {
				let data = self.get_data_copy(&receiver).await;

				if data.len() > 0 {
					if let Err(_) = socket.write_all(data.as_slice()).await {
						log::warn!("Failed to send data to peer {:?}, disconnecting...",
							peer_address);
						socket.shutdown().await?;

						let result: Result<(), std::io::Error> = Ok(());
						return result;
					}
				}
				else {
					log::warn!("Ignoring empty chunk from the serial data sender")
				}
			}
			else {
				// The sender has been dropped; disconnect gracefully from the peer.
				log::warn!("Serial data sender has been dropped, disconnecting from peer: {:?}",
					peer_address);
				socket.shutdown().await?;

				let result: Result<(), std::io::Error> = Ok(());
				return result;
			}
		}
    }

	async fn get_data_copy(&self, receiver_instance: &Receiver<Chunk>) -> Vec<u8>
	{
		// Quickly copy the data into a vec and release the borrowed chunk
		// TODO: use borrow_and_update() instead?
		let chunk = receiver_instance.borrow();
		return chunk.data[0 .. chunk.size].to_vec();
	}

	pub async fn read_serial(&self, sender: &Sender<Chunk>) -> Result<(), std::io::Error>
	{
		let path = "/dev/ttyUSB0";
		let baud_rate = 115200u32;
		let mut port = tokio_serial::new(path, baud_rate).open_native_async()?;
		let mut buffer = [0_u8; 1024];

		log::info!("Opened {} with a baud rate of {}", path, baud_rate);

		loop {
			let bytes_read = port.read(&mut buffer).await?;

			if bytes_read > 0 {
				//let string = String::from_utf8_lossy(&buffer);
				//log::debug!("Data ({} bytes):\n{}", bytes_read, string);

				let chunk = Chunk {
					data: buffer.clone(),
					size: bytes_read
				};

				sender.send(chunk).or_else(|_: SendError<Chunk>| -> Result<(), ()> {
					log::warn!("Failed to transfer obtained serial data to the TCP connection(s)");
					return Ok(());
				}).unwrap();
			}
			else {
				// No more data
				sleep(std::time::Duration::from_millis(500)).await;
			}
		}
	}
}
