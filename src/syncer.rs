use crate::{arb::ArbFile, project::Project, watcher::DirWatcher};
use serde_json::Value;

/// Synchronizes keys from the template ARB file to all other ARB files in the directory.
fn sync_keys(project: &Project) -> Result<(), String> {
    let template_path = project.arb_template_path();
    let template_arb = ArbFile::new(template_path.clone());
    let template = template_arb.read()?;

    let l10n_dir = project.root_dir.join(&project.l10n_dir);

    if let Ok(entries) = std::fs::read_dir(l10n_dir) {
        for dirent in entries.flatten() {
            let path = dirent.path();
            // Skip the template file itself and any non-ARB files
            if dirent
                .file_name()
                .to_str()
                .is_none_or( |s| s.eq(&project.arb_template)) // default to true means we skip files with strange names
                || path.extension().is_none_or(|ext| ext != "arb")
            {
                continue;
            }

            println!("[syncer] Checking file: {:?}", path.file_name().unwrap());
            let other_arb = ArbFile::new(path);
            let mut other_content = other_arb.read()?;

            let mut changed = false;
            for &key in &template
                .keys()
                .filter(|k| !k.starts_with('@'))
                .collect::<Vec<_>>()
            {
                if !other_content.contains_key(key) {
                    let template_value = template.get(key).unwrap().as_str().unwrap_or("");
                    let placeholder = format!("#{}", template_value);
                    println!("  -> Adding missing key '{}' with placeholder", key);
                    other_content.insert(key.clone(), Value::String(placeholder));
                    changed = true;
                }
            }

            if changed {
                other_arb.write(&other_content)?;
            }
        }
    }
    Ok(())
}

pub fn start(p: Project) -> Result<(), String> {
    println!("[syncer] Started. Performing initial sync...");
    sync_keys(&p)?;
    println!("[syncer] Initial sync complete. Watching for changes in template ARB file...");

    // This watcher specifically monitors the template ARB file.
    for _ in DirWatcher::new(&p.arb_template_path())?.flatten() {
        // Debounce: Wait a moment to avoid rapid-fire event processing.
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("[syncer] Template ARB file changed. Re-running sync...");
        if let Err(e) = sync_keys(&p) {
            println!("[syncer] Error during sync: {}", e);
        }
    }
    Ok(())
}

pub fn spawn(p: Project) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = start(p) {
            println!("[syncer] Worker thread terminated with error: {e}");
        }
    })
}
