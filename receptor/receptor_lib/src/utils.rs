extern crate fs_extra;
extern crate hyper;
extern crate url;
extern crate yaml_rust;

use self::hyper::server::Request;
use self::url::Url;

use std::collections::HashMap;

pub fn get_url_params(req: &Request) -> HashMap<String, String> {
	let parsed_url = Url::parse(&format!("http://example.com/{}", req.uri())).unwrap();

	let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

	hash_query
}
