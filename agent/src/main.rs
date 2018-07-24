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
use std::{env::current_exe, net::SocketAddr, path::PathBuf};
use std::{thread, time};
use tokio::{net::TcpStream, prelude::Future};


#[derive(Debug, PartialEq, Deserialize)]
struct ReceptorAddr {
	host: String,
	port: i64
}

#[derive(Debug, PartialEq, Deserialize)]
struct Config {
	machine_identifier: Option<String>,
	receptor:           ReceptorAddr
}


fn main() -> Result<(), String> {
	env_logger::init();

	let mut exec_path_buff = current_exe().map_err(|e| e.to_string())?;
	exec_path_buff.pop();
	let exec_path = exec_path_buff
		.to_str()
		.ok_or_else(|| "Couldn't get execution path".to_string())?;

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

	let config = read_cfg::<Config>(&agent_config)?;

	let mut plugins = plugins::init(config_dir);

	let hostname = config
		.machine_identifier
		.unwrap_or(hostname::get_hostname().ok_or_else(|| "Couldn't get host name".to_string())?);

	let addr = format!("{}:{}", config.receptor.host, config.receptor.port);

	let mut sender = StatusSender::new(
		hostname,
		addr.parse()
			.map_err(|_| format!("Couldn't convert {} to Socket Address", addr))?
	);

	loop {
		thread::sleep(time::Duration::from_millis(1000));

		let mut payload = Vec::new();

		for p in &mut plugins {
			sender.arbitrate(&mut **p, &mut payload);
		}

		debug!("Payload content: {:?}", payload);

		if !payload.is_empty() {
			let serialized_payload = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

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
