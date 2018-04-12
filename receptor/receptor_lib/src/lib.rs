extern crate rusqlite;
extern crate shared_lib;

pub mod utils;

use rusqlite::Connection;
pub use shared_lib::{current_ts, get_yml_config, read_cfg, Status};

pub trait ReceptorPlugin {
	fn name(&self) -> &'static str;

	fn gather(&mut self, db_conn: &Connection) -> Result<String, String>;

	fn ready(&self) -> bool;
}
