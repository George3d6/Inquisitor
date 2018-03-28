/*
    This plugin is used to periodically execute a series of remote commands and return the output
*/
extern crate agent_lib;
extern crate serde_json;
use agent_lib::AgentPlugin;
use agent_lib::utils;

extern crate fs_extra;
use self::fs_extra::dir::get_size;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct FileInfo {
    last_line: i64,
    last_size: i64,
    look_for: String,
}

pub struct Plugin {
    last_call_ts: i64,
    periodicity: i64,
    file_info_map: HashMap<String, FileInfo>,
    disable: bool,
}

impl Plugin {
    fn config(plugin: &mut Plugin) {
        let config = utils::get_yml_config("file_checker.yml");

        if config["disable"].as_bool().unwrap_or(false) {
            plugin.disable = true;
            return;
        } else {
            plugin.disable = false;
        }

        let keyphrase: Vec<String> = config["keyphrase"]
            .as_vec()
            .expect("Can't read commands vector")
            .iter()
            .map(|x| String::from(x.as_str().expect("Can't read command element")))
            .collect();

        let files: Vec<String> = config["files"]
            .as_vec()
            .expect("Can't read commands vector")
            .iter()
            .map(|x| String::from(x.as_str().expect("Can't read command element")))
            .collect();

        for i in 0..files.len() {
            let fp = File::open(&files[i]).expect("Can't open file");
            let nr_lines = BufReader::new(fp).lines().count() as i64;
            let file_size = get_size(&files[i]).expect("Can't get file size !") as i64;

            plugin.file_info_map.insert(
                files[i].clone(),
                FileInfo {
                    last_line: nr_lines,
                    last_size: file_size,
                    look_for: keyphrase[i].clone(),
                },
            );
        }

        plugin.periodicity = config["periodicity"]
            .as_i64()
            .expect("Can't read periodicity as i64");
    }
}

pub fn new() -> Plugin {
    let mut new_plugin = Plugin {
        disable: false,
        last_call_ts: 0,
        periodicity: 0,
        file_info_map: HashMap::new(),
    };
    Plugin::config(&mut new_plugin);
    new_plugin
}

impl AgentPlugin for Plugin {
    fn name(&self) -> String {
        String::from("File checker")
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
                    if line_nr > file_info.last_line && line.contains(&file_info.look_for) {
                        results.push((file_name.clone(), format!("{}: {}", line_nr, line)));
                    }
                }
                let new_file_info = FileInfo {
                    last_line: line_nr,
                    last_size: size as i64,
                    look_for: file_info.look_for.clone(),
                };
                new_file_info_arr.push((file_name.clone(), new_file_info));
            }
        }

        for t in new_file_info_arr {
            self.file_info_map.insert(t.0, t.1);
        }

        if !results.is_empty() {
            Ok(serde_json::to_string(&results).expect("Can't serialize command result map"))
        } else {
            Err(String::from("Nothing to read"))
        }
    }

    fn ready(&self) -> bool {
        if self.disable {
            return false;
        }
        self.last_call_ts + self.periodicity < utils::current_ts()
    }

    fn when_ready(&self) -> i64 {
        if self.disable {
            return 999;
        }
        utils::current_ts() - self.last_call_ts + self.periodicity
    }
}
