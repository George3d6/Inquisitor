#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate futures;
extern crate tokio;
use futures::Future;
use tokio::net::TcpStream;


mod status;
use status::Status;
mod plugin_interface;
mod utils;
use plugin_interface::AgentPlugin;
mod plugins;

extern crate hostname;

use std::net::SocketAddr;
use std::{thread, time};
use std::vec::Vec;
use std::string::String;


struct StatusSender {
    pub addr:  SocketAddr,
    pub hostname: String,
}

impl StatusSender {
    fn new(hostname: String) -> StatusSender {
        return StatusSender{addr: SocketAddr::from(([127, 0, 0, 1], 1478)), hostname: hostname}
    }

    pub fn arbitrate<PluginType>(&mut self, plugin: &mut PluginType, payload: &mut Vec<Status>) where PluginType: AgentPlugin {
        if plugin.ready() {
            let name = plugin.name();
            match plugin.gather() {
                Ok(message) => {
                    let status = Status{sender: self.hostname.clone(), ts: utils::current_ts(), message: message, plugin_name: name};
                    payload.push(status);
                }
                Err(_) => ()
            }
        }
    }
}


fn main() {
    {{CREATE_PLUGINS}}

    let config = utils::get_yml_config("agent_config.yml");

    let hostanme = match config["machine_identifier"].as_str() {
            Some(name) => String::from(name),
            None => hostname::get_hostname().unwrap(),
    };

    let mut sender = StatusSender::new(hostanme);
    loop {
        thread::sleep(time::Duration::from_millis(1000));
        let mut payload = Vec::new();

        {{USE_PLUGINS}}

        if payload.len() > 0 {
            let serialized_payload = serde_json::to_string(&payload).expect("Can't serialize payload");

            let send = TcpStream::connect(&sender.addr)
                .and_then(|stream| {
                    return tokio::io::write_all(stream, serialized_payload)
                }).map_err(|e| eprintln!("Error: {}", e)).map(|_| ());

            tokio::run(send);
        }
    }
}
