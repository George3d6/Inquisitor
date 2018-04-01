/*
    This plugin is simply used to make sure the agent is running.

    It's information can also be used by other plugins in order to determine machine reliability
    or find machines with unsynchronized clocks
*/
extern crate agent_lib;

use agent_lib::{current_ts, get_yml_config, AgentPlugin};


pub struct Plugin {
	last_call_ts: i64,
	periodicity:  i64,
	enabled:      bool
}


impl Plugin {
	fn config(&mut self) -> Result<(), String> {
		let config = get_yml_config("alive.yml").map_err(|e| e.to_string())?;
		if config["enabled"].as_bool().unwrap_or(false) {
			self.enabled = true;
		} else {
			self.enabled = false;
			return Ok(());
		}

		self.periodicity = match config["periodicity"].as_i64() {
			Some(val) => val,
			_ => return Err("Can't properly read key periodicity !".to_string())
		};
		return Ok(());
	}
}


pub fn new() -> Result<Plugin, String> {
	let mut new_plugin = Plugin {
		enabled:      false,
		last_call_ts: 0,
		periodicity:  0
	};

	let error = Plugin::config(&mut new_plugin);

	match error {
		Ok(()) => return Ok(new_plugin),
		Err(err) => return Err(err)
	};

	if new_plugin.enabled {
		Ok(new_plugin)
	} else {
		Err("Alive plugins disabled".into())
	}
}


impl AgentPlugin for Plugin {
	fn name(&self) -> String {
		String::from("Alive")
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
