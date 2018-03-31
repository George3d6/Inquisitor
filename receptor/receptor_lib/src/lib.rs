pub extern crate shared_lib;
extern crate rusqlite;
pub mod utils;
use rusqlite::Connection;
use std::string::String;

pub trait ReceptorPlugin {
    fn name(&self) -> String;
    fn gather(&mut self, db_conn: &Connection) -> Result<String, String>;
    fn ready(&self) -> bool;
}
