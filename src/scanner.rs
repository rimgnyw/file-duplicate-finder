/*
*   The scanner quickly scans all files to determine if
*   parallelism is worth it
*
* */
use std::{
    collections::VecDeque,
    fs::{self, metadata},
    io,
    path::PathBuf,
};

const BASE_DIR: &str = "./testdata"; // the base directory used for testing
const PARALLELISM_THREASHOLD: usize = 100; // the threshold where we decide parallelism is worth using

struct FsEntry {
    is_dir: bool,
    path: PathBuf,
}

pub struct Scanner {
    count: usize,
    queue: VecDeque<FsEntry>,
}
pub fn use_parallelism() -> io::Result<bool> {
    let mut scanner = Scanner::new(BASE_DIR)?;
    let result = scanner.scan_dirs()?;
    Ok(result)
}

impl Scanner {
    pub fn new(base_dir: &str) -> io::Result<Scanner> {
        let mut queue: VecDeque<FsEntry> = VecDeque::new();
        let file = metadata(base_dir)?;

        let entry = FsEntry {
            is_dir: file.is_dir(),
            path: PathBuf::from(base_dir),
        };

        queue.push_back(entry);

        let scanner = Scanner { count: 0, queue };
        Ok(scanner)
    }

    pub fn scan_dirs(&mut self) -> io::Result<bool> {
        while !self.queue.is_empty() {
            if self.count >= PARALLELISM_THREASHOLD {
                return Ok(true);
            }

            let dir = self.queue.pop_front().unwrap();

            if !dir.is_dir {
                continue;
            }

            let objects = self.find_entries(dir.path)?;

            self.count += objects.len();

            for object in objects {
                self.queue.push_back(object);
            }
        }
        Ok(false)
    }

    fn find_entries(&self, path: PathBuf) -> io::Result<Vec<FsEntry>> {
        let objects = fs::read_dir(path)?
            .map(|res| {
                let e = res?;
                Ok(FsEntry {
                    is_dir: e.file_type()?.is_dir(),
                    path: e.path(),
                })
            })
            .collect::<Result<Vec<FsEntry>, io::Error>>()?;

        Ok(objects)
    }
}
