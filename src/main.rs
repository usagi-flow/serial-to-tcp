#[macro_use]
extern crate lazy_static;

mod app;
mod chunk;

use app::App;

#[tokio::main]
async fn main()
{
	//let package_name = env!("CARGO_PKG_NAME").replace('-', "_");
	//let log_config = format!("info, {} = trace", package_name);
	//let logger = flexi_logger::Logger::try_with_str(log_config).unwrap();
	//logger.start().unwrap();

	simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info) .env().init().unwrap();

	let default_panic = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |info| {
		log::error!("A panic occurred, application terminating.");
		default_panic(info);
		std::process::exit(1);
	}));

	log::trace!("Starting application...");

	let app = App::instance();
	app.run().await.unwrap();
}
