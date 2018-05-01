#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate hostname;
extern crate inquisitor_lib;
extern crate plugins;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate tokio;

use clap::{App, Arg};
use inquisitor_lib::{current_ts, read_cfg, AgentPlugin, Status};
use std::env::current_exe;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::{thread, time};
use tokio::net::TcpStream;
use tokio::prelude::Future;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ReceptorAddr {
	host: String,
	port: i64
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	machine_identifier: Option<String>,
	receptor:           ReceptorAddr
}


fn main() {
	env_logger::init();

	let mut exec_path_buff = current_exe().unwrap();
	exec_path_buff.pop();
	let exec_path = exec_path_buff.into_os_string().into_string().unwrap();

	let matches = App::new("Inquisitor agent")
		.version("0.3.1")
		.about(
			"The agent component of the inquisitor monitoring suite,
					  for more infomration visit: \
			 https://github.com/George3d6/Inquisitor"
		)
		.arg(
			Arg::with_name("config_dir")
				.long("config-dir")
				.help("The directory where the agent looks for it's configuration files")
				.default_value(&exec_path)
				.takes_value(true)
				.required(false)
		)
		.get_matches();

	// Produce config path
	let config_dir = PathBuf::from(matches.value_of("config_dir").unwrap()); //_or(&cfg_file_path_str);
	let mut agent_config = config_dir.clone();
	agent_config.push("agent_config.yml");

	let config = read_cfg::<Config>(&agent_config).unwrap();

	let mut plugins = plugins::init(config_dir);

	let hostname = config
		.machine_identifier
		.unwrap_or_else(|| hostname::get_hostname().unwrap());

	let addr = format!("{}:{}", config.receptor.host, config.receptor.port);

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
