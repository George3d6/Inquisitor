#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate rusqlite;
use rusqlite::Connection;

extern crate futures;
extern crate tokio;
extern crate tokio_core;
use futures::Future;
use tokio::net::{TcpStream, TcpListener};
use tokio_core::reactor::Core;
use futures::Stream;
use tokio::io::AsyncRead;

extern crate hyper;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};

extern crate hyper_staticfile;
use hyper_staticfile::Static;

extern crate fs_extra;

mod status;
use status::Status;
mod plugin_interface;
mod utils;
use plugin_interface::ReceptorPlugin;
mod plugins;

mod database;
use database::{initialize_database, get_connection};

use std::vec::Vec;
use std::{thread, time};
use std::env::current_exe;
use std::path::Path;


struct DataServer {
    pub db_conn: Connection,
    static_: Static,
}

impl Service for DataServer {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        if req.path() == "/plugin_data" {
            let mut response = Response::new();
            match (req.method(), req.path()) {

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
                    if level == "agent" {
                        let mut raw_data = self.db_conn.prepare("SELECT * FROM agent_status WHERE strftime('%s',ts_received) > :ts_start AND strftime('%s',ts_received) < :ts_end AND plugin_name=:plugin_name").expect("Can't select from database");

                        let raw_status_iter = raw_data.query_map_named(&[(":ts_start", &ts_start), (":ts_end", &ts_end), (":plugin_name", &plugin_name)], |row| {
                            (row.get(0), row.get(1), row.get(3))
                        }).expect("Problem getting raw status");
                        let status_tsv_itter = raw_status_iter.map(|rs| {
                            let (sender, message, ts): (String, String, i64) = rs.expect("Corrupt status in database");
                            return format!("{}\t{}\t{}", sender, message, ts)
                        });
                        let status_tsv_vec: Vec<String> = status_tsv_itter.collect();

                        response.set_body(status_tsv_vec.join("\n"));

                    } else {
                        let mut raw_data =  self.db_conn.prepare("SELECT message, strftime('%s', ts) FROM receptor_status WHERE strftime('%s',ts) > :ts_start AND strftime('%s',ts) < :ts_end AND plugin_name=:plugin_name").expect("Can't select from database");

                        let raw_status_iter = raw_data.query_map_named(&[(":ts_start", &ts_start), (":ts_end", &ts_end), (":plugin_name", &plugin_name)], |row| {
                            (row.get(0), row.get(1))
                        }).expect("Problem getting receptor_status tuple");
                        let status_tsv_itter = raw_status_iter.map(|rs| {
                            let (message, ts): (String, String) = rs.expect("Corrupt status in database");
                            return format!("{}\t{}", message, ts)
                        });
                        let status_tsv_vec: Vec<String> = status_tsv_itter.collect();

                        response.set_body(status_tsv_vec.join("\n"));
                    };


                },

                _ => response.set_status(StatusCode::NotFound),
            }
            Box::new(futures::future::ok(response))
        }
        else if req.path() == "/plugin_list" {
            let mut response = Response::new();
            match (req.method(), req.path()) {
                (&Method::Get, "/plugin_list") => {
                    let params = utils::get_url_params(&req);
                    let level = match params.get("level") {
                        Some(name) => name,
                        None => "",
                    };

                    let mut raw_data = if level == "agent" {
                        self.db_conn.prepare("SELECT DISTINCT(plugin_name) FROM agent_status").expect("Can't select from database")
                    } else {
                        self.db_conn.prepare("SELECT DISTINCT(plugin_name) FROM receptor_status").expect("Can't select from database")
                    };

                    let raw_status_iter = raw_data.query_map_named(&[], |row| {
                        row.get(0)
                    }).expect("Problem getting receptor_status tuple");

                    let status_tsv_itter = raw_status_iter.map(|s| {
                        let name: String = s.expect("Corrupt status in database");
                        return name
                    });

                    let status_tsv_vec: Vec<String> = status_tsv_itter.collect();

                    response.set_body(status_tsv_vec.join("\n"));
                }
                _ => response.set_status(StatusCode::NotFound),
            }
            Box::new(futures::future::ok(response))
        }
        else {
            self.static_.call(req)
        }

    }
}


