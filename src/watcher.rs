use notify::{recommended_watcher, EventKind, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

use crate::utils::stringe;

pub struct DirWatcher {
    rx: Receiver<notify::Event>,
    _watcher: notify::INotifyWatcher,
}

impl DirWatcher {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let (tx, rx) = channel();
        let mut watcher = stringe(
            "error creating file watcher",
            recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
                Ok(event) => {
                    if let EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) =
                        event.kind
                    {
                        tx.send(event).expect("Failed to send event");
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }),
        )?;
        stringe(
            "Error starting watcher",
            watcher.watch(path.as_path(), notify::RecursiveMode::Recursive),
        )?;
        Ok(Self {
            rx,
            _watcher: watcher,
        })
    }
}

impl Iterator for DirWatcher {
    type Item = Option<PathBuf>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match stringe("Recv error", self.rx.recv()) {
            Ok(event) => {
                if let notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) = event.kind {
                    for path in event.paths {
                        if path.exists() {
                            return Some(Some(path));
                        }
                    }
                }
                None
            }
            Err(e) => {
                println!("{e}");
                None
            }
        })
    }
}
