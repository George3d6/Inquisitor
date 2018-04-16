extern crate cargo_metadata;

use std::{fs::File, io::Write, path::Path};

fn main() {
	// Get list of 'dependencies'
	let packages = &cargo_metadata::metadata_deps(Some(Path::new("Cargo.toml")), true)
		.unwrap()
		.packages;

	let v = &packages.iter().find(|&x| x.name == "plugins").unwrap().dependencies;

	let mut plugins = vec![];

	for p in v {
		if p.kind == cargo_metadata::DependencyKind::Normal && p.name != "inquisitor_lib" && p.name != "env_logger"
			&& p.name != "log"
		{
			plugins.push(p.name.clone());
		}
	}

	println!("Compiling with plugins: {:?}", plugins);

	// Write to src/lib.rs
	let mut f = File::create("src/lib.rs").unwrap();

	f.write_all(
		format!(
			"extern crate inquisitor_lib;
            use inquisitor_lib::AgentPlugin;

			#[macro_use]
			extern \
			 crate log;
			extern crate env_logger;

            #[macro_use]
            mod plugin_initialization;

            \
			 plugins!({});
            ",
			plugins.join(", ")
		).as_bytes()
	).unwrap();
}
