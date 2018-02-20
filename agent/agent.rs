#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate time;
use bincode::{serialize, deserialize, Infinite};

mod status;
use status::*;

use std::vec::Vec;
use std::string::String;
use std::io::prelude::*;
use std::net::TcpStream;


fn test_messages() {
    let statuses = vec![Status {sender: String::from("George's computer, dynamic IP"), ts: time::now_utc().tm_sec as i64, message: String::from("test plugin 1"), plugin_name: String::from("plugin 1")},
                        Status {sender: String::from("George's computer, dynamic IP"), ts: time::now_utc().tm_sec as i64, message: String::from("test plugin 2"), plugin_name: String::from("plugin 2")}];

    let mut stream = TcpStream::connect("127.0.0.1:1478").expect("Can't initialize tcp stream");
    for status in statuses {
        let payload = serialize(&status, Infinite).expect("Can't serialize payload");
        stream.write(&payload);
        stream.flush();
    }
}


fn main() {
    test_messages();
}
