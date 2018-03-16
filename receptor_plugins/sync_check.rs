extern crate serde_json;

use rusqlite::Connection;

use plugin_interface::ReceptorPlugin;
use utils;

use std::collections::HashMap;
use std::string::String;


pub struct Plugin {
    last_call_ts: i64,
    periodicity: i64,
    disable: bool,
}

impl Plugin {
    fn config(plugin: &mut Plugin) {
        let config = utils::get_yml_config(&format!("{}.yml",file!().replace("plugins/", "").replace(".rs", "")));

        if config["disable"].as_bool().unwrap_or(false) {
            plugin.disable = true;
            return
        } else {
            plugin.disable = false;
        }

        plugin.periodicity = config["periodicity"].as_i64().expect("Can't read periodicity as i64");
    }
}

impl ReceptorPlugin for Plugin {

    fn new() -> Plugin {
        let mut new_plugin = Plugin{disable: false, last_call_ts: 0, periodicity: 0};
        Plugin::config(&mut new_plugin);
        return new_plugin
    }

    fn name(&self) -> String {
        return String::from("Sync check");
    }

    fn gather(&mut self, db_conn: &Connection) -> Result<String, String> {
        self.last_call_ts = utils::current_ts();

        let mut raw_data = db_conn.prepare("SELECT strftime('%s', ts_received) - max(ts_sent) as diff, sender FROM agent_status GROUP BY sender;").expect("Can't select from database");


        let raw_iter = raw_data.query_map(&[], |row| {
            (row.get(1), row.get(0))
        }).expect("Problem getting sender and ts diff touple");

        let mut diff_map: HashMap<String, i64> = HashMap::new();
        for res in raw_iter {
            let (sender, val) = res.unwrap();
            diff_map.insert(sender, val);
        }


        return Ok(serde_json::to_string(&diff_map).expect("Can't serialize clock dif map"))
    }

    fn ready(&self) -> bool {
        if self.disable {
            return false
        }
        return self.last_call_ts + self.periodicity < utils::current_ts()
    }
}
