use std::time::{SystemTime, UNIX_EPOCH};
use std::env::current_exe;
use std::fs::File;
use std::io::prelude::*;

extern crate yaml_rust;
use self::yaml_rust::{YamlLoader, Yaml};


pub fn current_ts() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).expect("Error getting system time !?").as_secs() as i64
}

pub fn get_yml_config(name: &str) -> Yaml {
    let mut cfg_file_path = current_exe().unwrap();
    cfg_file_path.pop();
    cfg_file_path.push(name);
    let mut cfg_file = File::open(cfg_file_path).expect("Can't find configuration file system_monitor.yml");
    let mut contents = String::new();
    cfg_file.read_to_string(&mut contents).expect("something went wrong reading the file system_monitor.yml");

    let mut docs = YamlLoader::load_from_str(&contents).unwrap();
    return docs.remove(0);
}
