#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate yaml_rust;

use self::yaml_rust::{Yaml, YamlLoader};
extern crate fs_extra;
use self::fs_extra::file::read_to_string;
use std::env::current_exe;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::de::{DeserializeOwned};


#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub sender: String,
    pub ts: i64,
    pub message: String,
    pub plugin_name: String,
}


pub fn current_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting system time !?")
        .as_secs() as i64
}


pub fn get_yml_config(name: &str) -> Result<Yaml, String> {
    let mut cfg_file_path = current_exe().unwrap();
    cfg_file_path.pop();
    cfg_file_path.push(name);
    let contents = match read_to_string(&cfg_file_path) {
        Ok(content) => content,
        Err(_) => return Err(format!("Config file {} not found !", name))
    };
    let mut docs = match YamlLoader::load_from_str(&contents) {
        Ok(docs) => docs,
        Err(_) => return Err(format!("File {}, content is not valid yml !", name))
    };
    if docs.is_empty() {
        return Err(format!("No valid yml documents inside: {} !", name))
    }
    Ok(docs.remove(0))
}


pub fn get_yml_config_string(name: &str) -> Result<String, String> {
    let mut cfg_file_path = current_exe().unwrap();
    cfg_file_path.pop();
    cfg_file_path.push(name);
    return Ok(read_to_string(&cfg_file_path).map_err(|e| e.to_string())?)
}


pub fn read_cfg<ConfigT>(name: &str) -> Result<ConfigT, String> where ConfigT: DeserializeOwned {
    let cfg_str = get_yml_config_string(name)?;
    let cfg: ConfigT = serde_yaml::from_str(&cfg_str).map_err(|e| e.to_string())?;
    return Ok(cfg)
}
