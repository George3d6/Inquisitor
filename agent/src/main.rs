#[macro_use]
extern crate serde_derive;
extern crate agent_lib;
extern crate futures;
extern crate hostname;
extern crate plugins;
extern crate serde_json;
extern crate tokio;

mod status;

use agent_lib::AgentPlugin;
use agent_lib::utils;
use futures::Future;
use status::Status;
use std::net::SocketAddr;
use std::{thread, time};
use tokio::net::TcpStream;

fn main() {
    let mut plugins = plugins::init();

    let config = utils::get_yml_config("agent_config.yml");

    let hostanme = config["machine_identifier"]
        .as_str()
        .map(String::from)
        .unwrap_or_else(|| hostname::get_hostname().unwrap());

    let addr = format!(
        "{}:{}",
        config["receptor"]["host"].as_str().unwrap(),
        config["receptor"]["port"].as_i64().unwrap()
    );

    let mut sender =
        StatusSender::new(hostanme, addr.parse().expect("Couldn't convert IP address"));
    loop {
        let mut payload = Vec::new();

        for mut p in &mut plugins {
            sender.arbitrate(&mut p, &mut payload);
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

        let s = &plugins.iter().min_by_key(|x| x.when_ready()).unwrap();
        thread::sleep(time::Duration::from_secs(s.when_ready() as u64));
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

    pub fn arbitrate(&mut self, plugin: &mut Box<AgentPlugin>, payload: &mut Vec<Status>) {
        if plugin.ready() {
            let name = plugin.name();
            if let Ok(message) = plugin.gather() {
                let status = Status {
                    sender: self.hostname.clone(),
                    ts: utils::current_ts(),
                    message,
                    plugin_name: name,
                };
                payload.push(status);
            }
        }
    }
}
