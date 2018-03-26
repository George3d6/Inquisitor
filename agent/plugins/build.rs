extern crate cargo_metadata;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Get list of 'dependencies'
    let v = &cargo_metadata::metadata_deps(Some(Path::new("Cargo.toml")), false)
        .unwrap()
        .packages[0]
        .dependencies;
    let mut plugins = vec![];
    for p in v {
        if p.kind == cargo_metadata::DependencyKind::Normal && p.name != "agent_lib" {
            plugins.push(p.name.clone());
        }
    }
    println!("{:?}", plugins);

    // Write to src/lib.rs
    let mut f = File::create("src/lib.rs").unwrap();
    f.write_all(
        format!(
    "extern crate agent_lib;
    use agent_lib::plugin_interface::AgentPlugin;

    #[macro_use]
    mod plugin_initialization;

    plugins!({});",
            plugins.join(", ")
        ).as_bytes(),
    ).unwrap();
}
