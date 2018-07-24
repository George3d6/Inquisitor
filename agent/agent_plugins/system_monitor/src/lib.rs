#[macro_use]
extern crate serde_derive;
extern crate inquisitor_lib;
extern crate serde_json;
extern crate sysinfo;

use inquisitor_lib::{current_ts, read_cfg, AgentPlugin};
use std::{collections::HashMap, path::PathBuf};
use sysinfo::{DiskExt, NetworkExt, ProcessorExt, System, SystemExt};


#[derive(Debug, PartialEq, Deserialize)]
struct Config {
	enabled:     bool,
	periodicity: i64
}


#[derive(Serialize, Debug)]
struct MachineState<'a> {
	fs_state:      Vec<HashMap<&'a str, String>>,
	memory_map:    HashMap<&'a str, String>,
	swap_map:      HashMap<&'a str, String>,
	processor_map: HashMap<&'a str, f32>,
	network_map:   HashMap<&'a str, String>
}


pub struct Plugin {
	sys:          System,
	last_call_ts: i64,
	periodicity:  i64,
	enabled:      bool
}

impl AgentPlugin for Plugin {
	fn new(mut cfg_path: PathBuf) -> Result<Plugin, String> {
		cfg_path.push("system_monitor.yml");
		let cfg = read_cfg::<Config>(&cfg_path)?;
		if cfg.enabled {
			let plugin = Plugin {
				enabled:      true,
				sys:          System::new(),
				last_call_ts: 0,
				periodicity:  cfg.periodicity
			};
			Ok(plugin)
		} else {
			Err("System monitor disabled".into())
		}
	}

	fn name(&self) -> &'static str {
		"System monitor"
	}

	fn gather(&mut self) -> Result<String, String> {
		self.last_call_ts = current_ts();
		self.sys.refresh_all();
		let mut fs_state: Vec<HashMap<&str, String>> = Vec::new();

		for disk in self.sys.get_disks() {
			let mut disk_state = HashMap::new();
			disk_state.insert("mount_point", format!("{}", disk.get_mount_point().to_string_lossy()));
			disk_state.insert("available_space", format!("{}", disk.get_available_space()));
			disk_state.insert("total_space", format!("{}", disk.get_total_space()));
			fs_state.push(disk_state);
		}

		let mut memory_map = HashMap::new();
		memory_map.insert("total_memory", format!("{}", self.sys.get_total_memory()));
		memory_map.insert("used_memory", format!("{}", self.sys.get_used_memory()));

		let mut swap_map = HashMap::new();
		swap_map.insert("total_swap", format!("{}", self.sys.get_total_swap()));
		swap_map.insert("used_swap", format!("{}", self.sys.get_used_swap()));

		let processors = self.sys.get_processor_list();
		let mut processor_map = HashMap::new();

		let total_usage: f32 =
			processors.iter().fold(0f32, |sum, val| sum + val.get_cpu_usage()) / (processors.len() as f32);

		processor_map.insert("total_usage", total_usage);

		let mut network_map = HashMap::new();
		let network = self.sys.get_network();
		network_map.insert("in", format!("{}", network.get_income()));
		network_map.insert("out", format!("{}", network.get_outcome()));

		let machine_state = MachineState {
			fs_state,
			memory_map,
			swap_map,
			processor_map,
			network_map
		};

		serde_json::to_string(&machine_state).map_err(|e| e.to_string())
	}

	fn ready(&self) -> bool {
		if !self.enabled {
			return false;
		}

		self.last_call_ts + self.periodicity < current_ts()
	}
}
