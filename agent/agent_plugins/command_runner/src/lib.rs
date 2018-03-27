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
    commands: Vec<Vec<String>>,
    disable: bool,
}

impl Plugin {
    fn config(plugin: &mut Plugin) {
        let config = utils::get_yml_config(&format!("command_runner.yml"));
        if config["disable"].as_bool().unwrap_or(false) {
            plugin.disable = true;
            return;
        } else {
            plugin.disable = false;
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

pub fn new() -> Plugin {
    let mut new_plugin = Plugin {
        disable: false,
        last_call_map: HashMap::new(),
        periodicity_map: HashMap::new(),
        commands: Vec::new(),
    };
    Plugin::config(&mut new_plugin);
    new_plugin
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
                .insert(command_name.clone(), utils::current_ts());

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
        if self.disable {
            return false;
        }
        for (name, _) in &self.last_call_map {
            if self.last_call_map.get(name).unwrap() + self.periodicity_map.get(name).unwrap()
                < utils::current_ts()
            {
                return true;
            }
        }
        false
    }
}
