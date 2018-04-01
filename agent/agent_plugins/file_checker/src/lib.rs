/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/
extern crate agent_lib;
#[macro_use]
extern crate serde_derive;
extern crate fs_extra;
extern crate serde_json;

use agent_lib::{current_ts, read_cfg, AgentPlugin};
use fs_extra::dir::get_size;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	enabled:     bool,
	periodicity: i64,
	files:       Vec<String>,
	keyphrase:   Vec<String>
}


struct FileInfo {
	last_line: i64,
	last_size: i64,
	look_for:  String
}


pub struct Plugin {
	last_call_ts:  i64,
	periodicity:   i64,
	file_info_map: HashMap<String, FileInfo>,
	enabled:       bool
}

impl Plugin {
	fn config(plugin: &mut Plugin) -> Result<(), String> {
		let cfg = read_cfg::<Config>("file_checker.yml")?;
		plugin.enabled = cfg.enabled;
		if !plugin.enabled {
			return Ok(());
		}
		plugin.periodicity = cfg.periodicity;

		for i in 0..cfg.files.len() {
			let fp = File::open(&cfg.files[i]).map_err(|e| e.to_string())?;

			let nr_lines = BufReader::new(fp).lines().count() as i64;

			let file_size = get_size(&cfg.files[i]).map_err(|e| e.to_string())? as i64;

			plugin.file_info_map.insert(
				cfg.files[i].clone(),
				FileInfo {
					last_line: nr_lines,
					last_size: file_size,
					look_for:  cfg.keyphrase[i].clone()
				}
			);
		}
		return Ok(())
	}
}

pub fn new() -> Result<Plugin, String> {
	let mut new_plugin = Plugin {
		enabled:       false,
		last_call_ts:  0,
		periodicity:   0,
		file_info_map: HashMap::new()
	};

	Plugin::config(&mut new_plugin)?;

	if new_plugin.enabled {
		Ok(new_plugin)
	} else {
		Err("File checker disabled".into())
	}
}

impl AgentPlugin for Plugin {
	fn name(&self) -> String {
		String::from("File checker")
	}

	fn gather(&mut self) -> Result<String, String> {
		self.last_call_ts = current_ts();

		let mut results = Vec::new();

		let mut new_file_info_arr = Vec::new();

		for (file_name, file_info) in &self.file_info_map {
			let size = get_size(&file_name).map_err(|e| e.to_string())? as i64;

			if size != file_info.last_size {
				let fp = File::open(&file_name).map_err(|e| e.to_string())?;

				let mut line_nr = 0;

				for line_res in BufReader::new(fp).lines() {
					let line = line_res.map_err(|e| e.to_string())?;
					line_nr += 1;
					if line_nr > file_info.last_line && line.contains(&file_info.look_for) {
						results.push((file_name.clone(), format!("{}: {}", line_nr, line)));
					}
				}

				let new_file_info = FileInfo {
					last_line: line_nr,
					last_size: size as i64,
					look_for:  file_info.look_for.clone()
				};

				new_file_info_arr.push((file_name.clone(), new_file_info));
			}
		}

		for t in new_file_info_arr {
			self.file_info_map.insert(t.0, t.1);
		}

		if !results.is_empty() {
			Ok(serde_json::to_string(&results).map_err(|e| e.to_string())?)
		} else {
			Err(String::from("Nothing to read"))
		}
	}

	fn ready(&self) -> bool {
		if !self.enabled {
			return false;
		}

		self.last_call_ts + self.periodicity < current_ts()
	}
}
