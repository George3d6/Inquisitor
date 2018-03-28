extern crate yaml_rust;
use self::yaml_rust::{Yaml, YamlLoader};

extern crate fs_extra;
use self::fs_extra::file::read_to_string;

use std::env::current_exe;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting system time !?")
        .as_secs() as i64
}

pub fn get_yml_config(name: &str) -> Yaml {
    let mut cfg_file_path = current_exe().unwrap();
    cfg_file_path.pop();
    cfg_file_path.push(name);
    let contents = read_to_string(&cfg_file_path).unwrap();
    let mut docs = YamlLoader::load_from_str(&contents).unwrap();
    docs.remove(0)
}
