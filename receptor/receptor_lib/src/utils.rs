extern crate fs_extra;
extern crate hyper;
extern crate url;
extern crate yaml_rust;

use self::fs_extra::file::read_to_string;
use self::hyper::server::Request;
use self::url::Url;
use self::yaml_rust::{Yaml, YamlLoader};

use std::collections::HashMap;
use std::env::current_exe;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_url_params(req: &Request) -> HashMap<String, String> {
    let parsed_url = Url::parse(&format!("http://badhyper.io/{}", req.uri().as_ref())).unwrap();
    let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    hash_query
}
