extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use futures::{Future, Stream};

use tokio_core::reactor::Core;

use hyper::Client;
use hyper::{Method, Request};
use hyper::header::{ContentLength, ContentType};

use std::{thread, time};


fn main() {
	let mut core = Core::new().expect("Can't start the tokio core !");

	let client = Client::configure()
		.connector(::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
		.build(&core.handle());

	let my_endpoint = "your_app/enpoint/w.e you call it add in config later";


	loop {
		thread::sleep(time::Duration::from_millis(1000));

		let uri = format!("https://hooks.slack.com/services/{}", my_endpoint)
			.parse()
			.expect("Can't parse url");

		let json = r#"{"text":"test"}"#;

		let mut req = Request::new(Method::Post, uri);
		req.headers_mut().set(ContentType::json());
		req.headers_mut().set(ContentLength(json.len() as u64));
		req.set_body(json);

		let post = client.request(req).and_then(|res| {
			println!("POST: {}", res.status());
			res.body().concat2()
		});

		core.run(post).expect("Can't run senders !");
	}
}
