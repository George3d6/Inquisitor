#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate yaml_rust;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hyper;
extern crate rusqlite;
extern crate url;

use self::yaml_rust::{Yaml, YamlLoader};
extern crate fs_extra;
use self::fs_extra::file::read_to_string;
use self::hyper::server::Request;
use self::url::Url;
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};


#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
	pub sender:      String,
	pub ts:          i64,
	pub message:     String,
	pub plugin_name: String
}

pub fn current_ts() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("Error getting system time !?")
		.as_secs() as i64
}

pub fn read_cfg<ConfigT>(cfg_file_path: String) -> Result<ConfigT, String>
where
	ConfigT: DeserializeOwned
{
	debug!("Reading config from: {:?}", cfg_file_path.display());
	let cfg_str = read_to_string(&cfg_file_path).map_err(|e| e.to_string())?;
	let cfg: ConfigT = serde_yaml::from_str(&cfg_str).map_err(|e| e.to_string())?;
	Ok(cfg)
}

pub trait AgentPlugin {
	fn name(&self) -> &'static str;

	fn gather(&mut self) -> Result<String, String>;

	fn ready(&self) -> bool;
}

pub fn get_url_params(req: &Request) -> HashMap<String, String> {
	let parsed_url = Url::parse(&format!("http://example.com/{}", req.uri())).unwrap();

	let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

	hash_query
}

pub trait ReceptorPlugin {
	fn name(&self) -> &'static str;

	fn gather(&mut self, db_conn: &Connection) -> Result<String, String>;

	fn ready(&self) -> bool;
}
