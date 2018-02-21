use std::fs::copy;
use std::io::prelude::*;
use std::fs::File;
use std::string::String;


fn main() {
    let common_files = vec!["status.rs"];
    for file in common_files {
        copy(["../", file].join(""), file).unwrap();
    }

    let common_files = vec!["system_monitor.rs"];
    for file in &common_files {
        copy(["../agent_plugins/", file].join(""), ["plugins/", file].join("")).unwrap();
    }

    let mut buffer = File::create("plugins/mod.rs").unwrap();
    let plugin_mod_vec: Vec<String> = common_files.iter().map(|s| s.replace(".rs", ";")).map(|s| String::from("\npub mod ") + &s).collect();
    buffer.write_all(plugin_mod_vec.join("").as_bytes()).unwrap();
}
