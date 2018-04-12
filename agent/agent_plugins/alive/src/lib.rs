/*
    This plugin is simply used to make sure the agent is running.

    It's information can also be used by other plugins in order to determine machine reliability
    or find machines with unsynchronized clocks
*/
extern crate agent_lib;
#[macro_use]
extern crate serde_derive;

use agent_lib::{current_ts, read_cfg, AgentPlugin};


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	enabled:     bool,
	periodicity: i64
}


pub struct Plugin {
	last_call_ts: i64,
	periodicity:  i64,
	enabled:      bool
}

pub fn new() -> Result<Plugin, String> {
	let cfg = read_cfg::<Config>("alive.yml")?;
	if cfg.enabled {
		Ok(Plugin {
			enabled:      true,
			last_call_ts: 0,
			periodicity:  cfg.periodicity
		})
	} else {
		Err("Alive plugin disabled".into())
	}
}

impl AgentPlugin for Plugin {
	fn name(&self) -> &str {
		"Alive"
	}

	fn gather(&mut self) -> Result<String, String> {
		self.last_call_ts = current_ts();
		Ok(String::from("I live"))
	}

	fn ready(&self) -> bool {
		if !self.enabled {
			return false;
		}
		self.last_call_ts + self.periodicity < current_ts()
	}
}
