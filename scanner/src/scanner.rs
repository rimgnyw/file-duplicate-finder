use std::cmp::max;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;
use std::thread::{self};
use std::{fs, io};

use md5::Digest;

pub struct Entry {
    is_dir: bool,
    pub name: OsString,
    pub path: PathBuf,
}

struct ThreadManager {
    working_threads: AtomicUsize,
    max_threads: usize,

    file_map: Mutex<HashMap<Digest, Vec<Entry>>>,
}

impl ThreadManager {
    fn new() -> io::Result<Self> {
        let max_threads = max(1, available_parallelism()?.get() - 1); // remove one thread spot to account for handler thread but make sure we have at least one worker

        Ok(ThreadManager {
            working_threads: AtomicUsize::new(0),
            max_threads,
            file_map: Mutex::new(HashMap::new()),
        })
    }

    fn run_parallel_scan(self: &Arc<Self>, base_dirs: &Vec<PathBuf>) -> io::Result<()> {
        let mut entry_list: Vec<Entry> = Vec::new();
        for dir in base_dirs {
            if !dir.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{:?} is not a directory", dir),
                ));
            }

            match dir.file_name() {
                Some(name) => {
                    let entry = Entry {
                        is_dir: true,
                        name: name.to_os_string(),
                        path: dir.to_owned(),
                    };

                    entry_list.push(entry);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unable to read file name for: {}", dir.display()),
                    ))
                }
            }
        }

        let (send_queue, reciever) = channel();
        let recieve_queue = Arc::new(Mutex::new(reciever));

        for entry in entry_list {
            self.working_threads.fetch_add(1, Ordering::SeqCst);
            send_queue.send(Some(entry)).unwrap();
        }

        let mut threads = Vec::new();
        for _ in 0..self.max_threads {
            let send_clone = send_queue.clone();
            let recieve_clone = recieve_queue.clone();
            let self_clone = self.clone();
            let thread = thread::spawn(move || self_clone.threaded_scan(send_clone, recieve_clone));
            threads.push(thread);
        }

        for thread in threads {
            let _ = thread.join().unwrap();
        }

        Ok(())
    }

    fn threaded_scan(
        self: &Arc<Self>,
        send_queue: Sender<Option<Entry>>,
        recieve_queue: Arc<Mutex<Receiver<Option<Entry>>>>,
    ) -> io::Result<()> {
        loop {
            // blocks the thread until a task is recieved
            let task = {
                let reciever = recieve_queue.lock().unwrap();
                reciever.recv()
            };

            match task {
                Ok(t) => match t {
                    Some(dir) => {
                        let entries = self.find_entries(&dir.path)?;

                        for entry in entries {
                            if entry.is_dir {
                                self.working_threads.fetch_add(1, Ordering::SeqCst);
                                let _ = send_queue.send(Some(entry));
                            } else {
                                let mut files = self.file_map.lock().unwrap();
                                let file_hash = self.hash_file(&entry.path)?;
                                match files.get_mut(&file_hash) {
                                    Some(file) => file.push(entry),
                                    None => _ = files.insert(file_hash, vec![entry]),
                                }
                            }
                        }

                        let prev = self.working_threads.fetch_sub(1, Ordering::SeqCst);
                        if prev == 1 {
                            for _ in 0..self.max_threads {
                                let _ = send_queue.send(None);
                            }

                            drop(send_queue);
                            break;
                        }
                    }
                    None => break,
                },
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn hash_file(&self, path: &PathBuf) -> io::Result<Digest> {
        let file_contents = fs::read(path)?;
        Ok(md5::compute(file_contents))
    }

    fn find_entries(&self, path: &PathBuf) -> io::Result<Vec<Entry>> {
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
}

pub fn run_scan(base_dirs: &Vec<PathBuf>) -> io::Result<HashMap<Digest, Vec<Entry>>> {
    let threaded_scanner = Arc::new(ThreadManager::new()?);
    threaded_scanner.run_parallel_scan(base_dirs)?;

    Ok(Arc::into_inner(threaded_scanner)
        .unwrap()
        .file_map
        .into_inner()
        .unwrap())
}
