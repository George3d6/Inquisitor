extern crate receptor_lib;
extern crate rusqlite;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;

use receptor_lib::{current_ts, read_cfg, ReceptorPlugin};
use rusqlite::Connection;
use std::string::String;
use std::vec::Vec;
use std::collections::HashMap;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	enabled:     bool,
	periodicity: i64,
	checks:      Vec<Vec<String>>,
	keys:        Vec<Vec<String>>
}


pub struct Plugin {
	last_call_ts: i64,
	periodicity:  i64,
	enabled:      bool,
	checks:       Vec<Vec<String>>,
	keys:         Vec<Vec<String>>
}

impl Plugin {
	fn config(&mut self) -> Result<(), String> {
		let cfg = read_cfg::<Config>("comparator.yml")?;
		self.enabled = cfg.enabled;
		if !self.enabled {
			return Ok(());
		}
		self.periodicity = cfg.periodicity;
		for check in cfg.checks {
			self.checks.push(check);
		}
		for key in cfg.keys {
			self.keys.push(key);
		}
		return Ok(());
	}
}

pub fn new() -> Result<Plugin, String> {
	let mut new_plugin = Plugin {
		enabled:      true,
		last_call_ts: current_ts(),
		periodicity:  0,
		keys:         vec![],
		checks:       vec![]
	};

	new_plugin.config()?;

	if new_plugin.enabled {
		Ok(new_plugin)
	} else {
		Err("Sync check disabled".into())
	}
}

impl ReceptorPlugin for Plugin {
	fn name(&self) -> String {
		String::from("Comparator")
	}

	fn gather(&mut self, db_conn: &Connection) -> Result<String, String> {
		let mut results: Vec<HashMap<String, String>> = Vec::new();

		for z in 0..self.keys.len() {
			let key = &self.keys[z];
			let mut raw_data = db_conn
				.prepare(
					"SELECT sender, message FROM agent_status WHERE ts_received > :ts_received AND plugin_name = :plugin_name;"
				)
				.map_err(|e| e.to_string())?;

			let raw_iter = raw_data
				.query_map_named(&[(":ts_received", &self.last_call_ts), (":plugin_name", &key[0])], |row| {
					(row.get(0), row.get(1))
				})
				.map_err(|e| e.to_string())?;

			let mut data: Vec<(String, String)> = Vec::new();
			for res in raw_iter {
				let (sender, message) = res.map_err(|e| e.to_string())?;
				data.push((sender, message));
			}

			for n in 0..data.len() {
				let message = &data[n].1;
				let sender = &data[n].0;
				let mut obj: serde_json::Value = serde_json::from_str(&message).map_err(|e| e.to_string())?;
				debug!("Original object: {:?} produced from: {}", &obj, &message);
				for i in 1..key.len() {
					let a1 = obj.clone();
					debug!("Tryinga find: '{}' in {:?}", &key[i], &a1);
					let a2  = match a1.get(&key[i]) {
						Some(v) => v,
						_ => continue
					};
					debug!("{:?} !", a2);
					obj = (*a2).clone();
				}
				debug!("Getting value from: {}", obj);
				let val  = match obj.as_str() {
					Some(v) => v,
					_ => continue
				};
				debug!("Got value: {}", val);

				let operator = &self.checks[z][0];
				let comparator = &self.checks[z][1];

				debug!("{} {} {}", val, operator, comparator);

				if operator == "<" {
					let fval = val.trim_right_matches("\n").parse::<f64>().map_err(|e| e.to_string())?;
					let fcomparator = comparator.parse::<f64>().map_err(|e| e.to_string())?;
					if fval < fcomparator {
						let mut warning: HashMap<String, String> = HashMap::new();
						warning.insert("sender".to_string(), sender.to_string());
						warning.insert("operation".to_string(), format!("{} {} {}", val, operator, comparator));
						warning.insert("key".to_string(), format!("{:?}", self.keys));
						results.push(warning);
					}
				} else if operator == ">" {
					let fval = val.trim_right_matches("\n").parse::<f64>().map_err(|e| e.to_string())?;
					let fcomparator = comparator.parse::<f64>().map_err(|e| e.to_string())?;
					if fval > fcomparator {
						let mut warning: HashMap<String, String> = HashMap::new();
						warning.insert("sender".to_string(), sender.to_string());
						warning.insert("operation".to_string(), format!("{} {} {}", val, operator, comparator));
						warning.insert("key".to_string(), format!("{:?}", self.keys));
						results.push(warning);
					}
				} else if operator == "==" || operator == "=" {
					if val == comparator {
						let mut warning: HashMap<String, String> = HashMap::new();
						warning.insert("sender".to_string(), sender.to_string());
						warning.insert("operation".to_string(), format!("{} {} {}", val, operator, comparator));
						warning.insert("key".to_string(), format!("{:?}", self.keys));
						results.push(warning);
					}
				} else if operator == "contains" {
					if val == comparator {
						let mut warning: HashMap<String, String> = HashMap::new();
						warning.insert("sender".to_string(), sender.to_string());
						warning.insert("operation".to_string(), format!("{} {} {}", val, operator, comparator));
						warning.insert("key".to_string(), format!("{:?}", self.keys));
						results.push(warning);
					}
				} else {
					return Err("Unknown operator".to_string());
				}
			}
		}
		debug!("{:?}", results);
		let mut results_map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
		results_map.insert("warnings".to_string(), results);
		let message = serde_json::to_string(&results_map).map_err(|e| e.to_string())?;
		self.last_call_ts = current_ts();
		Ok(message)
	}

	fn ready(&self) -> bool {
		if !self.enabled {
			return false;
		}
		self.last_call_ts + self.periodicity < current_ts()
	}
}
