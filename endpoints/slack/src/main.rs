extern crate inquisitor_lib;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::{thread, time};
use std::vec::Vec;
use inquisitor_lib::{current_ts, read_cfg};
use std::collections::HashMap;
use std::cmp;
use std::env::current_exe;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Check {
	plugin: String,
	sender: String,
	level:  String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Receptor {
	host: String,
	port: u32
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
	receptor: Receptor,
	monitor:  Vec<Check>,
	endpoint: String
}


fn process_row(row: &str) -> (String, i64) {
	let vals: Vec<&str> = row.split('\t').collect();
	info!("{:?}", vals);
	let ts = vals[1].parse::<i64>().map_err(|e| e.to_string()).unwrap();
	let message = vals[0];
	(message.to_string(), ts)
}


fn main() {
	env_logger::init();

	let client = reqwest::Client::new();

	let mut config_path_buff = current_exe().unwrap();
	config_path_buff.pop();
	config_path_buff.push("config.yml");

	let cfg = read_cfg::<Config>(&config_path_buff).expect("Can't find config.yml file");
	let receptor_uri_base = format!("{}:{}", cfg.receptor.host, cfg.receptor.port);

	let my_endpoint = cfg.endpoint;

	let slack_uri = format!("https://hooks.slack.com/services/{}", my_endpoint);

	let mut ts_collect = current_ts();

	loop {
		thread::sleep(time::Duration::from_millis(1000));
		for check in &cfg.monitor {
			let mut res = client
				.get(&format!(
					"{}/plugin_data?level={}&name={}&ts_start={}&ts_end=9923146529",
					receptor_uri_base, &check.level, &check.plugin, &ts_collect
				))
				.send()
				.unwrap();
			let text = res.text().unwrap();
			let rows: Vec<(String, i64)> = text.split('\n')
				.map(|x| x.to_string())
				.filter(|x| x.len() > 1)
				.map(|x| process_row(&x))
				.collect();
			ts_collect = cmp::max(ts_collect, rows.iter().map(|x| x.1).fold(0i64, cmp::max));
			debug!("Collecting starting from timestamp: {} !", ts_collect);
			for r in rows {
				let mut form = HashMap::new();
				form.insert("text", r.0);
				let slack_reponse = client.post(&slack_uri).json(&form).send().unwrap();
				debug!("{:?}", slack_reponse);
			}
		}
	}
}