fn proces_status(stream: TcpStream, db_conn: Connection) {
    let (reader, _) = stream.split();
    let conn = tokio::io::read_to_end(reader, Vec::new()).then(move |res| {
        let payload = Vec::from(res.expect("Can't read input from agent").1);
        let statuses: Vec<Status> = serde_json::from_slice(&payload).expect("Can't deserialize status");
        for status in statuses {
            db_conn.execute("INSERT INTO agent_status(sender, message, plugin_name, ts_sent)
                VALUES (?1, ?2, ?3, ?4)", &[&status.sender, &status.message, &status.plugin_name,
                &status.ts.to_string()]).expect("Can't insert status into agent_status table");
        }
        Ok(())
    });
    tokio::spawn(conn);
}


struct PluginRunner {
    pub db_conn: Connection,
}

impl PluginRunner {
    pub fn new() -> PluginRunner {
        return PluginRunner{db_conn: get_connection()}
    }

    pub fn run_plugin<PluginType>(&self, plugin: &mut PluginType) where PluginType: ReceptorPlugin {
        if plugin.ready() {
            let name = plugin.name();
            let message = plugin.gather(&self.db_conn).expect(&format!("Issue running gather on plugin: {}", name) as &str);

            self.db_conn.execute("INSERT INTO receptor_status(message, plugin_name)
                VALUES (?1, ?2)", &[&message, &name]).expect("Can't insert recptor side plugin data into receptor_status table");
        }
    }

}


fn main() {
    let config = utils::get_yml_config("receptor_config.yml");

    let clean_older_than = config["clean_older_than"].as_i64().expect("Please specify a time after which logs should start being removed from the database under the root parameter: 'clean_older_than' [type==i64]");

    initialize_database();

    /* Do some administrative sutff */
    thread::spawn(move || {
        loop {
             let db_conn = get_connection();

             db_conn.execute("DELETE FROM agent_status WHERE CAST(strftime('%s',ts_received) as decimal)
             < (CAST(strftime('%s',CURRENT_TIMESTAMP) as decimal) - ?1)", &[&clean_older_than])
             .expect("Can't clean up agent_status table");

             db_conn.execute("DELETE FROM receptor_status WHERE CAST(strftime('%s',ts) as decimal)
             < (CAST(strftime('%s',CURRENT_TIMESTAMP) as decimal) - ?1)", &[&clean_older_than])
             .expect("Can't clean up receptor_status table");

             thread::sleep(time::Duration::from_millis(360 * 1000));
        }
    });


    /* Run receptor side plugins */
    thread::spawn(|| {
        {{CREATE_PLUGINS}}

        let plugin_runner = PluginRunner::new();

        loop {
            {{USE_PLUGINS}}
            thread::sleep(time::Duration::from_millis(1000));
        }
    });


    /* Run http server for the web UI and http endpoints to get plugin data */
    let server_addr_str = format!("{}:{}", config["server"]["bind"].as_str().unwrap(), config["server"]["port"].as_i64().unwrap());
    thread::spawn(move || {
        let server_addr = server_addr_str.parse().expect("Can't parse HTTP server address");
        let mut root = current_exe().unwrap();
        root.pop();

        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let handle1 = handle.clone();
        let handler = handle.clone();
        let handleds = handle.clone();

        let serve = Http::new().serve_addr_handle(&server_addr, &handle, move || Ok(DataServer{db_conn: get_connection(),
            static_: Static::new(&handleds, Path::new( &format!("{}{}", root.to_str().unwrap(), "/web_ui/") ))})).expect("Can't start HTTP server");


        handler.spawn(serve.for_each(move |conn| {
            handle1.spawn(conn.map(|_| ()).map_err(|err| println!("srv1 error: {:?}", err)));
            Ok(())
        }).map_err(|_| ()));

        core.run(futures::future::empty::<(), ()>()).unwrap();
    });


    /* Listen for incoming statuses from agents and process them & validate them & insert them into the database */
    let receptor_addr_str = format!("{}:{}", config["receptor"]["bind"].as_str().unwrap(), config["receptor"]["port"].as_i64().unwrap());
    let listener_addr = receptor_addr_str.parse().expect("Can't parse TCP server address");
    let listener = TcpListener::bind(&listener_addr).expect("Can't start TCP server");

    let receptor = listener.incoming().for_each(move |stream| {
        proces_status(stream, get_connection());
        Ok(())
    }).map_err(|err| {
        println!("IO error {:?}", err);
    });

    tokio::run(receptor);
}
