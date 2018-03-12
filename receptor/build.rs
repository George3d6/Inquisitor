extern crate fs_extra;

use std::fs::read_dir;
use std::fs::create_dir_all;
use std::io::prelude::*;
use std::fs::File;
use std::string::String;


fn main() {

    create_dir_all("plugins").expect("Can't create plugin director !?");
    create_dir_all("target/debug").expect("Can't create plugin director !?");
    create_dir_all("target/release").expect("Can't create plugin director !?");

    /* Overly fucking pednatic libraries force us to use different options for copying a single file -_- */

    let common_files = vec!["status.rs"];
    for file in common_files {
        fs_extra::file::copy(["../", file].join(""), file, &fs_extra::file::CopyOptions{overwrite: true, skip_exist: false, buffer_size: 64000}).unwrap();
    }

    let paths = read_dir("../receptor_plugins/").unwrap();

    let mut common_files: Vec<String> = Vec::new();
    for path in paths {
        let p = path.unwrap().path();
        let s = p.components().last().unwrap().as_os_str().to_str().unwrap();

        common_files.push(String::from(s));
    }

    let rust_files: Vec<String> = common_files.iter().filter(|s| s.contains(".rs")).map(|s| s.clone()).collect();
    for file in &rust_files {
        fs_extra::file::copy([String::from("../receptor_plugins/"), file.clone()].join(""), [String::from("plugins/"), file.clone()].join(""), &fs_extra::file::CopyOptions{overwrite: true, skip_exist: false, buffer_size: 64000}).unwrap();
    }

    let aux_files: Vec<String> = common_files.iter().filter(|s| !s.contains(".rs")).map(|s| s.clone()).collect();

    for file in aux_files {
        for dest in vec!["target/debug/", "target/release/"] {
            fs_extra::file::copy([String::from("../receptor_plugins/"), file.clone()].join(""), [dest, &file].join(""), &fs_extra::file::CopyOptions{overwrite: true, skip_exist: false, buffer_size: 64000}).unwrap();
            fs_extra::dir::copy("../web_ui", dest, &fs_extra::dir::CopyOptions{overwrite: true, skip_exist: false, buffer_size: 64000, copy_inside: true, depth: 9999}).unwrap();
        }
    }



    let mut agent_file = File::open("receptor.rs").expect("Can't find receptor.rs");
    let mut agent_contents = String::new();
    agent_file.read_to_string(&mut agent_contents).expect("something went wrong reading the file receptor.rs");

    let create_plugins_vec: Vec<String> = rust_files.iter().map(|s| s.replace(".rs", "")).map(|s| format!("let mut {plugin_name} = plugins::{plugin_name}::Plugin::new();", plugin_name=s)).collect();
    agent_contents = agent_contents.replace("{{CREATE_PLUGINS}}", &create_plugins_vec.join("\n      "));

    let use_plugins_vec: Vec<String> = rust_files.iter().map(|s| s.replace(".rs", "")).map(|s| format!("plugin_runner.run_plugin(&mut {plugin_name});", plugin_name=s)).collect();
    agent_contents = agent_contents.replace("{{USE_PLUGINS}}", &use_plugins_vec.join("\n        "));

    File::create("receptor_processed.rs").unwrap().write_all(agent_contents.as_bytes()).unwrap();



    let plugin_mod_vec: Vec<String> = rust_files.iter().map(|s| s.replace(".rs", ";")).map(|s| String::from("\npub mod ") + &s).collect();
    let mut plugin_mod_file = File::create("plugins/mod.rs").unwrap();
    plugin_mod_file.write_all(plugin_mod_vec.join("").as_bytes()).unwrap();
}
