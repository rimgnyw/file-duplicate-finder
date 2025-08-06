use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;
use std::thread::{self};
use std::{fs, io};

use crate::surveyor::should_use_parallelism;

struct Entry {
    is_dir: bool,
    name: OsString,
    path: PathBuf,
}

pub struct ThreadManager {
    // queue: Mutex<VecDeque<Entry>>,
    send_queue: Sender<Entry>,
    recieve_queue: Mutex<Receiver<Entry>>,
    // active_threads: Mutex<usize>,
    working_threads: AtomicUsize,
    max_threads: usize,

    name_map: Mutex<HashMap<OsString, Vec<PathBuf>>>,
}

impl ThreadManager {
    fn new() -> io::Result<Self> {
        let max_threads = available_parallelism()?.get() - 1; // remove one thread spot to account for handler thread

        let (send_queue, recieve_queue) = channel();
        Ok(ThreadManager {
            // queue,
            send_queue,
            recieve_queue: Mutex::new(recieve_queue),
            working_threads: AtomicUsize::new(0),
            max_threads,
            name_map: Mutex::new(HashMap::new()),
        })
    }

    fn run_parallel_scan(self: &Arc<Self>, base_dir: &str) {
        let dir = Path::new(base_dir);

        match dir.file_name() {
            Some(name) => {
                let entry = Entry {
                    is_dir: true, // Surveyor wouldn't want to use threads if it wasn't a dir
                    name: name.to_os_string(),
                    path: dir.to_path_buf().canonicalize().unwrap(),
                };

                self.send_queue.send(entry).unwrap();

                let mut threads = Vec::new();
                for _ in 0..7 {
                    let self_clone = self.clone();
                    let thread = thread::spawn(move || self_clone.threaded_scan());
                    threads.push(thread);
                }

                for thread in threads {
                    let _ = thread.join().unwrap();
                }
                println!("I'm done waiting");
                // let mut q = queue.lock().unwrap();
                // q.push_back(entry);
                // drop(q);
            }
            _ => eprintln!("Invalid base directory path: {}", base_dir),
        }
    }

    fn threaded_scan(self: &Arc<Self>) -> io::Result<()> {
        // let mut active_threads = self.active_threads.lock().unwrap();
        // *active_threads += 1;
        // drop(active_threads);

        let sender_clone = self.send_queue.clone();

        loop {
            // blocks the thread until a task is recieved
            let task = {
                let reciever = self.recieve_queue.lock().unwrap();
                reciever.recv()
            };
            println!("new task");

            match task {
                Ok(dir) => {
                    let entries = self.find_entries(dir.path)?;

                    for entry in entries {
                        if entry.is_dir {
                            self.working_threads.fetch_add(1, Ordering::SeqCst);
                            let _ = sender_clone.send(entry);
                        } else {
                            let mut names = self.name_map.lock().unwrap();
                            match names.get_mut(&entry.name) {
                                Some(name) => name.push(entry.path),
                                None => _ = names.insert(entry.name, vec![entry.path]),
                            }
                        }
                    }

                    let prev = self.working_threads.fetch_sub(1, Ordering::SeqCst);
                    println!("working threads: {}", prev);
                    if prev == 1 {
                        println!("got done");
                        drop(sender_clone);
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn find_entries(&self, path: PathBuf) -> io::Result<Vec<Entry>> {
        let entries = fs::read_dir(path)?
            .map(|res| {
                let e = res?;
                Ok(Entry {
                    is_dir: e.file_type()?.is_dir(),
                    name: e.file_name(),
                    path: e.path(),
                })
            })
            .collect::<Result<Vec<Entry>, io::Error>>()?;

        Ok(entries)
    }

    pub fn list_dupes(&self) {
        let map = self.name_map.lock().unwrap();
        for (name, paths) in map.iter() {
            if paths.len() > 1 {
                println!("{:?} exists at {:?}", name, paths);
            }
        }
    }
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

    fn sequential_scan(&mut self, dir: PathBuf) -> io::Result<()> {
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
                self.sequential_scan(entry.path)?;
            } else {
                match self.name_map.get_mut(&entry.name) {
                    Some(name) => name.push(entry.path),
                    None => _ = self.name_map.insert(entry.name, vec![entry.path]),
                };
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
}
pub fn run_scan(root: &str) -> io::Result<()> {
    if should_use_parallelism(root)? {
        let threaded_scanner = Arc::new(ThreadManager::new()?);
        threaded_scanner.run_parallel_scan(root);
        // threaded_scanner.list_dupes();

        println!("parallelism done");

        Ok(())
    } else {
        let mut sequential_scanner = Scanner::new();
        sequential_scanner.sequential_scan(PathBuf::from(root))?;
        // sequential_scanner.list_dupes();
        println!("sequential done");
        Ok(())
    }
}
