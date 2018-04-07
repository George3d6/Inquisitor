extern crate receptor_lib;
extern crate rusqlite;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use receptor_lib::{current_ts, read_cfg, ReceptorPlugin};
use rusqlite::Connection;
use std::string::String;
use std::vec::Vec;


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
		last_call_ts: current_ts() - 1523092463,
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
		let mut results: Vec<String> = Vec::new();

		for key in &self.keys {
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

			for d in data {
				let mut obj = json!(d.1);
				println!("Original object: {:?} produced from: {}", obj, d.1);
				for i in 1..key.len() {
					let a1 = obj.clone();
					println!("Tryinga find: '{}' in {:?}", &key[i], &a1);
					let a2  = match a1.get(&key[i]) {
						Some(v) => v,
						_ => continue
					};
					println!("{:?} !", a2);
					obj = json!((*a2).clone());
				}
				let a  = match obj.as_str() {
					Some(v) => v,
					_ => return Err(String::from("Can't properly parse value !"))
				};
			}
		}

		let message = serde_json::to_string(&results).map_err(|e| e.to_string())?;

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
