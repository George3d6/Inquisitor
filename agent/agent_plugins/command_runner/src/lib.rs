/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/

extern crate inquisitor_lib;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use inquisitor_lib::{current_ts, read_cfg, AgentPlugin};

use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;


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
	enabled:         bool,
	cfg_file:        PathBuf
}

pub fn new(mut cfg_path: PathBuf) -> Result<Plugin, String> {
	let cfg = read_cfg::<Config>(cfg_file)?;
	if !cfg.enabled {
		return Err("Command runner disabled".into());
	}
	let mut new_plugin = Plugin {
		enabled: cfg.enabled,
		last_call_map: HashMap::new(),
		periodicity_map: HashMap::new(),
		commands: cfg.commands,
		cfg_file
	};
	for i in 0..new_plugin.commands.len() {
		let command_name = new_plugin.commands[i].join(" ");
		new_plugin
			.periodicity_map
			.insert(command_name.clone(), cfg.periodicity_arr[i]);
		new_plugin.last_call_map.insert(command_name, 0);
	}
	Ok(new_plugin)
}

impl AgentPlugin for Plugin {
	fn name(&self) -> &'static str {
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

		serde_json::to_string(&results).map_err(|e| e.to_string())
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
