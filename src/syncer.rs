use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use crate::{arb::ArbFile, project::Project, watcher::DirWatcher};

fn diff_contents(
    before_v: &serde_json::Value,
    after_v: &serde_json::Value,
) -> Result<Vec<String>, String> {
    let mut changed = Vec::<String>::new();
    if let Some(before) = before_v.as_object() && let Some(after) = after_v.as_object() {
        for key in after.keys() {
            if !before.contains_key(key) || !before.get(key).eq(&after.get(key)) {
                changed.push(key.to_string());
            }
        }
    }
    Ok(changed)
}
pub fn start(p: Project) -> Result<(), String> {
    let arb_template = ArbFile::new(p.arb_template_path());
    let mut last_content = arb_template.read()?;
    for _ in DirWatcher::new(arb_template.path.as_path())?.flatten() {
        let new_content = arb_template.read()?;
        let changes = diff_contents(&last_content, &new_content)?;
        last_content  = new_content;
        if let Ok(entries) = std::fs::read_dir(p.root_dir.join(&p.l10n_dir)) {
            for dirent in entries.flatten() {
                if dirent.path().extension().is_some() && dirent.path().extension().unwrap().as_bytes().eq("arb".as_bytes()) && !dirent.file_name().as_bytes().eq(p.arb_template.as_bytes()) {
                    let other_arb = ArbFile::new(dirent.file_name().into());
                    let mut other_content = other_arb.read()?;
                    if let Some(main) = last_content.as_object() && let Some(other) = other_content.as_object_mut() {
                        for key in &changes {
                            other.insert(key.clone(), serde_json::Value::String(format!("${0}", main.get(key).expect("Key just disappeared"))));
                        }
                        other_arb.write(&other_content) ?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn spawn(p: Project) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = start(p) {
            println!("Extractor worker error: {e}");
        }
    })
}
