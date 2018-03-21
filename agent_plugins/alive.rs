/*
    This plugin is simply used to make sure the agent is running.

    It's information can also be used by other plugins in order to determine machine reliability
    or find machines with unsynchronized clocks
*/
use plugin_interface::AgentPlugin;
use utils;


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

impl AgentPlugin for Plugin {

    fn new() -> Plugin {
        let mut new_plugin = Plugin{disable: false, last_call_ts: 0, periodicity: 0};
        Plugin::config(&mut new_plugin);
        new_plugin
    }

    fn name(&self) -> String {
        String::from("Alive")
    }

    fn gather(&mut self) -> Result<String, String> {
        self.last_call_ts = utils::current_ts();
        Ok(String::from("I live"))
    }

    fn ready(&self) -> bool {
        if self.disable {
            return false
        }
        self.last_call_ts + self.periodicity < utils::current_ts()
    }
}
