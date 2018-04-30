//! This library is intended for use in both the Inquisitor Agent and the Inquisitor Receptor.
//! Plugin authors must implement the plugin trait for their desired platform, but they
//! may also make use of several convience functions included in this library.
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
#[macro_use]
extern crate log;
extern crate hyper;
extern crate rusqlite;
extern crate url;

extern crate fs_extra;
use self::fs_extra::file::read_to_string;
use self::hyper::server::Request;
use self::url::Url;
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// This struct is for communication between agent and receptor.
/// Plugins should not use this directly
#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
	pub sender:      String,
	pub ts:          i64,
	pub message:     String,
	pub plugin_name: String
}

/// A utility function that returns the current timestamp in seconds
pub fn current_ts() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("Error getting system time !?")
		.as_secs() as i64
}

/// A utility function that returns a deserialized object from a configuration file.
/// Plugins should use this instead of manipulating their own configuration file
///
/// # TODO examples
pub fn read_cfg<ConfigT>(cfg_file_path: &PathBuf) -> Result<ConfigT, String>
where
	ConfigT: DeserializeOwned
{
	debug!("Reading config from: {:?}", cfg_file_path.display());
	let cfg_str = read_to_string(&cfg_file_path).map_err(|e| e.to_string())?;
	let cfg: ConfigT = serde_yaml::from_str(&cfg_str).map_err(|e| e.to_string())?;
	Ok(cfg)
}

/// This trait is required by agent plugins
pub trait AgentPlugin {
	/// Returns the plugin's name. Sent to the server to tag plugin messages
	fn name(&self) -> &'static str;
	/// This is the 'worker' function, and will be called when the ready fucntion returns true.
	/// Currently this requires plugins to return a string.
	/// An `Ok` will be sent to the server, while currently an `Err` is output to the terminal on the agent
	fn gather(&mut self) -> Result<String, String>;
	/// This function tells the agent if the plugin is ready to be run.
	fn ready(&self) -> bool;
}

pub fn get_url_params(req: &Request) -> HashMap<String, String> {
	let parsed_url = Url::parse(&format!("http://example.com/{}", req.uri())).unwrap();

	let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

	hash_query
}

/// This trait is required by receptor plugins
pub trait ReceptorPlugin {
	/// Returns the plugin's name. Stored in the database with the message returnd
	fn name(&self) -> &'static str;
	/// This is the 'worker' function, and will be called when the ready fucntion returns true.
	/// Currently this requires plugins to return a string.
	/// An `Ok` will be stored, while currently an `Err` is output to the terminal on the receptor
	fn gather(&mut self, db_conn: &Connection) -> Result<String, String>;
	/// This function tells the receptor if the plugin is ready to be run.
	fn ready(&self) -> bool;
}
