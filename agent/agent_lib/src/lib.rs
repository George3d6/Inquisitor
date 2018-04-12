extern crate inquisitor_shared_lib;

pub use inquisitor_shared_lib::{current_ts, get_yml_config, get_yml_config_string, read_cfg, Status};

pub trait AgentPlugin {
	fn name(&self) -> &str;

	fn gather(&mut self) -> Result<String, String>;

	fn ready(&self) -> bool;
}
