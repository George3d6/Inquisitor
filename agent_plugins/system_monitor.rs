extern crate serde;
extern crate serde_json;
extern crate serde_derive;

extern crate sysinfo;
use self::sysinfo::{ProcessExt, SystemExt, System, Disk, DiskExt};

use plugin_interface::AgentPlugin;

use std::collections::HashMap;
use std::vec::Vec;

pub struct Plugin {
    sys: System,
}

impl AgentPlugin for Plugin {
    fn new() -> Plugin {
        return Plugin{sys: System::new()};
    }

    fn gather(&mut self) -> Result<String, String> {
        self.sys.refresh_all();

        let mut machine_state = HashMap::new();

        let mut fs_state: Vec<String> = Vec::new();

        for disk in self.sys.get_disks() {
            let mut disk_state = HashMap::new();
            disk_state.insert("mount_point",        format!("{}",   disk.get_mount_point().to_string_lossy()        ));
            disk_state.insert("available_space",    format!("{}",   disk.get_available_space()                      ));
            disk_state.insert("total_space",        format!("{}",   disk.get_total_space  ()                        ));

            fs_state.push(serde_json::to_string(&disk_state).expect("Can't serialize disk_state"));
        }

        machine_state.insert("fs_state", serde_json::to_string(&fs_state).expect("Can't serialize fs_state"));
        return Ok(serde_json::to_string(&machine_state).expect("Can't serialize fs_state"))
    }

    fn ready(&self) -> bool {
        return true
    }
}
