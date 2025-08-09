use std::cmp::max;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;
use std::thread::{self};
use std::{fs, io};

struct Entry {
    is_dir: bool,
    name: OsString,
    path: PathBuf,
}

struct ThreadManager {
    working_threads: AtomicUsize,
    max_threads: usize,

    name_map: Mutex<HashMap<OsString, Vec<PathBuf>>>,
}

impl ThreadManager {
    fn new() -> io::Result<Self> {
        let max_threads = max(1, available_parallelism()?.get() - 1); // remove one thread spot to account for handler thread but make sure we have at least one worker

        Ok(ThreadManager {
            working_threads: AtomicUsize::new(0),
            max_threads,
            name_map: Mutex::new(HashMap::new()),
        })
    }

    fn run_parallel_scan(self: &Arc<Self>, base_dir: &str) -> io::Result<()> {
        let dir = Path::new(base_dir);

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
                    path: dir.to_path_buf().canonicalize()?,
                };

                let (send_queue, reciever) = channel();
                let recieve_queue = Arc::new(Mutex::new(reciever));

                self.working_threads.fetch_add(1, Ordering::SeqCst);
                send_queue.send(Some(entry)).unwrap();

                let mut threads = Vec::new();
                for _ in 0..self.max_threads {
                    let send_clone = send_queue.clone();
                    let recieve_clone = recieve_queue.clone();
                    let self_clone = self.clone();
                    let thread =
                        thread::spawn(move || self_clone.threaded_scan(send_clone, recieve_clone));
                    threads.push(thread);
                }

                for thread in threads {
                    let _ = thread.join().unwrap();
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid base directory path: {}", base_dir),
                ))
            }
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
                        let entries = self.find_entries(dir.path)?;

                        for entry in entries {
                            if entry.is_dir {
                                self.working_threads.fetch_add(1, Ordering::SeqCst);
                                let _ = send_queue.send(Some(entry));
                            } else {
                                let mut names = self.name_map.lock().unwrap();
                                match names.get_mut(&entry.name) {
                                    Some(name) => name.push(entry.path),
                                    None => _ = names.insert(entry.name, vec![entry.path]),
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
}

pub fn run_scan(root: &str) -> io::Result<HashMap<OsString, Vec<PathBuf>>> {
    let threaded_scanner = Arc::new(ThreadManager::new()?);
    threaded_scanner.run_parallel_scan(root)?;

    Ok(Arc::into_inner(threaded_scanner)
        .unwrap()
        .name_map
        .into_inner()
        .unwrap())
}
