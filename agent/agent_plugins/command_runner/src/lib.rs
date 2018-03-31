/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/
extern crate agent_lib;
extern crate shared_lib;
extern crate serde_json;
use agent_lib::AgentPlugin;
use shared_lib::{get_yml_config, current_ts};

use std::collections::HashMap;
use std::process::Command;

pub struct Plugin {
    last_call_map: HashMap<String, i64>,
    periodicity_map: HashMap<String, i64>,
    commands: Vec<Vec<String>>,
    enabled: bool,
}

impl Plugin {
    fn config(plugin: &mut Plugin) {
        let config = get_yml_config(&format!("command_runner.yml")).unwrap();
        if config["enabled"].as_bool().unwrap_or(false) {
            plugin.enabled = true;
        } else {
            plugin.enabled = false;
            return;
        }
        plugin.commands = config["commands"]
            .as_vec()
            .expect("Can't read commands vector")
            .iter()
            .map(|x| {
                x.as_vec()
                    .expect("Can't read command")
                    .iter()
                    .map(|x| String::from(x.as_str().expect("Can't read command element")))
                    .collect()
            })
            .collect();

        let periodicity_arr: Vec<i64> = config["periodicity_arr"]
            .as_vec()
            .expect("Can't read periodicity vector")
            .iter()
            .map(|x| x.as_i64().expect("Can't read periodicity"))
            .collect();

        for i in 0..plugin.commands.len() {
            let command_name = plugin.commands[i].join(" ");
            plugin
                .periodicity_map
                .insert(command_name.clone(), periodicity_arr[i]);
            plugin.last_call_map.insert(command_name, 0);
        }
    }
}

pub fn new() -> Result<Plugin, String> {
    let mut new_plugin = Plugin {
        enabled: false,
        last_call_map: HashMap::new(),
        periodicity_map: HashMap::new(),
        commands: Vec::new(),
    };
    Plugin::config(&mut new_plugin);
    if new_plugin.enabled {
        Ok(new_plugin)
    } else {
        Err("Command runner disabled".into())
    }
}

impl AgentPlugin for Plugin {
    fn name(&self) -> String {
        String::from("Command runner")
    }

    fn gather(&mut self) -> Result<String, String> {
        let mut results = HashMap::new();
        for command in &self.commands {
            let command_name = command.join(" ");
            self.last_call_map
                .insert(command_name.clone(), current_ts());

            let mut cmd = Command::new(&command[0]);
            if command.len() > 1 {
                cmd.args(&command[1..command.len()]);
            }

            let output = cmd.output().unwrap();

            let str_output = if output.status.success() {
                String::from_utf8(output.stdout).unwrap()
            } else {
                String::from_utf8(output.stderr).unwrap()
            };

            results.insert(command_name, str_output);
        }

        Ok(serde_json::to_string(&results).expect("Can't serialize command result map"))
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
