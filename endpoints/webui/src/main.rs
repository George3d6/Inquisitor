extern crate shared_lib;
extern crate hyper;
extern crate hyper_staticfile;
extern crate futures;
extern crate tokio_core;
extern crate env_logger;
#[macro_use] extern crate log;

use futures::Future;
use futures::Stream;
use tokio_core::reactor::Core;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use hyper_staticfile::Static;
use std::env::current_exe;
use std::path::Path;


struct DataServer {
	static_:     Static
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
				(&Method::Get, "/plugin_list") => {
					response.set_body("");
				}
				_ => response.set_status(StatusCode::NotFound)
			}

			Box::new(futures::future::ok(response))

		}
        else if req.path() == "/plugin_list" {
			let mut response = Response::new();

			match (req.method(), req.path()) {
				(&Method::Get, "/plugin_list") => {
					response.set_body("");
				}
				_ => response.set_status(StatusCode::NotFound)
			}

			Box::new(futures::future::ok(response))
		}
        else {
			self.static_.call(req)
		}
	}
}


fn main() {
    env_logger::init();
    let mut root = current_exe().unwrap();
    for _ in 0..3 {
		root.pop();
	}
    let web_ui_root = format!("{}{}", root.to_str().unwrap(), "/static/");
    debug!("Serving the web ui's static files from: {}", web_ui_root);

    let mut core = Core::new().unwrap();
    let handle = core.handle();
	let handle_cp_1 = handle.clone();
	let handle_cp_2 = handle.clone();

    let addr = "127.0.0.1:3000".parse().unwrap();

    let serve = Http::new()
        .serve_addr_handle(&addr, &handle, move || { Ok(DataServer {static_: Static::new(&handle_cp_1, Path::new(&web_ui_root))}) })
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

	core.run(futures::future::empty::<(), ()>()).unwrap();
}
