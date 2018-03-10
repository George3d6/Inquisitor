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
}

impl Plugin {
    fn config(plugin: &mut Plugin) {
        let config = utils::get_yml_config(&format!("{}.yml",file!().replace("plugins/", "").replace(".rs", "")));
        plugin.periodicity = config["periodicity"].as_i64().expect("Can't read periodicity as i64");
    }
}

impl AgentPlugin for Plugin {

    fn new() -> Plugin {
        let mut new_plugin = Plugin{last_call_ts: 0, periodicity: 0};
        Plugin::config(&mut new_plugin);
        return new_plugin
    }

    fn name(&self) -> String {
        return String::from("Alive");
    }

    fn gather(&mut self) -> Result<String, String> {
        self.last_call_ts = utils::current_ts();
        return Ok(String::from("I live"))
    }

    fn ready(&self) -> bool {
        return self.last_call_ts + self.periodicity < utils::current_ts()
    }
}
