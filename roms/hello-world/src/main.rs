use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;

use purremu_gb_hello_world_rom::build_rom;

fn main() -> io::Result<()> {
    let output = env::args_os()
        .nth(1)
        .unwrap_or_else(|| OsString::from("target/hello-world.gb"));
    let output = PathBuf::from(output);

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output, build_rom())?;
    println!("wrote {}", output.display());

    Ok(())
}
