/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/

extern crate agent_lib;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use agent_lib::{current_ts, read_cfg, AgentPlugin};

use std::collections::HashMap;
use std::process::Command;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
	enabled:         bool,
	periodicity_arr: Vec<i64>,
	commands:        Vec<Vec<String>>
}


pub struct Plugin {
	last_call_map:   HashMap<String, i64>,
	periodicity_map: HashMap<String, i64>,
	commands:        Vec<Vec<String>>,
	enabled:         bool
}


impl Plugin {
	fn config(plugin: &mut Plugin) -> Result<(), String> {
		let cfg = read_cfg::<Config>("command_runner.yml")?;
		plugin.enabled = cfg.enabled;
		if !plugin.enabled {
			return Ok(());
		}
		plugin.commands = cfg.commands;
		for i in 0..plugin.commands.len() {
			let command_name = plugin.commands[i].join(" ");
			plugin
				.periodicity_map
				.insert(command_name.clone(), cfg.periodicity_arr[i]);
			plugin.last_call_map.insert(command_name, 0);
		}
		Ok(())
	}
}

pub fn new() -> Result<Plugin, String> {
	let mut new_plugin = Plugin {
		enabled:         false,
		last_call_map:   HashMap::new(),
		periodicity_map: HashMap::new(),
		commands:        Vec::new()
	};

	Plugin::config(&mut new_plugin)?;

	if new_plugin.enabled {
		Ok(new_plugin)
	} else {
		Err("Command runner disabled".into())
	}
}

impl AgentPlugin for Plugin {
	fn name(&self) -> &str {
		"Command runner"
	}

	fn gather(&mut self) -> Result<String, String> {
		let mut results = HashMap::new();

		for command in &self.commands {
			let command_name = command.join(" ");

			self.last_call_map.insert(command_name.clone(), current_ts());

			let mut cmd = Command::new(&command[0]);

			if command.len() > 1 {
				cmd.args(&command[1..command.len()]);
			}

			let output = cmd.output().map_err(|e| e.to_string())?;

			let str_output = if output.status.success() {
				String::from_utf8(output.stdout).map_err(|e| e.to_string())?
			} else {
				String::from_utf8(output.stderr).map_err(|e| e.to_string())?
			};

			results.insert(command_name, str_output);
		}

		let message = serde_json::to_string(&results).map_err(|e| e.to_string())?;

		Ok(message)
	}

	fn ready(&self) -> bool {
		if !self.enabled {
			return false;
		}

		self.last_call_map
			.iter()
			.any(|(k, v)| v + self.periodicity_map[k] < current_ts())
	}
}
