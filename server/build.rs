use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// Get the version from environment set by Cargo
fn main() {
    let version = env!("CARGO_PKG_VERSION");

    // Get the output dir where build artifacts go
    let out_dir = std::env::current_dir().unwrap();
    let version_file_path = Path::new(&out_dir).join("version.txt");

    let mut file = File::create(&version_file_path).expect("Failed to create version file");
    write!(file, "{version}").expect("Failed to write version");
    println!("{:?}", version_file_path);
}
