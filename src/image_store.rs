// SPDX-License-Identifier: GPL-3.0

use cosmic::widget::image::Handle;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct ImageStore {
    artwork_dir: PathBuf,
    cache: Arc<Mutex<HashMap<PathBuf, Arc<Handle>>>>,
    queue: Arc<Mutex<VecDeque<PathBuf>>>,
    tx: mpsc::Sender<PathBuf>,
}

impl ImageStore {
    pub fn new(artwork_dir: PathBuf) -> Self {
        let (tx, mut rx) = mpsc::channel::<PathBuf>(64);

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        let cache_clone = cache.clone();
        let queue_clone = queue.clone();

        tokio::spawn(async move {
            while let Some(path) = rx.recv().await {
                // Remove path from queue
                queue_clone.lock().unwrap().retain(|p| p != &path);

                // If path is already in cache, skip loading
                if cache_clone.lock().unwrap().contains_key(&path) {
                    continue;
                }

                match fs::read(&path) {
                    Ok(data) => {
                        cache_clone.lock().unwrap().insert(
                            path,
                            Arc::new(cosmic::widget::image::Handle::from_bytes(data)),
                        );
                    }
                    Err(err) => {
                        eprintln!("Failed to load image: {:?} {}", path, err);
                    }
                }
            }
        });

        Self {
            artwork_dir,
            cache,
            queue,
            tx,
        }
    }
}

impl ImageStore {
    pub fn request(&self, path: String) {
        let artwork_path = self.artwork_dir.join(path);

        if self.cache.lock().unwrap().contains_key(&artwork_path) {
            return;
        }

        let mut q = self.queue.lock().unwrap();
        if q.contains(&artwork_path) {
            return;
        }

        q.push_back(artwork_path.clone());
        let _ = self.tx.try_send(artwork_path);
    }

    pub fn get(&self, path: &String) -> Option<Arc<Handle>> {
        let artwork_path = self.artwork_dir.join(path);
        self.cache.lock().unwrap().get(&artwork_path).cloned()
    }
}
