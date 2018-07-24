mod database;

extern crate env_logger;
extern crate fs_extra;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate clap;
extern crate inquisitor_lib;
extern crate plugins;
extern crate rusqlite;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate tokio;
extern crate tokio_core;
extern crate url;

use clap::{App, Arg};
use database::{get_connection, initialize_database};
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use inquisitor_lib::{read_cfg, ReceptorPlugin, Status};
use rusqlite::Connection;
use std::collections::HashMap;
use std::env::current_exe;
use std::path::PathBuf;
use std::{thread, time};
use tokio::io::AsyncRead;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::{future, Future, Stream};
use tokio_core::reactor::Core;

#[derive(Debug, PartialEq, Deserialize)]
struct ConfigServerAddr {
	bind: String,
	port: i64
}

#[derive(Debug, PartialEq, Deserialize)]
struct Config {
	clean_older_than: i64,
	server:           ConfigServerAddr,
	receptor:         ConfigServerAddr
}

struct DataServer {
	pub db_conn: Connection
}

impl Service for DataServer {
	type Request = Request;

	type Response = Response;

	type Error = hyper::Error;

	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Request) -> Self::Future {
		if req.path() == "/plugin_data" {
			let mut response = Response::new();

			match (req.method(), req.path()) {
				(&Method::Get, "/plugin_data") => {
					let params = get_url_params(&req);

					let plugin_name = params.get("name").map(|s| s.as_ref()).unwrap_or("");

					let level = params.get("level").map(|s| s.as_ref()).unwrap_or("");

					let ts_start = params.get("ts_start").map(|s| s.as_ref()).unwrap_or("-1");

					let ts_end = params.get("ts_end").map(|s| s.as_ref()).unwrap_or("-1");

					if level == "agent" {
						let mut raw_data = self.db_conn
							.prepare(
								"SELECT * FROM agent_status WHERE strftime('%s',ts_received) > :ts_start AND \
								 strftime('%s',ts_received) < :ts_end AND plugin_name=:plugin_name"
							)
							.expect("Can't select from database");

						let raw_status_iter = raw_data
							.query_map_named(
								&[
									(":ts_start", &ts_start),
									(":ts_end", &ts_end),
									(":plugin_name", &plugin_name)
								],
								|row| (row.get(0), row.get(1), row.get(3))
							)
							.expect("Problem getting raw status");

						let status_tsv_itter = raw_status_iter.map(|rs| {
							let (sender, message, ts): (String, String, i64) = rs.expect("Corrupt status in database");

							format!("{}\t{}\t{}", sender, message, ts)
						});

						let status_tsv_vec: Vec<String> = status_tsv_itter.collect();

						response.set_body(status_tsv_vec.join("\n"));
					} else {
						let mut raw_data = self.db_conn
							.prepare(
								"SELECT message, strftime('%s', ts) FROM receptor_status WHERE strftime('%s',ts) > \
								 :ts_start AND strftime('%s',ts) < :ts_end AND plugin_name=:plugin_name"
							)
							.expect("Can't select from database");

						let raw_status_iter = raw_data
							.query_map_named(
								&[
									(":ts_start", &ts_start),
									(":ts_end", &ts_end),
									(":plugin_name", &plugin_name)
								],
								|row| (row.get(0), row.get(1))
							)
							.expect("Problem getting receptor_status tuple");

						let status_tsv_itter = raw_status_iter.map(|rs| {
							let (message, ts): (String, String) = rs.expect("Corrupt status in database");

							format!("{}\t{}", message, ts)
						});

						let status_tsv_vec: Vec<String> = status_tsv_itter.collect();

						response.set_body(status_tsv_vec.join("\n"));
					};
				}

				_ => response.set_status(StatusCode::NotFound)
			};

			Box::new(future::ok(response))
		} else if req.path() == "/plugin_list" {
			let mut response = Response::new();

			match (req.method(), req.path()) {
				(&Method::Get, "/plugin_list") => {
					let params = get_url_params(&req);

					let level = params.get("level").map(|s| s.as_ref()).unwrap_or("");

					let mut raw_data = if level == "agent" {
						self.db_conn
							.prepare("SELECT DISTINCT(plugin_name) FROM agent_status")
							.expect("Can't select from database")
					} else {
						self.db_conn
							.prepare("SELECT DISTINCT(plugin_name) FROM receptor_status")
							.expect("Can't select from database")
					};

					let raw_status_iter = raw_data
						.query_map_named(&[], |row| row.get(0))
						.expect("Problem getting receptor_status tuple");

					let status_tsv_itter = raw_status_iter.map(|s| s.expect("Corrupt status in database"));

					let status_tsv_vec: Vec<String> = status_tsv_itter.collect();

					response.set_body(status_tsv_vec.join("\n"));
				}
				_ => response.set_status(StatusCode::NotFound)
			}

			Box::new(future::ok(response))
		} else {
			let mut response = Response::new();
			response.set_status(StatusCode::NotFound);
			Box::new(future::ok(response))
		}
	}
}

fn proces_status(stream: TcpStream, db_conn: Connection) {
	let (reader, _) = stream.split();

	let conn = tokio::io::read_to_end(reader, Vec::new()).then(move |res| {
		let payload = res.expect("Can't read input from agent").1;

		let statuses: Vec<Status> = serde_json::from_slice(&payload).expect("Can't deserialize status");

		for status in statuses {
			db_conn
				.execute(
					"INSERT INTO agent_status(sender, message, plugin_name, ts_sent)
                VALUES (?1, ?2, \
					 ?3, ?4)",
					&[
						&status.sender,
						&status.message,
						&status.plugin_name,
						&status.ts.to_string()
					]
				)
				.expect("Can't insert status into agent_status table");
		}

		Ok(())
	});

	tokio::spawn(conn);
}

