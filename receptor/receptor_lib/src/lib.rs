extern crate rusqlite;
extern crate shared_lib;

pub mod utils;

use rusqlite::Connection;
pub use shared_lib::{current_ts, get_yml_config, Status};
use std::string::String;

pub trait ReceptorPlugin {
	fn name(&self) -> String;

	fn gather(&mut self, db_conn: &Connection,) -> Result<String, String,>;

	fn ready(&self) -> bool;
}
