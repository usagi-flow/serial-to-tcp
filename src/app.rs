use std::net::SocketAddr;

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
use crate::config::Config;

lazy_static! {
	static ref INSTANCE: App = {
		return App::new();
	};
}

pub struct App
{
}

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
		let address = SocketAddr::new(Config::get().bind_address, Config::get().bind_port);
		let listener = TcpListener::bind(address).await?;

		let (sender, receiver) = watch::channel::<Chunk>(Chunk::default());

		tokio::spawn(async move {
			loop {
				self.read_serial(&sender).await.unwrap();

				if Config::get().poll_serial {
					// Wait for the serial port to become available
					sleep(std::time::Duration::from_millis(3000)).await;
				}
				else {
					log::error!("Serial port could not be opened");
					break;
				}
			}
		});

		log::info!("Listening for connections on {}:{}...",
			Config::get().bind_address, Config::get().bind_port);

		loop {
			let (socket, peer_address) = listener.accept().await?;
			let receiver_instance = receiver.clone();

			// Each time a client connects, handle its connection asynchronously and accept new connections as
			// soon as possible.
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

	pub async fn read_serial(&self, sender: &Sender<Chunk>) -> Result<(), String>
	{
		let path = &Config::get().serial_device_path;
		let baud_rate = Config::get().serial_baud_rate;
		let mut buffer = [0_u8; 1024];
		let mut port;

		match tokio_serial::new(path, baud_rate).open_native_async() {
			Ok(opened_port) => port = opened_port,
			// If we're polling, don't treat a failed attempt to open the port as an error.
			Err(_) if Config::get().poll_serial => return Ok(()),
			// If we're not polling, do treat a failed attempt to open the port as an error.
			Err(e) => return Err(e.to_string())
		}

		log::info!("Opened {} with a baud rate of {}", path, baud_rate);

		loop {
			let bytes_read;

			match port.read(&mut buffer).await {
				Ok(bytes_read_) => bytes_read = bytes_read_,
				// If we're polling, don't treat a failed attempt to read as an error.
				Err(_) if Config::get().poll_serial => {
					if let Err(_) = port.shutdown().await {
						log::warn!("Failed to cleanly shut down the serial port after failing to read from it.");
					}
					return Ok(());
				}
				// If we're not polling, do treat a failed attempt to read as an error.
				Err(e) => return Err(e.to_string())
			}

			if bytes_read > 0 {
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
