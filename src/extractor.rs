use crate::project::Project;
use crate::utils::{id_string, localization_getter, stringe};
use crate::watcher::DirWatcher;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

type ExtractResult = Result<Option<(String, BTreeMap<String, String>)>, String>;

/// Extract marked strings from the file and replace them with `AppLocalizations` calls
/// return the modifed content and Ordered mapping of the extracted strings.
fn extract_from_file(file: &Path) -> ExtractResult {
    let content = stringe(
        "could not read the file content",
        std::fs::read_to_string(file),
    )?;
    let translation_string_re = Regex::new("_([\"'](?:\\\\\"|\\\\'|[^\"'])*[\"'])").unwrap();

    let mut new_strings = BTreeMap::new();
    let mut new_content = content.clone();
    let mut changed = false;

    for cap in translation_string_re
        .captures_iter(&content)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let full_match = cap.get(0).unwrap();
        let raw_inner_string = cap.get(1).unwrap().as_str();
        let string_content = match serde_json::from_str(raw_inner_string) {
            Ok(o) => o,
            Err(e) => {
                println!("Could not parse dart string: {e}");
                continue;
            }
        };

        let id = id_string(string_content);
        new_content.replace_range(
            full_match.start()..full_match.end(),
            &localization_getter(&id),
        );
        new_strings.entry(id).or_insert(string_content.to_string());
        changed = true;
    }

    if changed {
        Ok(Some((new_content, new_strings)))
    } else {
        Ok(None)
    }
}

fn update_arb_file(
    project: &Project,
    new_strings: &BTreeMap<String, String>,
) -> Result<bool, String> {
    if new_strings.is_empty() {
        return Ok(false);
    }

    let arb_path = project.arb_template_path();
    let arb_content = stringe(
        "Could not read arb file content",
        std::fs::read_to_string(&arb_path),
    )?;
    let mut arb_data: BTreeMap<String, Value> =
        serde_json::from_str(&arb_content).map_err(|e| e.to_string())?;

    let mut changed = false;
    for (key, value) in new_strings {
        if !arb_data.contains_key(key) {
            println!("[extractor] Adding new key: {}", key);
            arb_data.insert(key.clone(), Value::String(value.clone()));
            let metadata_key = format!("@{}", key);
            arb_data
                .entry(metadata_key)
                .or_insert(Value::Object(Map::new()));
            changed = true;
        }
    }

    if changed {
        let updated_content = serde_json::to_string_pretty(&arb_data).map_err(|e| e.to_string())?;
        stringe(
            "could not write back arb file content after adding new keys",
            std::fs::write(&arb_path, updated_content),
        )?;
    }

    Ok(changed)
}

fn ensure_localization_import(project: &Project, content: &mut String) {
    let l10n_path_str = project
        .l10n_dir
        .strip_prefix("lib/")
        .unwrap_or(&project.l10n_dir)
        .to_str()
        .unwrap();
    let import_statement = format!(
        "import 'package:{}/{}/{}';\n",
        project.name, l10n_path_str, project.localizations_file
    );
    let import_re = Regex::new(&format!("import.*{}", project.localizations_file)).unwrap();

    if !import_re.is_match(content) {
        println!("[extractor] Adding import: {}", import_statement);
        content.insert_str(0, &import_statement);
    }
}

fn process_file(p: &Project, path: &Path) -> Result<(), String> {
    match extract_from_file(path) {
        Ok(Some((mut modified_content, new_strings))) => {
            if update_arb_file(p, &new_strings)? {
                ensure_localization_import(p, &mut modified_content);
            }
            std::fs::write(path, modified_content).map_err(|e| e.to_string())?;
            println!(
                "[extractor] Updated {} and template ARB file.",
                path.display()
            );
        }
        Ok(None) => {}
        Err(e) => {
            println!(
                "[extractor] Error processing file {}: {}",
                path.display(),
                e
            );
        }
    }
    Ok(())
}

pub fn start(p: Project) -> Result<(), String> {
    let lib_dir = p.root_dir.join("lib");
    println!("[extractor] Starting initial scan of 'lib/' directory...");
    for entry in stringe("Could not list files in arb", std::fs::read_dir(&lib_dir))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "dart") {
            process_file(&p, path.as_path())?;
        }
    }
    println!("[extractor] Initial scan complete. Now watching for changes...");

    for path in DirWatcher::new(&lib_dir)?.flatten() {
        if path.is_file() && path.extension().is_some_and(|ext| ext == "dart") {
            println!("[extractor] File changed: {}", path.display());
            std::thread::sleep(std::time::Duration::from_millis(300)); // Debounce
            if let Err(e) = process_file(&p, &path) {
                println!(
                    "[extractor] Error processing change for {}: {}",
                    path.display(),
                    e
                );
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
