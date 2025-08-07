use std::time::Instant;

use scanner::run_scan;
use surveyor::should_use_parallelism;

mod scanner;
mod surveyor;

const BASE_DIR: &str = "./testdata/100-files"; // the base directory used for testing

fn main() {
    // if let Ok(enable_parallelism) = use_parallelism() {
    //     println!("{}", enable_parallelism);
    // }

    let start = Instant::now();

    match should_use_parallelism(BASE_DIR) {
        Ok(with_parallelism) => {
            println!("{}", with_parallelism);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    match run_scan(BASE_DIR) {
        Ok(result) => {}

        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    let duration = start.elapsed();
    println!("Time Elapse: {:?}", duration);
}

#[cfg(test)]
mod testing {

    use super::*;

    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::io::{BufWriter, Write};
    use std::path::PathBuf;

    fn parse_duplicate_files_from_log(
        log_path: &str,
    ) -> std::io::Result<HashMap<OsString, Vec<PathBuf>>> {
        let file = File::open(log_path)?;
        let reader = BufReader::new(file);

        let mut dupes_map: HashMap<OsString, Vec<PathBuf>> = HashMap::new();
        let mut current_filename: Option<OsString> = None;
        let mut in_dupe_section = false;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            // Start reading when the section starts
            if trimmed == "Duplicate files and their locations:" {
                in_dupe_section = true;
                continue;
            }

            if !in_dupe_section {
                continue;
            }

            if trimmed.ends_with(':') && !trimmed.contains('/') {
                // This is a filename line, e.g., "d3.txt:"
                let filename = trimmed.trim_end_matches(':').to_string();
                current_filename = Some(OsString::from(filename));
            } else if let Some(filename) = &current_filename {
                // This is a file path line
                let path = PathBuf::from(trimmed);
                dupes_map.entry(filename.clone()).or_default().push(path);
            }
        }

        Ok(dupes_map)
    }
    fn write_map_to_file(
        map: &HashMap<OsString, Vec<PathBuf>>,
        file_path: &str,
    ) -> std::io::Result<()> {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        for (filename, paths) in map {
            writeln!(writer, "{}:", filename.to_string_lossy())?;
            for path in paths {
                writeln!(writer, "  {}", path.display())?;
            }
        }

        Ok(())
    }

    fn compare_with_log(path: &str) {
        let mut result = run_scan(path).unwrap();
        result.remove(&OsString::from("log.txt"));

        let keys_to_remove: Vec<OsString> = result
            .iter()
            .filter(|(_, paths)| paths.len() < 2)
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            result.remove(&key);
        }

        let reference = parse_duplicate_files_from_log(&format!("{}/log.txt", path)).unwrap();

        let _ = write_map_to_file(&result, "./result_log.txt");
        let _ = write_map_to_file(&reference, "./reference_log.txt");

        assert_eq!(
            result.len(),
            reference.len(),
            "Mismatch in number of duplicate filenames"
        );

        for (filename, reference_paths) in &reference {
            let result_paths = result
                .get(filename)
                .expect(&format!("Missing entry for {:?}", filename));

            assert_eq!(
                result_paths.len(),
                reference_paths.len(),
                "Mismatch in number of paths for {:?}",
                filename
            );

            // Optionally sort for comparison if order doesn't matter
            let mut result_sorted = result_paths.clone();
            let mut reference_sorted = reference_paths.clone();

            result_sorted.sort();
            reference_sorted.sort();

            assert_eq!(
                result_sorted, reference_sorted,
                "Paths mismatch for {:?}",
                filename
            );
        }
    }

    #[test]
    fn test_2() {
        compare_with_log("./testdata/two-subdirs-with-dupes");
    }
    #[test]
    fn files_100() {
        compare_with_log("./testdata/100-files");
    }
}
