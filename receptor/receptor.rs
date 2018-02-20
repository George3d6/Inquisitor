#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate bincode;
use bincode::{deserialize};
use serde_json::{to_string};

extern crate rusqlite;
use rusqlite::Connection;

extern crate futures;
extern crate tokio;
extern crate tokio_io;
use futures::{Future, Stream};
use tokio::executor::current_thread;
use tokio::net::{TcpStream, TcpListener};
use tokio_io::{io, AsyncRead};

extern crate hyper;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};

mod status;
use status::*;

mod database;
use database::*;

use std::vec::Vec;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;


struct DataServer {
    pub db_conn: Connection,
}

impl Service for DataServer {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {

        let mut response = Response::new();
        match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                let mut raw_statuses = self.db_conn.prepare("SELECT * FROM raw_status").expect("Can't select from raw status database");
                let raw_status_iter = raw_statuses.query_map(&[], |row| {
                    Status{sender: row.get(0), message: row.get(1), plugin_name: row.get(2), ts: row.get(3)}
                }).expect("Problem getting raw status");
                for s in raw_status_iter {
                    println!("Sender is {:?}", s.unwrap().sender);
                }
                response.set_body(String::from("AA"));
            }
            _ => {
                response.set_status(StatusCode::NotFound);
            },
        }
        Box::new(futures::future::ok(response))
    }
}


fn proces_status(stream: TcpStream, db_conn: Rc<RefCell<Connection>>) {
    let (reader, _) = stream.split();
    let conn = io::read_to_end(reader, Vec::new()).then(move |res| {
        let payload = Vec::from(res.expect("Can't read input from agent").1);
        let status: Status = deserialize(&payload).expect("Can't deserialize status");
        db_conn.borrow_mut().execute("INSERT INTO raw_status(sender, message, plugin_name, ts_sent)
            VALUES (?1, ?2, ?3, ?4)", &[&status.sender, &status.message, &status.plugin_name,
            &status.ts.to_string()]).expect("Can't insert status into raw_status table");
        Ok(())
    });
    current_thread::spawn(conn);
}


fn main() {
    initialize_database();

    thread::spawn(move || {
        let server_addr = "127.0.0.1:1834".parse().expect("Can't parse HTTP server address");
        let server = Http::new().bind(&server_addr, move || Ok(DataServer{db_conn: get_connection()})).expect("Can't start HTTP server");
        server.run();
    });


    let listener_addr = "127.0.0.1:1478".parse().expect("Can't parse TCP server address");
    let listener = TcpListener::bind(&listener_addr).expect("Can't start TCP server");

    let db_conn: Rc<RefCell<Connection>> = Rc::new(RefCell::new(get_connection()));
    let receptor = listener.incoming().for_each(move |stream| {
        proces_status(stream, db_conn.clone());
        Ok(())
    }).map_err(|err| {
        println!("IO error {:?}", err);
    });

    current_thread::run(|_| {
        current_thread::spawn(receptor);

    });
}
