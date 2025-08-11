use std::collections::HashMap;
use std::env::Args;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::iter::Skip;
use std::path::PathBuf;
use std::process::exit;
use std::{env, io};
// use std::time::Instant;

use scanner::run_scan;

mod scanner;

// const BASE_DIR: &str = "./testdata/single-file"; // the base directory used for testing

fn main() {
    let args = env::args().skip(1);

    let args = match parse_args(args) {
        Ok(paths) => paths,
        Err(_) => {
            eprintln!("invalid arguments");
            exit(1)
        }
    };

    match run_scan(&args) {
        Ok(result) => {
            let r = write_log_to_file(&result, "./results.log");
            if r.is_err() {
                eprintln!("Failed to write log");
            }
        }

        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // let duration = start.elapsed();
    // println!("Time Elapse: {:?}", duration);
}

fn parse_args(args: Skip<Args>) -> io::Result<Vec<PathBuf>> {
    let mut args = args
        .map(PathBuf::from)
        .map(|path| path.canonicalize())
        .collect::<Result<Vec<PathBuf>, io::Error>>()?;

    args.sort_by_key(|path| path.components().count());

    let mut result: Vec<PathBuf> = Vec::new();

    let mut found_subdirs = false;
    // Filter out any paths that are contained by other paths
    for arg in args {
        if !result.iter().any(|prefix| arg.starts_with(prefix)) {
            result.push(arg);
        } else {
            found_subdirs = true;
        }
    }

    if found_subdirs {
        println!("INFO: Nested directories were removed");
    }

    Ok(result)
}

#[cfg(not(target_os = "windows"))]
fn write_log_to_file(map: &HashMap<OsString, Vec<PathBuf>>, log_path: &str) -> std::io::Result<()> {
    let file = File::create(log_path)?;
    let mut writer = BufWriter::new(file);

    for (filename, paths) in map {
        if paths.len() > 1 {
            writeln!(writer, "{}:", filename.to_string_lossy())?;
            for path in paths {
                writeln!(writer, "  {}", path.to_owned().display())?;
            }
        }
    }
    println!(
        "Results written to:\n{}",
        PathBuf::from(log_path).canonicalize().unwrap().display()
    );

    Ok(())
}

#[cfg(target_os = "windows")]
fn normalise_paths(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with(r"\\?\") {
        return PathBuf::from(&s[4..]);
    }
    path
}

#[cfg(target_os = "windows")]
fn write_log_to_file(map: &HashMap<OsString, Vec<PathBuf>>, log_path: &str) -> std::io::Result<()> {
    let file = File::create(log_path)?;
    let mut writer = BufWriter::new(file);

    for (filename, paths) in map {
        if paths.len() > 1 {
            writeln!(writer, "{}:", filename.to_string_lossy())?;
            for path in paths {
                writeln!(writer, "  {}", normalise_paths(path.to_owned()).display())?;
            }
        }
    }
    println!(
        "Results written to:\n{}",
        normalise_paths(PathBuf::from(log_path).canonicalize().unwrap()).display()
    );

    Ok(())
}
