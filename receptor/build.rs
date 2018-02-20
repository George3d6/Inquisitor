use std::fs::copy;


fn main() {
    let common_files = vec!["status.rs"];
    for file in common_files {
        copy(["../", file].join(""), file);
    }
}
