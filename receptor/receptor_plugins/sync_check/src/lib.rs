extern crate receptor_lib;
extern crate rusqlite;
extern crate serde_json;
extern crate shared_lib;

use receptor_lib::ReceptorPlugin;
use receptor_lib::utils;
use rusqlite::Connection;
use shared_lib::current_ts;
use shared_lib::get_yml_config;

use std::collections::HashMap;
use std::string::String;

pub struct Plugin {
    last_call_ts: i64,
    periodicity: i64,
    enabled: bool,
}

impl Plugin {
    fn config(plugin: &mut Plugin) -> Result<(), String> {
        let config = match get_yml_config("sync_check.yml") {
            Ok(config) => config,
            Err(err) => return Err(err),
        };

        if config["enabled"].as_bool().unwrap_or(false) {
            plugin.enabled = true;
            return Ok(());
        } else {
            plugin.enabled = false;
        }

        plugin.periodicity = match config["periodicity"].as_i64() {
            Some(val) => val,
            _ => return Err("Can't properly read key periodicity !".to_string()),
        };
        return Ok(());
    }
}

pub fn new() -> Result<Plugin, String> {
    let mut new_plugin = Plugin {
        enabled: true,
        last_call_ts: 0,
        periodicity: 0,
    };
    let error = Plugin::config(&mut new_plugin);
    match error {
        Ok(()) => return Ok(new_plugin),
        Err(err) => return Err(err),
    };
}

impl ReceptorPlugin for Plugin {
    fn name(&self) -> String {
        String::from("Sync check")
    }

    fn gather(&mut self, db_conn: &Connection) -> Result<String, String> {
        self.last_call_ts = current_ts();

        let mut raw_data = db_conn.prepare("SELECT strftime('%s', ts_received) - max(ts_sent) as diff, sender FROM agent_status GROUP BY sender;").expect("Can't select from database");

        let raw_iter = raw_data
            .query_map(&[], |row| (row.get(1), row.get(0)))
            .expect("Problem getting sender and ts diff touple");

        let mut diff_map: HashMap<String, i64> = HashMap::new();
        for res in raw_iter {
            let (sender, val) = res.unwrap();
            diff_map.insert(sender, val);
        }

        Ok(serde_json::to_string(&diff_map).expect("Can't serialize clock dif map"))
    }

    fn ready(&self) -> bool {
        if !self.enabled {
            return false;
        }
        self.last_call_ts + self.periodicity < current_ts()
    }
}
