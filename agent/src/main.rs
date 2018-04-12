#[macro_use]
extern crate log;
extern crate agent_lib;
extern crate env_logger;
extern crate hostname;
extern crate plugins;
extern crate serde_json;
extern crate tokio;

use agent_lib::{current_ts, get_yml_config, AgentPlugin, Status};
use std::net::SocketAddr;
use std::{thread, time};
use tokio::net::TcpStream;
use tokio::prelude::Future;


fn main() {
	env_logger::init();

	let mut plugins = plugins::init();

	let config = get_yml_config("agent_config.yml").unwrap();

	let hostname = config["machine_identifier"]
		.as_str()
		.map(String::from)
		.unwrap_or_else(|| hostname::get_hostname().unwrap());

	let addr = format!(
		"{}:{}",
		config["receptor"]["host"].as_str().unwrap(),
		config["receptor"]["port"].as_i64().unwrap()
	);

	let mut sender = StatusSender::new(hostname, addr.parse().expect("Couldn't convert IP address"));

	loop {
		thread::sleep(time::Duration::from_millis(1000));

		let mut payload = Vec::new();

		for p in &mut plugins {
			sender.arbitrate(&mut **p, &mut payload);
		}

		debug!("Payload content: {:?}", payload);

		if !payload.is_empty() {
			let serialized_payload = serde_json::to_string(&payload).expect("Can't serialize payload");

			let send = TcpStream::connect(&sender.addr)
				.and_then(|stream| tokio::io::write_all(stream, serialized_payload))
				.map_err(|e| error!("Error: {}", e))
				.map(|_| ());

			tokio::run(send);
		}
	}
}

struct StatusSender {
	pub addr:     SocketAddr,
	pub hostname: String
}

impl StatusSender {
	fn new(hostname: String, addr: std::net::SocketAddr) -> StatusSender {
		StatusSender { addr, hostname }
	}

	pub fn arbitrate(&mut self, plugin: &mut AgentPlugin, payload: &mut Vec<Status>) {
		if plugin.ready() {
			match plugin.gather() {
				Ok(message) => {
					let status = Status {
						sender: self.hostname.clone(),
						ts: current_ts(),
						message,
						plugin_name: plugin.name().to_string()
					};

					payload.push(status);
				}
				Err(err) => {
					error!("Error: {} ! When running gather for plguin {}", err, plugin.name());
					return;
				}
			};
		}
	}
}
