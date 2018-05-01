//! This library is intended for use in both the Inquisitor Agent and the Inquisitor Receptor.
//! Plugin authors must implement the plugin trait for their desired platform, but they
//! may also make use of several convenience functions included in this library.
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
#[macro_use]
extern crate log;
extern crate rusqlite;

extern crate fs_extra;
use self::fs_extra::file::read_to_string;
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// This struct is for communication between agent and receptor.
/// Plugins should not use this directly
/// This struct is considered "internal API" and can change in patch versions
#[doc(hidden)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
	pub sender:      String,
	pub ts:          i64,
	pub message:     String,
	pub plugin_name: String
}

/// A utility function that returns the current timestamp in seconds
/// # Example
/// ```
/// # extern crate inquisitor_lib;
/// # use inquisitor_lib::current_ts;
/// #
/// # fn main() {
/// use std::{thread, time::Duration};
/// let ts1 = current_ts();
/// thread::sleep(Duration::from_secs(1));
/// let ts2 = current_ts();
/// assert_eq!(ts1, ts2 -1);
/// # }
/// ```
pub fn current_ts() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("Error getting system time !?")
		.as_secs() as i64
}

/// A utility function that returns a deserialized object from a configuration file.
/// Plugins should use this instead of manipulating their own configuration file
/// # Example
/// ```
/// #[macro_use]
/// extern crate serde_derive;
/// extern crate serde_yaml;
/// extern crate inquisitor_lib;
/// # use std::path::PathBuf;
/// # use std::fs::File;
/// # use std::io::Write;
/// use inquisitor_lib::read_cfg;
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize, Eq)]
/// struct Config {
/// 	enabled: bool,
/// }
///
/// fn main() {
/// 	// Setup
///     let config: Config = Config { enabled: true };
///     let p = PathBuf::from("test.yml");
///     let mut file = File::create(&p).unwrap();
///     file.write_all(serde_yaml::to_string(&config).unwrap().as_bytes()).unwrap();
///
/// 	// Read function
///     let read_config = read_cfg::<Config>(&p).unwrap();
///     assert_eq!(config, read_config);
/// 	# std::fs::remove_file(&p).unwrap()
/// }
///
/// ```
pub fn read_cfg<ConfigT>(cfg_file_path: &Path) -> Result<ConfigT, String>
where
	ConfigT: DeserializeOwned
{
	debug!("Reading config from: {:?}", cfg_file_path.display());
	let cfg_str = read_to_string(&cfg_file_path).map_err(|e| e.to_string())?;
	let cfg: ConfigT = serde_yaml::from_str(&cfg_str).map_err(|e| e.to_string())?;
	Ok(cfg)
}

/// This trait is required by agent plugins. Structs that implement this trait should
/// be called "Plugin" or your crate must have a `pub alias Plugin = $YOUR_PLUGIN_STRUCT`.
pub trait AgentPlugin {
	/// This creates the object the the agent uses to run the plugin.
	fn new(PathBuf) -> Result<Self, String>
	where
		Self: Sized;
	/// Returns the "human readable" name of your plugin. 
    /// Sent to the server to tag plugin messages.
	fn name(&self) -> &'static str;
	/// This is the 'worker' function, and will be called when the ready function returns true.
	/// Currently this requires plugins to return a string.
	/// An `Ok` will be sent to the server, while currently an `Err` is output to the terminal on the agent
	fn gather(&mut self) -> Result<String, String>;
	/// This function tells the agent if the plugin is ready to be run.
	fn ready(&self) -> bool;
}

/// This trait is required by receptor plugins. Structs that implement this trait should
/// be called "Plugin" or your crate must have a `pub alias Plugin = $YOUR_PLUGIN_STRUCT`.
pub trait ReceptorPlugin {
	/// This creates the object the the receptor uses to run the plugin.
	fn new(PathBuf) -> Result<Self, String>
	where
		Self: Sized;
	/// Returns the "human readable" name of your plugin.
    /// Stored in the database with the message returned.
	fn name(&self) -> &'static str;
	/// This is the 'worker' function, and will be called when the ready function returns true.
	/// Currently this requires plugins to return a string.
	/// An `Ok` will be stored, while currently an `Err` is output to the terminal on the receptor
	fn gather(&mut self, db_conn: &Connection) -> Result<String, String>;
	/// This function tells the receptor if the plugin is ready to be run.
	fn ready(&self) -> bool;
}
