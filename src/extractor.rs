use crate::project::Project;
use crate::utils::{id_string, localization_getter};
use crate::watcher::DirWatcher;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Extracts marked strings from a Dart file, replaces them with localization getters,
/// and returns the modified content plus the new key-value pairs.
fn extract_from_file(file: &Path) -> Result<(String, BTreeMap<String, String>), String> {
    let content = std::fs::read_to_string(file).map_err(|e| e.to_string())?;
    let translation_string_re = Regex::new("_\"((?:\\\\\"|[^\"])*)\"").unwrap();

    let mut new_strings = BTreeMap::new();
    let mut modified_content = content.clone();
    let mut has_changed = false;

    for cap in translation_string_re.captures_iter(&content).collect::<Vec<_>>().into_iter().rev() {
        let full_match = cap.get(0).unwrap();
        let inner_string = cap.get(1).unwrap().as_str();

        let id = id_string(inner_string);
        modified_content.replace_range(full_match.start()..full_match.end(), &localization_getter(&id));
        new_strings.entry(id).or_insert(inner_string.to_string());
        has_changed = true;
    }

    if has_changed {
        Ok((modified_content, new_strings))
    } else {
        Ok((content, BTreeMap::new()))
    }
}

/// Adds new strings to the template ARB file.
fn update_arb_file(project: &Project, new_strings: &BTreeMap<String, String>) -> Result<bool, String> {
    if new_strings.is_empty() {
        return Ok(false);
    }

    let arb_path = project.arb_template_path();
    let arb_content = std::fs::read_to_string(&arb_path).map_err(|e| e.to_string())?;
    let mut arb_data: BTreeMap<String, Value> =
        serde_json::from_str(&arb_content).map_err(|e| e.to_string())?;

    let mut changed = false;
    for (key, value) in new_strings {
        if !arb_data.contains_key(key) {
            println!("[extractor] Adding new key: {}", key);
            arb_data.insert(key.clone(), Value::String(value.clone()));
            let metadata_key = format!("@{}", key);
            if !arb_data.contains_key(&metadata_key) {
                arb_data.insert(metadata_key, Value::Object(Map::new()));
            }
            changed = true;
        }
    }

    if changed {
        let updated_content = serde_json::to_string_pretty(&arb_data).map_err(|e| e.to_string())?;
        std::fs::write(&arb_path, updated_content).map_err(|e| e.to_string())?;
    }

    Ok(changed)
}

/// Adds the necessary localization import to the Dart file if it's missing.
fn ensure_localization_import(project: &Project, content: &mut String) {
    let l10n_path_str = project.l10n_dir.strip_prefix("lib/").unwrap_or(&project.l10n_dir).to_str().unwrap();
    let import_statement = format!(
        "import 'package:{}/{}/{}.dart';",
        project.name, l10n_path_str, project.localizations_file
    );
    let import_re = Regex::new(&format!("import.*{}.dart'", project.localizations_file)).unwrap();
    
    if !import_re.is_match(content) {
        println!("[extractor] Adding import: {}", import_statement);
        content.insert_str(0, &format!("{}\n", import_statement));
    }
}

fn process_file(p: &Project, path: &Path) -> Result<(), String> {
    match extract_from_file(path) {
        Ok((mut modified_content, new_strings)) => {
            if !new_strings.is_empty() {
                if update_arb_file(p, &new_strings)? {
                     ensure_localization_import(p, &mut modified_content);
                }
                std::fs::write(path, modified_content).map_err(|e| e.to_string())?;
                println!("[extractor] Updated {} and template ARB file.", path.display());
            }
        }
        Err(e) => {
            println!("[extractor] Error processing file {}: {}", path.display(), e);
        }
    }
    Ok(())
}

pub fn start(p: Project) -> Result<(), String> {
    let lib_dir = p.root_dir.join("lib");
    println!("[extractor] Starting initial scan of 'lib/' directory...");
    for entry in walkdir::WalkDir::new(&lib_dir) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "dart") {
            process_file(&p, path)?;
        }
    }
    println!("[extractor] Initial scan complete. Now watching for changes...");

    for path_opt in DirWatcher::new(&lib_dir)?.flatten() {
        if let Some(path) = path_opt {
             if path.is_file() && path.extension().map_or(false, |ext| ext == "dart") {
                println!("[extractor] File changed: {}", path.display());
                std::thread::sleep(std::time::Duration::from_millis(300)); // Debounce
                if let Err(e) = process_file(&p, &path) {
                    println!("[extractor] Error processing change for {}: {}", path.display(), e);
                }
            }
        }
    }
    Ok(())
}

pub fn spawn(p: Project) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = start(p) {
            println!("[extractor] Worker thread terminated with error: {e}");
        }
    })
}
