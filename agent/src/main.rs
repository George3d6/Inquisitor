#![allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate plugins;
extern crate serde_json;
extern crate tokio;
use futures::Future;
use tokio::net::TcpStream;

mod status;
use status::Status;
extern crate hostname;

use std::net::SocketAddr;
use std::string::String;
use std::vec::Vec;
use std::{thread, time};

fn main() {
    let plugins = plugins::init();
    for p in plugins {
        println!("{}", p.name());
    }
}

/*struct StatusSender {
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

    pub fn arbitrate<PluginType>(&mut self, plugin: &mut PluginType, payload: &mut Vec<Status>)
    where
        PluginType: AgentPlugin,
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

fn main() {
    $$CREATE_PLUGINS$$

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

        $$USE_PLUGINS$$

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
*/
