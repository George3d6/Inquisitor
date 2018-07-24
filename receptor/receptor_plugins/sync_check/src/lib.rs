extern crate inquisitor_lib;
extern crate rusqlite;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use inquisitor_lib::{current_ts, read_cfg, ReceptorPlugin};
use rusqlite::Connection;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	enabled:     bool,
	periodicity: i64
}


pub struct Plugin {
	last_call_ts: i64,
	periodicity:  i64,
	enabled:      bool,
	cfg_path:     PathBuf
}

impl Plugin {
	fn config(&mut self) -> Result<(), String> {
		let cfg = read_cfg::<Config>(&self.cfg_path)?;
		self.enabled = cfg.enabled;
		if self.enabled {
			self.periodicity = cfg.periodicity;
		}
		Ok(())
	}
}

impl ReceptorPlugin for Plugin {
	fn new(mut cfg_path: PathBuf) -> Result<Plugin, String> {
		cfg_path.push("sync_check.yml");
		let mut new_plugin = Plugin {
			enabled: false,
			last_call_ts: 0,
			periodicity: 0,
			cfg_path
		};

		new_plugin.config()?;

		if new_plugin.enabled {
			Ok(new_plugin)
		} else {
			Err("Sync check disabled".into())
		}
	}

	fn name(&self) -> &'static str {
		"Sync check"
	}

	fn gather(&mut self, db_conn: &Connection) -> Result<String, String> {
		self.last_call_ts = current_ts();

		let mut raw_data = db_conn
			.prepare(
				"SELECT strftime('%s', ts_received) - max(ts_sent) as diff, sender FROM agent_status GROUP BY sender;"
			)
			.map_err(|e| e.to_string())?;

		let raw_iter = raw_data
			.query_map(&[], |row| (row.get(1), row.get(0)))
			.map_err(|e| e.to_string())?;

		let mut diff_map: HashMap<String, i64> = HashMap::new();

		for res in raw_iter {
			let (sender, val) = res.map_err(|e| e.to_string())?;

			diff_map.insert(sender, val);
		}

		let message = serde_json::to_string(&diff_map).map_err(|e| e.to_string())?;

		Ok(message)
	}

	fn ready(&self) -> bool {
		if !self.enabled {
			return false;
		}

		self.last_call_ts + self.periodicity < current_ts()
	}
}
