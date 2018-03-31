extern crate hyper;
extern crate yaml_rust;
extern crate fs_extra;
extern crate url;

use self::yaml_rust::{Yaml, YamlLoader};
use self::hyper::server::Request;
use self::fs_extra::file::read_to_string;
use self::url::Url;

use std::collections::HashMap;
use std::env::current_exe;
use std::time::{SystemTime, UNIX_EPOCH};


pub fn get_url_params(req: &Request) -> HashMap<String, String> {
    let parsed_url = Url::parse(&format!("http://badhyper.io/{}", req.uri().as_ref())).unwrap();
    let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    hash_query
}
