use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use scanner::run_scan;

mod scanner;

// const BASE_DIR: &str = "./testdata/single-file"; // the base directory used for testing

fn main() {
    let mut args = env::args().skip(1);
    let base_dir = args.next();
    if base_dir.is_none() {
        eprintln!("Error: Not enough arguments");
        return;
    }

    if !args.next().is_none() {
        eprintln!("Error: Too many arguments");
        return;
    }

    let start = Instant::now();

    match run_scan(&base_dir.unwrap()) {
        Ok(result) => {
            let r = write_log_to_file(&result, "./scan.log");
            if r.is_err() {
                eprintln!("Failed to write log");
            } else {
                println!(
                    "log written to {:?}",
                    PathBuf::from("./scan.log").canonicalize().unwrap()
                );
            }
        }

        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    let duration = start.elapsed();
    println!("Time Elapse: {:?}", duration);
}

fn write_log_to_file(
    map: &HashMap<OsString, Vec<PathBuf>>,
    file_path: &str,
) -> std::io::Result<()> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    for (filename, paths) in map {
        if paths.len() > 1 {
            writeln!(writer, "{}:", filename.to_string_lossy())?;
            for path in paths {
                writeln!(writer, "  {}", path.display())?;
            }
        }
    }

    Ok(())
}
