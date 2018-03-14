/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/
extern crate serde_json;

use plugin_interface::AgentPlugin;
use utils;

extern crate fs_extra;
use self::fs_extra::dir::get_size;

use std::vec::Vec;
use std::collections::HashMap;
use std::string::String;
use std::io::{BufReader,BufRead};
use std::fs::File;


struct FileInfo {
    last_line   : i64,
    last_size   : i64,
    look_for    : String,
}


pub struct Plugin {
    last_call_ts: i64,
    periodicity: i64,
    file_info_map: HashMap<String, FileInfo>,
}

impl Plugin {
    fn config(plugin: &mut Plugin) {
        let config = utils::get_yml_config(&format!("{}.yml",file!().replace("plugins/", "").replace(".rs", "")));

        let keyphrase: Vec<String> = config["keyphrase"].as_vec().expect("Can't read commands vector")
        .iter().map(|x| String::from(x.as_str().expect("Can't read command element"))).collect();

        let files: Vec<String> = config["files"].as_vec().expect("Can't read commands vector")
        .iter().map(|x| String::from(x.as_str().expect("Can't read command element"))).collect();

        for i in 0..files.len() {

            let fp = File::open(&files[i]).expect("Can't open file");
            let nr_lines = BufReader::new(fp).lines().count() as i64;
            let file_size = get_size(&files[i]).expect("Can't get file size !") as i64;

            plugin.file_info_map.insert(files[i].clone(), FileInfo{last_line: nr_lines, last_size: file_size, look_for: keyphrase[i].clone()});
        }

        plugin.periodicity = config["periodicity"].as_i64().expect("Can't read periodicity as i64");
    }
}

impl AgentPlugin for Plugin {

    fn new() -> Plugin {
        let mut new_plugin = Plugin{last_call_ts: 0, periodicity: 0, file_info_map: HashMap::new()};
        Plugin::config(&mut new_plugin);
        return new_plugin
    }

    fn name(&self) -> String {
        return String::from("File checker");
    }

    fn gather(&mut self) -> Result<String, String> {
        self.last_call_ts = utils::current_ts();
        
        let mut results = Vec::new();
        let mut new_file_info_arr = Vec::new();

        for (file_name, file_info) in &self.file_info_map {
            let size = get_size(&file_name).expect("Can't get file size !") as i64;
            if size != file_info.last_size {
                let fp = File::open(&file_name).expect("Can't open file");
                let mut line_nr = 0;
                for line_res in BufReader::new(fp).lines() {
                    let line = line_res.unwrap();
                    line_nr += 1;
                    if line_nr > file_info.last_line {
                        if line.contains(&file_info.look_for) {
                            results.push((file_name.clone(), format!("{}: {}", line_nr, line)));
                        }
                    }
                }
                let new_file_info = FileInfo{last_line: line_nr, last_size: size as i64, look_for: file_info.look_for.clone()};
                new_file_info_arr.push((file_name.clone(), new_file_info));
            }
        }

        for t in new_file_info_arr {
            self.file_info_map.insert(t.0, t.1);
        }


        if results.len() > 0 {
            return Ok(serde_json::to_string(&results).expect("Can't serialize command result map"))
        } else {
            return Err(String::from("Nothing to read"))
        }
    }

    fn ready(&self) -> bool {
        return self.last_call_ts + self.periodicity < utils::current_ts()
    }
}