struct PluginRunner {
	pub db_conn: Connection
}

impl PluginRunner {
	pub fn new() -> PluginRunner {
		PluginRunner {
			db_conn: get_connection()
		}
	}

	pub fn run_plugin(&self, plugin: &mut ReceptorPlugin) {
		if plugin.ready() {
			let message = match plugin.gather(&self.db_conn) {
				Ok(message) => message,
				Err(err) => {
					error!("Error: {} ! When running gather for plguin {}", err, plugin.name());
					return;
				}
			};

			let name = plugin.name();

			self.db_conn
				.execute(
					"INSERT INTO receptor_status(message, plugin_name)
                VALUES (?1, ?2)",
					&[&message, &name]
				)
				.expect("Can't insert recptor side plugin data into receptor_status table");
		}
	}
}

fn main() -> Result<(), String> {
	env_logger::init();

	let mut exec_path_buff = current_exe().map_err(|e| e.to_string())?;
	exec_path_buff.pop();
	let exec_path = exec_path_buff
		.to_str()
		.ok_or_else(|| "Couldn't get execution path".to_string())?;

	let matches = App::new("Inquisitor receptor")
		.version("0.3.1")
		.about(
			"The receptor component of the inquisitor monitoring suite,
					  for more infomration visit: \
			 https://github.com/George3d6/Inquisitor"
		)
		.arg(
			Arg::with_name("config_dir")
				.long("config-dir")
				.help("The directory where the receptor looks for it's configuration files")
				.default_value(&exec_path)
				.takes_value(true)
				.required(false)
		)
		.get_matches();

	// Produce config path
	let config_dir = PathBuf::from(matches.value_of("config_dir").unwrap());
	let mut receptor_config = config_dir.clone();
	receptor_config.push("receptor_config.yml");
	let config = read_cfg::<Config>(&receptor_config)?;

	let clean_older_than = config.clean_older_than;

	initialize_database();

	/* Do some administrative sutff */

	let administrator_thread = thread::spawn(move || loop {
		let db_conn = get_connection();

		db_conn
			.execute(
				"DELETE FROM agent_status WHERE CAST(strftime('%s',ts_received) as decimal)
             < \
				 (CAST(strftime('%s',CURRENT_TIMESTAMP) as decimal) - ?1)",
				&[&clean_older_than]
			)
			.expect("Can't clean up agent_status table");

		db_conn
			.execute(
				"DELETE FROM receptor_status WHERE CAST(strftime('%s',ts) as decimal)
             < \
				 (CAST(strftime('%s',CURRENT_TIMESTAMP) as decimal) - ?1)",
				&[&clean_older_than]
			)
			.expect("Can't clean up receptor_status table");

		thread::sleep(time::Duration::from_millis(360 * 1000));
	});

	/* Run receptor side plugins */

	let _plugin_runner_thread = thread::spawn(|| {
		let mut plugins = plugins::init(config_dir);

		let plugin_runner = PluginRunner::new();

		loop {
			for p in &mut plugins {
				plugin_runner.run_plugin(&mut **p);
			}

			thread::sleep(time::Duration::from_millis(1000));
		}
	});

	/* Run http server for the web UI and http endpoints to get plugin data */

	let server_addr_str = format!("{}:{}", config.server.bind, config.server.port);

	let _hyper_server_thread = thread::spawn(move || {
		let server_addr = server_addr_str
			.parse()
			.expect("Couldn't convert server addr to Socket Address. Http server will not start");

		let mut core = Core::new().unwrap();

		let handle = core.handle();

		let handle1 = handle.clone();

		let handler = handle.clone();

		let serve = Http::new()
			.serve_addr_handle(&server_addr, &handle, move || {
				Ok(DataServer {
					db_conn: get_connection()
				})
			})
			.expect("Can't start HTTP server");

		debug!("Spawning server !");
		handler.spawn(
			serve
				.for_each(move |conn| {
					handle1.spawn(conn.map(|_| ()).map_err(|err| error!("srv1 error: {:?}", err)));

					Ok(())
				})
				.map_err(|_| ())
		);

		core.run(future::empty::<(), ()>()).unwrap();
	});

	// Listen for incoming statuses from agents and process them
	// validate them & insert them into the database
	let receptor_addr_str = format!("{}:{}", config.receptor.bind, config.receptor.port);

	let listener_addr = receptor_addr_str
		.parse()
		.map_err(|_| format!("Couldn't convert receptor addr {} to Socket Address", receptor_addr_str))?;

	let listener = TcpListener::bind(&listener_addr).expect("Can't start TCP server");

	let receptor = listener
		.incoming()
		.for_each(move |stream| {
			proces_status(stream, get_connection());

			Ok(())
		})
		.map_err(|err| {
			error!("IO error {:?}", err);
		});

	tokio::run(receptor);

	administrator_thread.join().unwrap();

	_plugin_runner_thread.join().unwrap();

	_hyper_server_thread.join().unwrap();
	Ok(())
}

pub fn get_url_params(req: &Request) -> HashMap<String, String> {
	use url::Url;
	let parsed_url = Url::parse(&format!("http://example.com/{}", req.uri())).unwrap();

	let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

	hash_query
}
