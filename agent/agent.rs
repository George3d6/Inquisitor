extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, deserialize};

extern crate time;

extern crate futures;
extern crate tokio;
extern crate tokio_io;
use futures::{Future, Stream};
use tokio::executor::current_thread;
use tokio::net::{TcpStream};
use tokio_io::{io, AsyncRead};

use std::io::prelude::*;
use std::net::TcpStream;

mod status;
use status::Status;

mod plugin_interface;
use plugin_interface::AgentPlugin;

mod plugins;


struct StatusSender {
    stream: TcpStream,
}

impl StatusSender {
    fn new() -> StatusSender {
        return StatusSender{stream: TcpStream::connect("127.0.0.1:1478").except("Can't create tcp stream")}
    }

    pub fn arbitrate<PluginType>(&self, &plugin: PluginType) {
        if(plugin.ready()) {
            let name = plugin.name();
            let message = plugin.gather().except("Issue running gather on plugin: " + name);
            let status = Status{sender: String::from("Add sender to config and/or autodetect sender")
            , ts: time::now_utc().tm_sec as i64, message: message, plugin_name: name};
        }
        /*
        fn name(&self) -> String;
        fn gather(&mut self) -> Result<String, String>;
        fn ready(&self) -> bool;
        */
    }
}


fn test_messages() {
    //Status{sender: String::from("George's computer, dynamic IP"), ts: time::now_utc().tm_sec as i64, message: String::from("test plugin 2"), plugin_name: String::from("plugin 2")}
    let mut sysinfo = plugins::system_monitor::Plugin::new();
    let info = sysinfo.gather().unwrap();

    let socket = TcpStream::connect(&"127.0.0.1:1478".parse().except("Can't parse address")).except("Can't start TCP stream");


    println!("{:?}", info);
}


fn main() {
    test_messages();
}
