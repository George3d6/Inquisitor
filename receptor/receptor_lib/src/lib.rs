extern crate rusqlite;
extern crate inquisitor_shared_lib;

pub mod utils;

use rusqlite::Connection;
pub use inquisitor_shared_lib::{current_ts, get_yml_config, read_cfg, Status};

pub trait ReceptorPlugin {
	fn name(&self) -> &str;

	fn gather(&mut self, db_conn: &Connection) -> Result<String, String>;

	fn ready(&self) -> bool;
}
