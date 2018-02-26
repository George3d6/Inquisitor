extern crate serde;
extern crate serde_json;
extern crate serde_derive;

extern crate sysinfo;
use self::sysinfo::{SystemExt, System, DiskExt, NetworkExt, ProcessorExt};

extern crate time;

use plugin_interface::AgentPlugin;

use std::collections::HashMap;
use std::vec::Vec;

pub struct Plugin {
    sys: System,
    last_call_ts: time::now_utc().tm_sec,
}

impl AgentPlugin for Plugin {
    fn new() -> Plugin {
        return Plugin{sys: System::new()}
    }

    fn name(&self) -> String {
        return String::from("System monitor")
    }

    fn gather(&mut self) -> Result<String, String> {
        self.sys.refresh_all();

        let mut machine_state = HashMap::new();

        let mut fs_state: Vec<String> = Vec::new();

        for disk in self.sys.get_disks() {
            let mut disk_state = HashMap::new();
            disk_state.insert("mount_point",        format!("{}",   disk.get_mount_point().to_string_lossy()));
            disk_state.insert("available_space",    format!("{}",   disk.get_available_space()));
            disk_state.insert("total_space",        format!("{}",   disk.get_total_space  ()));

            fs_state.push(serde_json::to_string(&disk_state).expect("Can't serialize disk_state"));
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

        machine_state.insert("fs_state", serde_json::to_string(&fs_state).expect("Can't serialize fs_state"));
        machine_state.insert("memory", serde_json::to_string(&memory_map).expect("Can't serialize memory_map"));
        machine_state.insert("swap", serde_json::to_string(&swap_map).expect("Can't serialize swap_map"));
        machine_state.insert("processor", serde_json::to_string(&processor_map).expect("Can't serialize swap_map"));
        machine_state.insert("network", serde_json::to_string(&network_map).expect("Can't serialize swap_map"));
        return Ok(serde_json::to_string(&machine_state).expect("Can't serialize fs_state"))
    }

    fn ready(&self) -> bool {
        return true
    }
}
