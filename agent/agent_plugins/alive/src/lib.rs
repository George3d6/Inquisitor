/*
    This plugin is simply used to make sure the agent is running.

    It's information can also be used by other plugins in order to determine machine reliability
    or find machines with unsynchronized clocks
*/
extern crate agent_lib;
use agent_lib::AgentPlugin;
use agent_lib::utils;

pub struct Plugin {
    last_call_ts: i64,
    periodicity: i64,
    enabled: bool,
}

impl Plugin {
    fn config(&mut self) {
        let config = utils::get_yml_config("alive.yml");
        if config["enabled"].as_bool().unwrap_or(false) {
            self.enabled = true;
        } else {
            self.enabled = false;
            return;
        }
        self.periodicity = config["periodicity"]
            .as_i64()
            .expect("Can't read periodicity as i64");
    }
}

pub fn new() -> Result<Plugin, String> {
    let mut new_plugin = Plugin {
        enabled: false,
        last_call_ts: 0,
        periodicity: 0,
    };
    Plugin::config(&mut new_plugin);
    if new_plugin.enabled {
        Ok(new_plugin)
    } else {
        Err("Alive plugins disabled".into())
    }
}

impl AgentPlugin for Plugin {
    fn name(&self) -> String {
        String::from("Alive")
    }

    fn gather(&mut self) -> Result<String, String> {
        self.last_call_ts = utils::current_ts();
        Ok(String::from("I live"))
    }

    fn ready(&self) -> bool {
        if !self.enabled {
            return false;
        }
        self.last_call_ts + self.periodicity < utils::current_ts()
    }
}
