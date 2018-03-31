extern crate agent_lib;
extern crate futures;
extern crate hostname;
extern crate plugins;
extern crate serde_json;
extern crate shared_lib;
extern crate tokio;

use shared_lib::{get_yml_config, current_ts};
use agent_lib::AgentPlugin;
use futures::Future;
use shared_lib::Status;
use std::net::SocketAddr;
use std::{thread, time};
use tokio::net::TcpStream;

fn main() {
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

        if !payload.is_empty() {
            let serialized_payload =
                serde_json::to_string(&payload).expect("Can't serialize payload");

            let send = TcpStream::connect(&sender.addr)
                .and_then(|stream| tokio::io::write_all(stream, serialized_payload))
                .map_err(|e| eprintln!("Error: {}", e))
                .map(|_| ());

            tokio::run(send);
        }
    }
}

struct StatusSender {
    pub addr: SocketAddr,
    pub hostname: String,
}

impl StatusSender {
    fn new(hostname: String, addr: std::net::SocketAddr) -> StatusSender {
        StatusSender { addr, hostname }
    }

    pub fn arbitrate(&mut self, plugin: &mut AgentPlugin, payload: &mut Vec<Status>) {
        if plugin.ready() {
            let name = plugin.name();
            if let Ok(message) = plugin.gather() {
                let status = Status {
                    sender: self.hostname.clone(),
                    ts: current_ts(),
                    message,
                    plugin_name: name,
                };
                payload.push(status);
            }
        }
    }
}
