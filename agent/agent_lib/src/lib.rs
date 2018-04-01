extern crate shared_lib;

pub use shared_lib::{current_ts, get_yml_config, Status};

pub trait AgentPlugin {
	fn name(&self) -> String;

	fn gather(&mut self) -> Result<String, String,>;

	fn ready(&self) -> bool;
}
