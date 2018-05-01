/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/

extern crate inquisitor_lib;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use inquisitor_lib::{current_ts, read_cfg, AgentPlugin};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	enabled:         bool,
	periodicity_arr: Vec<i64>,
	processes:       Vec<String>
}


pub struct Plugin {
	last_call_map:   HashMap<String, i64>,
	periodicity_map: HashMap<String, i64>,
	processes:       Vec<String>,
	enabled:         bool
}

impl AgentPlugin for Plugin {
	fn new(mut cfg_path: PathBuf) -> Result<Plugin, String> {
		cfg_path.push("process_counter.yml");
		let cfg = read_cfg::<Config>(&cfg_path)?;
		if cfg.enabled {
			let mut plugin = Plugin {
				enabled:         true,
				last_call_map:   HashMap::new(),
				periodicity_map: HashMap::new(),
				processes:       cfg.processes
			};
			for i in 0..plugin.processes.len() {
				plugin
					.periodicity_map
					.insert(plugin.processes[i].clone(), cfg.periodicity_arr[i]);
				plugin.last_call_map.insert(plugin.processes[i].clone(), 0);
			}
			Ok(plugin)
		} else {
			Err("Process counter disabled".into())
		}
	}

	fn name(&self) -> &'static str {
		"Process counter"
	}

	fn gather(&mut self) -> Result<String, String> {
		let mut results = HashMap::new();

		for process in &self.processes {
			self.last_call_map.insert(process.clone(), current_ts());

			let mut cmd = Command::new("pgrep");

			cmd.arg("-f");

			cmd.arg(&process);

			let output = cmd.output().map_err(|e| e.to_string())?;

			let mut running: i64 = 0;

			if output.status.success() {
				let str_output = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;

				if !str_output.is_empty() {
					let v: Vec<&str> = str_output.split('\n').filter(|&x| x != "").collect();

					running = v.len() as i64;
				}
			}

			results.insert(process, running);
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
