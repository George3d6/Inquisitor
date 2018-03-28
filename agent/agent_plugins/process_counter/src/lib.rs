/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/
extern crate agent_lib;
extern crate serde_json;
use agent_lib::AgentPlugin;
use agent_lib::utils;

use std::collections::HashMap;
use std::process::Command;

pub struct Plugin {
    last_call_map: HashMap<String, i64>,
    periodicity_map: HashMap<String, i64>,
    processes: Vec<String>,
    disable: bool,
}

impl Plugin {
    fn config(plugin: &mut Plugin) {
        let config = utils::get_yml_config(&format!("process_counter.yml"));

        if config["disable"].as_bool().unwrap_or(false) {
            plugin.disable = true;
            return;
        } else {
            plugin.disable = false;
        }

        plugin.processes = config["processes"]
            .as_vec()
            .expect("Can't read commands vector")
            .iter()
            .map(|x| String::from(x.as_str().expect("Can't read command element")))
            .collect();

        let periodicity_arr: Vec<i64> = config["periodicity_arr"]
            .as_vec()
            .expect("Can't read periodicity vector")
            .iter()
            .map(|x| x.as_i64().expect("Can't read periodicity"))
            .collect();

        for i in 0..plugin.processes.len() {
            plugin
                .periodicity_map
                .insert(plugin.processes[i].clone(), periodicity_arr[i]);
            plugin.last_call_map.insert(plugin.processes[i].clone(), 0);
        }
    }
}

pub fn new() -> Plugin {
    let mut new_plugin = Plugin {
        disable: false,
        last_call_map: HashMap::new(),
        periodicity_map: HashMap::new(),
        processes: Vec::new(),
    };
    Plugin::config(&mut new_plugin);
    new_plugin
}

impl AgentPlugin for Plugin {
    fn name(&self) -> String {
        String::from("Process counter")
    }

    fn gather(&mut self) -> Result<String, String> {
        let mut results = HashMap::new();
        for process in &self.processes {
            self.last_call_map
                .insert(process.clone(), utils::current_ts());

            let mut cmd = Command::new("pgrep");
            cmd.arg("-f");
            cmd.arg(&process);

            let output = cmd.output().unwrap();

            let mut running: i64 = 0;

            if output.status.success() {
                let str_output = String::from_utf8(output.stdout).unwrap();
                if !str_output.is_empty() {
                    let v: Vec<&str> = str_output.split('\n').filter(|&x| x != "").collect();
                    running = v.len() as i64;
                }
            }

            results.insert(process, running);
        }

        Ok(serde_json::to_string(&results).expect("Can't serialize command result map"))
    }

    fn ready(&self) -> bool {
        if self.disable {
            return false;
        }
        self.last_call_map
            .iter()
            .any(|(k, v)| v + self.periodicity_map[k] < utils::current_ts())
    }
}
