use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{fs, io};

use crate::surveyor::should_use_parallelism;

struct Entry {
    is_dir: bool,
    name: OsString,
    path: PathBuf,
}

pub struct Scanner {
    name_map: HashMap<OsString, Vec<PathBuf>>,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            name_map: HashMap::new(),
        }
    }

    fn recursive_scan(&mut self, dir: PathBuf) -> io::Result<()> {
        let entries = fs::read_dir(dir)?.map(|res| -> Result<Entry, std::io::Error> {
            let e = res?;
            Ok(Entry {
                is_dir: e.file_type()?.is_dir(),
                name: e.file_name(),
                path: e.path().canonicalize()?,
            })
        });

        for entry_result in entries {
            let entry = entry_result?;
            if entry.is_dir {
                self.recursive_scan(entry.path)?;
            } else {
                if self.name_map.contains_key(&entry.name) {
                    self.name_map.get_mut(&entry.name).unwrap().push(entry.path);
                } else {
                    self.name_map.insert(entry.name, vec![entry.path]);
                }
            }
        }

        Ok(())
    }

    pub fn list_dupes(&self) {
        for (name, paths) in &self.name_map {
            if paths.len() > 1 {
                println!("{:?} exists at {:?}", name, paths);
            }
        }
    }
    pub fn run_scan(&mut self, root: &str) -> io::Result<()> {
        if should_use_parallelism(root)? {
            unimplemented!();
        } else {
            self.recursive_scan(PathBuf::from(root))?;
            self.list_dupes();
            Ok(())
        }
    }
}
