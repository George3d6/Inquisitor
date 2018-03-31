extern crate shared_lib;
extern crate rusqlite;
pub mod utils;
pub use shared_lib::{current_ts, get_yml_config, Status};
use rusqlite::Connection;
use std::string::String;

pub trait ReceptorPlugin {
    fn name(&self) -> String;
    fn gather(&mut self, db_conn: &Connection) -> Result<String, String>;
    fn ready(&self) -> bool;
}
