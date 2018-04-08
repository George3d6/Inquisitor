extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate shared_lib;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::Client;
use hyper::{Method, Request};
use hyper::header::{ContentLength, ContentType};
use hyper::Uri;
use std::{thread, time};
use std::vec::Vec;
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
	monitor:	Vec<Check>
}


fn main() {
	let mut core = Core::new().expect("Can't start the tokio core !");
	let mut handle1 = core.handle();

	let client = Client::configure()
		.connector(::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
		.build(&core.handle());

	let my_endpoint = "your_app/enpoint/w.e you call it add in config later";

	let cfg = read_cfg::<Config>("config.yml").expect("Can't find config.yml file");

	let receptor_uri_base = format!("{}:{}", cfg.receptor.host, cfg.receptor.port);
	let slack_uri: Uri = format!("https://hooks.slack.com/services/{}", my_endpoint).parse().expect("Can't parse url");

	loop {
		thread::sleep(time::Duration::from_millis(1000));

		for check in &cfg.monitor {
			let receptor_uri: Uri = format!("{}?level={}&name={}&ts_start=0&ts_end=1823146529",
			receptor_uri_base, &check.level, &check.plugin).parse().expect("Can't parse url");

			println!("1");
			let get = client.get(receptor_uri).map(|rres| {
				println!("2");
				let message = rres.body().concat2();

				let json = format!("{:?}", &message);

				let mut sreq = Request::new(Method::Post, slack_uri.clone());
				sreq.headers_mut().set(ContentType::json());
				sreq.headers_mut().set(ContentLength(json.len() as u64));

				println!("Got from receptor: {}", &json);

				sreq.set_body(json);

				let post = client.request(sreq).and_then(|res| {
					println!("POST: {}", res.status());
					res.body().concat2()
				});
				//handle1.spawn(post).expect("Can't run post to slack endpoint");
			});
			core.run(get).expect("Can't run get to receptor");
		}
	}
}
