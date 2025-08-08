use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use scanner::run_scan;

mod scanner;

const BASE_DIR: &str = "./testdata/100-files"; // the base directory used for testing

fn main() {
    let start = Instant::now();

    match run_scan(BASE_DIR) {
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

    if map.len() == 0 {
        println!("No duplicates found");
        writeln!(writer, "No duplicates found")?;
        return Ok(());
    }
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
