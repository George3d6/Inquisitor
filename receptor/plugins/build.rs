extern crate cargo_metadata;

use std::{fs::File, io::Write};

fn main() {
	// Get list of 'dependencies'
	let packages = &cargo_metadata::metadata_deps(None, true)
		.unwrap()
		.packages;

	let v = &packages.iter().find(|&x| x.name == "receptor_plugins").unwrap().dependencies;

	let mut plugins = vec![];

	for p in v {
		if p.kind == cargo_metadata::DependencyKind::Normal && p.name != "inquisitor_lib" && p.name != "env_logger"
			&& p.name != "log"
		{
			plugins.push(p.name.clone());
		}
	}

	println!("{:?}", plugins);

	// Write to src/lib.rs
	let mut f = File::create("src/lib.rs").unwrap();

	f.write_all(
		format!(
			"extern crate inquisitor_lib;
            use inquisitor_lib::ReceptorPlugin;

            #[macro_use] \
			 extern crate log;
            extern crate env_logger;

            #[macro_use]
            mod \
			 plugin_initialization;

            plugins!({});
            ",
			plugins.join(", ")
		).as_bytes()
	).unwrap();
}
