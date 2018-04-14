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
use std::time::{SystemTime, UNIX_EPOCH};
use serde::de::DeserializeOwned;
use self::hyper::server::Request;
use self::url::Url;
use rusqlite::Connection;
use std::collections::HashMap;
use std::string::String;


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

pub fn get_yml_config(cfg_file_path: String) -> Result<Yaml, String> {
	let contents = match read_to_string(&cfg_file_path) {
		Ok(content) => content,
		Err(_) => return Err(format!("Config file {} not found !", cfg_file_path))
	};
	let mut docs = match YamlLoader::load_from_str(&contents) {
		Ok(docs) => docs,
		Err(_) => return Err(format!("File {}, content is not valid yml !", cfg_file_path))
	};
	if docs.is_empty() {
		return Err(format!("No valid yml documents inside: {} !", cfg_file_path));
	}
	Ok(docs.remove(0))
}

pub fn read_cfg<ConfigT>(cfg_file_path: String) -> Result<ConfigT, String>
where
	ConfigT: DeserializeOwned
{
	debug!("Reading config from: {:?}", cfg_file_path);
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
