#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod status;
use status::Status;
mod plugin_interface;
mod utils;
use plugin_interface::AgentPlugin;

use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};
use std::{thread, time};

mod plugins;


struct StatusSender {
    addr:  SocketAddr,
}

impl StatusSender {
    fn new() -> StatusSender {
        return StatusSender{addr: SocketAddr::from(([127, 0, 0, 1], 1478))}
    }

    pub fn arbitrate<PluginType>(&mut self, plugin: &mut PluginType) where PluginType: AgentPlugin {
        if plugin.ready() {
            let name = plugin.name();
            let message = plugin.gather().expect(&format!("Issue running gather on plugin: {}", name) as &str);
            let status = Status{sender: String::from("Add sender to config and/or autodetect sender"), ts: utils::current_ts(), message: message, plugin_name: name};

            let payload = serde_json::to_string(&status).expect("Can't serialize payload");
            let mut stream = TcpStream::connect(&self.addr).expect("Can't create tcp stream");
            stream.write(&payload.as_bytes()).expect("Can't write to tcp stream !");
            stream.flush().expect("Can't flush the tcp stream !");
        }
    }
}


fn test_messages() {
    {{CREATE_PLUGINS}}


    let mut sender = StatusSender::new();

    loop {
        thread::sleep(time::Duration::from_millis(1000));
        {{USE_PLUGINS}}
    }
}


fn main() {
    test_messages();
}
