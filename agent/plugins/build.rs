extern crate cargo_metadata;
use std::fs::File;
use std::fs::copy;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::Path;

fn main() {
    // Get list of 'dependencies'
    let packages = &cargo_metadata::metadata_deps(Some(Path::new("Cargo.toml")), true)
        .unwrap()
        .packages;
    let v = &packages
        .iter()
        .find(|&x| x.name == "plugins")
        .unwrap()
        .dependencies;
    let mut plugins = vec![];
    for p in v {
        if p.kind == cargo_metadata::DependencyKind::Normal && p.name != "agent_lib" && p.name != "shared_lib" {
            plugins.push(p.name.clone());
        }
    }
    println!("Compiling with plugins: {:?}", plugins);

    // Write to src/lib.rs
    let mut f = File::create("src/lib.rs").unwrap();
    f.write_all(
        format!(
            "extern crate agent_lib;
            use agent_lib::AgentPlugin;

            #[macro_use]
            mod plugin_initialization;

            plugins!({});
            ",
            plugins.join(", ")
        ).as_bytes(),
    ).unwrap();



    create_dir_all("../target/debug").unwrap();
    create_dir_all("../target/release").unwrap();

    copy("../inquisitor-agent.service", "../target/debug/inquisitor-agent.service").unwrap();
    copy("../inquisitor-agent.service", "../target/release/inquisitor-agent.service").unwrap();
    copy("../agent_config.yml", "../target/debug/agent_config.yml").unwrap();
    copy("../agent_config.yml", "../target/release/agent_config.yml").unwrap();

    for plugin in plugins {
        println!("{}", plugin);

        copy(format!("../agent_plugins/{x}/{x}.yml", x=plugin), format!("../target/debug/{x}.yml", x=plugin)).unwrap();
        copy(format!("../agent_plugins/{x}/{x}.yml", x=plugin), format!("../target/release/{x}.yml", x=plugin)).unwrap();
    }
}
