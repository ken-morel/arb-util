use std::process::Stdio;

use crate::{arb::ArbFile, project::Project, watcher::DirWatcher};
use serde_json::Value;
use tokio::time::sleep;

/// Synchronizes keys from the template ARB file to all other ARB files in the directory.
async fn sync_keys(project: &Project) -> Result<(), String> {
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
    flutter_gen().await;
    Ok(())
}

pub async fn flutter_gen() {
    match &mut  tokio::process::Command::new("flutter")
                .arg("gen-l10n")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn() {
        Ok(child) => {
            match child.wait().await {
                Ok(_) => println!("[syncer] sync and generation complete"),
                Err(e) => println!("[syncer] gen-l10n error {e}"),
            }
        }
        Err(e) => println!("[syncer] could not run flutter-genl10n: {e}"),
    }
}

pub async fn run(p: Project) -> Result<(), String> {
    println!("[syncer] Started. Making initial sync.");
    let mut watcher = DirWatcher::new(&p.arb_template_path(), true)?;
    while  watcher.next().await.is_some() {
        sleep(std::time::Duration::from_millis(500)).await;
        println!("[syncer] Template ARB file changed. Re-running sync...");
        if let Err(e) = sync_keys(&p).await {
            println!("[syncer] Error during sync: {}", e);
        }
    }
    Ok(())
}
