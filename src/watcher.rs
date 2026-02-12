use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::{channel, Receiver};

use crate::utils::stringe;

pub struct DirWatcher {
    rx: Receiver<Result<Event, notify::Error>>,
    _watcher: RecommendedWatcher,
    initial_yield: bool,
}

impl DirWatcher {
    pub fn new(path: &Path, initial_run: bool) -> Result<Self, String> {

        let (tx, rx) = channel(100);


        let mut watcher = stringe(
            "error creating file watcher",
            recommended_watcher(move |res: Result<Event, notify::Error>| {
                let _ = tx.blocking_send(res);
            }),
        )?;


        stringe(
            "Error starting watcher",
            watcher.watch(path, RecursiveMode::Recursive),
        )?;

        Ok(Self {
            rx,
            _watcher: watcher,
            initial_yield: initial_run,
        })
    }

    pub async fn next(&mut self) -> Option<PathBuf> {
        if self.initial_yield {
            self.initial_yield = false;
            return Some(PathBuf::new());
        }


        while let Some(res) = self.rx.recv().await {
            match res {
                Ok(event) => {

                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            for path in event.paths {
                                if path.exists() {
                                    return Some(path);
                                }
                            }
                        }
                        _ => continue,
                    }
                }
                Err(e) => {
                    println!("watch error: {:?}", e);
                    continue;
                }
            }
        }
        None
    }
}
