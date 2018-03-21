extern crate serde_json;

extern crate sysinfo;
use self::sysinfo::{SystemExt, System, DiskExt, NetworkExt, ProcessorExt};

use plugin_interface::AgentPlugin;
use utils;

use std::collections::HashMap;
use std::vec::Vec;


pub struct Plugin {
    sys: System,
    last_call_ts: i64,
    periodicity: i64,
    disable: bool,
}

#[derive(Serialize, Debug)]
struct MachineState<'a> {
    fs_state: Vec<HashMap<&'a str, String>>,
    memory_map: HashMap<&'a str, String>,
    swap_map: HashMap<&'a str, String>,
    processor_map: HashMap<&'a str, f32>,
    network_map: HashMap<&'a str, String>,
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

        plugin.periodicity = config["periodicity"].as_i64().expect("Can't read periodicity as i64")
    }
}

impl AgentPlugin for Plugin {

    fn new() -> Plugin {
        let mut new_plugin = Plugin{disable: false, sys: System::new(), last_call_ts: 0, periodicity: 0};
        Plugin::config(&mut new_plugin);
        new_plugin
    }

    fn name(&self) -> String {
        String::from("System monitor")
    }

    fn gather(&mut self) -> Result<String, String> {
        self.last_call_ts = utils::current_ts();

        self.sys.refresh_all();

        let mut fs_state: Vec<HashMap<&str, String>> = Vec::new();

        for disk in self.sys.get_disks() {
            let mut disk_state = HashMap::new();
            disk_state.insert("mount_point",        format!("{}",   disk.get_mount_point().to_string_lossy()));
            disk_state.insert("available_space",    format!("{}",   disk.get_available_space()));
            disk_state.insert("total_space",        format!("{}",   disk.get_total_space  ()));

            fs_state.push(disk_state);
        }

        let mut memory_map = HashMap::new();
        memory_map.insert("total_memory",   format!("{}",       self.sys.get_total_memory()));
        memory_map.insert("used_memory",    format!("{}",       self.sys.get_used_memory()));

        let mut swap_map = HashMap::new();
        swap_map.insert("total_swap",   format!("{}",       self.sys.get_total_swap()));
        swap_map.insert("used_swap",    format!("{}",       self.sys.get_used_swap()));

        let processors = self.sys.get_processor_list();
        let mut processor_map = HashMap::new();
        let total_usage: f32 = processors.iter().fold(0f32, |sum, val| sum + val.get_cpu_usage())/(processors.len() as f32);
        processor_map.insert("total_usage", total_usage);

        let mut network_map = HashMap::new();
        let network = self.sys.get_network();
        network_map.insert("in",  format!("{}", network.get_income()));
        network_map.insert("out", format!("{}", network.get_outcome()));

        let machine_state = MachineState{fs_state: fs_state,memory_map: memory_map, swap_map: swap_map,processor_map: processor_map, network_map: network_map};
        Ok(serde_json::to_string(&machine_state).expect("Can't serialize fs_state"))
    }

    fn ready(&self) -> bool {
        if self.disable {
            return false
        }
        self.last_call_ts + self.periodicity < utils::current_ts()
    }
}
