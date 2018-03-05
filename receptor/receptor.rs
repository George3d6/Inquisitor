#[macro_use]
extern crate serde_derive;
extern crate serde_json;

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
use status::Status;
mod utils;

mod database;
use database::{initialize_database, get_connection};

use std::vec::Vec;
use std::cell::RefCell;
use std::rc::Rc;
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
            (&Method::Get, "/") =>  response.set_body(String::from("I am alive !")),

            (&Method::Get, "/plugin_data") => {
                let params = utils::get_url_params(&req);
                let plugin_name = match params.get("name") {
                    Some(name) => name,
                    None => ""
                };

                let level = match params.get("level") {
                    Some(name) => name,
                    None => "",
                };

                let ts_start = match params.get("ts_start") {
                    Some(name) => name,
                    None => ""
                };

                let ts_end = match params.get("ts_end") {
                    Some(name) => name,
                    None => ""
                };

                let mut raw_data = if level == "raw" {
                    self.db_conn.prepare("SELECT * FROM raw_status WHERE strftime('%s',ts_received) > :ts_start AND strftime('%s',ts_received) < :ts_end AND plugin_name=:plugin_name").expect("Can't select from database")
                } else {
                    self.db_conn.prepare("SELECT * FROM processed_status WHERE strftime('%s',ts_received) > :ts_start AND strftime('%s',ts_received) < :ts_end AND plugin_name=:plugin_name").expect("Can't select from database")
                };

                let raw_status_iter = raw_data.query_map_named(&[(":ts_start", &ts_start), (":ts_end", &ts_end), (":plugin_name", &plugin_name)], |row| {
                    Status{sender: row.get(0), message: row.get(1), plugin_name: row.get(2), ts: row.get(3)}
                }).expect("Problem getting raw status");
                let status_csv_itter = raw_status_iter.map(|rs| {
                    let s = rs.expect("Corrupt status in database");
                    return format!("{}  {}  {}  {}", s.sender, s.message, s.plugin_name, s.ts)
                });
                let status_csv_vec: Vec<String> = status_csv_itter.collect();

                response.set_body(status_csv_vec.join("\n"));
            },

            _ => response.set_status(StatusCode::NotFound),
        }
        Box::new(futures::future::ok(response))
    }
}


fn proces_status(stream: TcpStream, db_conn: Rc<RefCell<Connection>>) {
    let (reader, _) = stream.split();
    let conn = io::read_to_end(reader, Vec::new()).then(move |res| {
        let payload = Vec::from(res.expect("Can't read input from agent").1);
        let status: Status = serde_json::from_slice(&payload).expect("Can't deserialize status");
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
        server.run().expect("Can't start hyper http server");
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
