extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate hyper_staticfile;
extern crate inquisitor_shared_lib;
extern crate tokio_core;
#[macro_use]
extern crate log;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;

use futures::Future;
use futures::Stream;
use tokio_core::reactor::Core;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use hyper_staticfile::Static;
use std::string::String;
use std::path::Path;
use inquisitor_shared_lib::read_cfg;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Receptor {
	host: String,
	port: u32
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	static_file_path: String,
	bind:             String,
	port:             u32,
	receptor:         Receptor
}

struct DataServer {
	static_:       Static,
	receptor_addr: String
}

impl Service for DataServer {
	type Request = Request;

	type Response = Response;

	type Error = hyper::Error;

	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Request) -> Self::Future {
		if req.path() == "/plugin_data" {
			let mut response = Response::new();

			match (req.method(), req.path()) {
				(&Method::Get, "/plugin_data") => {
					let proxy_addrs = format!(
						"{}{}?{}",
						self.receptor_addr,
						"/plugin_data",
						req.uri().query().unwrap_or("")
					);
					debug!("For /plugin_data calling receptor at: {}", proxy_addrs);
					let text = reqwest::get(&proxy_addrs).unwrap().text().unwrap();
					response.set_body(text);
				}
				_ => response.set_status(StatusCode::NotFound)
			}

			Box::new(futures::future::ok(response))
		} else if req.path() == "/plugin_list" {
			let mut response = Response::new();

			match (req.method(), req.path()) {
				(&Method::Get, "/plugin_list") => {
					let proxy_addrs = format!(
						"{}{}?{}",
						self.receptor_addr,
						"/plugin_list",
						req.uri().query().unwrap_or("")
					);
					debug!("For /plugin_list calling receptor at: {}", proxy_addrs);
					let text = reqwest::get(&proxy_addrs).unwrap().text().unwrap();
					response.set_body(text);
				}
				_ => response.set_status(StatusCode::NotFound)
			}

			Box::new(futures::future::ok(response))
		} else {
			self.static_.call(req)
		}
	}
}

fn main() {
	env_logger::init();

	let cfg = read_cfg::<Config>("config.yml").unwrap();
	debug!("Running with configuration {:?}", cfg);

	let mut core = Core::new().unwrap();
	let handle = core.handle();
	let handle_cp_1 = handle.clone();
	let handle_cp_2 = handle.clone();

	let addr_str = format!("{}:{}", cfg.bind, cfg.port);
	let addr = &addr_str.parse().unwrap();
	debug!("Running the web ui from addr: {}", &addr_str);

	let receptor_addr = format!("{}:{}", cfg.receptor.host, cfg.receptor.port);
	debug!("Calling the receptor from addr: {}", &receptor_addr);

	let serve = Http::new()
		.serve_addr_handle(&addr, &handle, move || {
			Ok(DataServer {
				static_:       Static::new(&handle_cp_1, Path::new(&cfg.static_file_path)),
				receptor_addr: receptor_addr.clone()
			})
		})
		.expect("Can't start HTTP server");

	debug!("Spawning server !");
	handle.spawn(
		serve
			.for_each(move |conn| {
				handle_cp_2.spawn(conn.map(|_| ()).map_err(|err| println!("srv1 error: {:?}", err)));

				Ok(())
			})
			.map_err(|_| ())
	);

	info!("Running the web ui server !");
	core.run(futures::future::empty::<(), ()>()).unwrap();
}
