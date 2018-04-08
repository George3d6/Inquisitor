extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate shared_lib;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::Client;
use hyper::{Method, Request};
use hyper::header::{ContentLength, ContentType};
use std::{thread, time};
use std::vec::Vector;
use shared_lib::{current_ts, read_cfg};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Check {
	plugin: String,
	sender: String,
	level: 	String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Receptor {
	host: String,
	port: u32
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	receptor:	Receptor,
	monitor:	Vector<Check>
}


fn main() {
	let mut core = Core::new().expect("Can't start the tokio core !");

	let client = Client::configure()
		.connector(::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
		.build(&core.handle());

	let my_endpoint = "your_app/enpoint/w.e you call it add in config later";

	let cfg = read_cfg::<Config>("config.yml")?;

	let receptor_uri_base = format!("{}:{}", cfg.receptor.host, cfg.receptor.port);
	let slack_uri = format!("https://hooks.slack.com/services/{}", my_endpoint).parse().expect("Can't parse url");

	loop {
		thread::sleep(time::Duration::from_millis(1000));

		for &check in monitor {
			let receptor_uri = format!("{}?level={}&name={}&ts_start=0&ts_end=1823146529", receptor_uri_base, check.level, check.name);

			let mut rreq = Request::new(Method::Get, &receptor_uri);
			let get = client.request(sreq).and_then(|rres| {
				let message = rres.body().concat2()

				let json = format!("{:?}", &message);

				let mut sreq = Request::new(Method::Post, &slack_uri);
				sreq.headers_mut().set(ContentType::json());
				sreq.headers_mut().set(ContentLength(json.len() as u64));
				sreq.set_body(json);

				let post = client.request(sreq).and_then(|res| {
					println!("POST: {}", res.status());
					res.body().concat2()
				});

			});

			core.run(get).expect("Can't get data from receptor");
		}
	}
}
