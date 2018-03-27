#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate plugins;
extern crate serde_json;
extern crate tokio;
extern crate agent_lib;
extern crate hostname;

mod status;

use futures::Future;
use tokio::net::TcpStream;
use status::Status;
use agent_lib::utils;
use agent_lib::plugin_interface::AgentPlugin;
use std::net::SocketAddr;
use std::{thread, time};

fn main() {
    let mut plugins = plugins::init();

    let config = utils::get_yml_config("agent_config.yml");

    let hostanme = config["machine_identifier"]
        .as_str()
        .map(|s| String::from(s))
        .unwrap_or(hostname::get_hostname().unwrap());

    let addr = format!(
        "{}:{}",
        config["receptor"]["host"].as_str().unwrap(),
        config["receptor"]["port"].as_i64().unwrap()
    );

    let mut sender = StatusSender::new(hostanme, addr);
    loop {
        thread::sleep(time::Duration::from_millis(1000));
        let mut payload = Vec::new();

        for mut p in &mut plugins {
            sender.arbitrate(&mut p, &mut payload);
        }

        if payload.len() > 0 {
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
    fn new(hostname: String, addr_str: String) -> StatusSender {
        StatusSender {
            addr: addr_str.parse().unwrap(),
            hostname: hostname,
        }
    }

    pub fn arbitrate(&mut self, plugin: &mut Box<AgentPlugin>, payload: &mut Vec<Status>)
    {
        if plugin.ready() {
            let name = plugin.name();
            match plugin.gather() {
                Ok(message) => {
                    let status = Status {
                        sender: self.hostname.clone(),
                        ts: utils::current_ts(),
                        message: message,
                        plugin_name: name,
                    };
                    payload.push(status);
                }
                Err(_) => (),
            }
        }
    }
}
