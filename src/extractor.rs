use crate::project::Project;
use crate::utils::{id_string, localization_getter, stringe};
use crate::watcher::DirWatcher;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;
use tokio::time::sleep;

type ExtractResult = Result<Option<(String, BTreeMap<String, String>)>, String>;

/// Extract marked strings from the file and replace them with `AppLocalizations` calls
/// return the modifed content and Ordered mapping of the extracted strings.
fn extract_from_file(file: &Path) -> ExtractResult {
    let content = stringe(
        "could not read the file content",
        std::fs::read_to_string(file),
    )?;
    let translation_string_re = Regex::new("_(\"((?:\\\\\"|[^\"])*)\")").unwrap();

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
                println!("Could not parse dart string({raw_inner_string}): {e}");
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
            arb_data.insert(key.clone(), Value::String(value.clone()));
            let metadata_key = format!("@{}", key);
            let metadata = create_metadata(value);
            println!("[extractor] Adding new key: {key} with metadata {metadata:?}");
            arb_data
                .entry(metadata_key)
                .or_insert(Value::Object(metadata));
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

fn create_metadata(txt: &str) -> Map<String, Value> {
    let mut metadata = Map::new();

    let regex = Regex::new("\\{(\\w+)\\}").unwrap();

    let mut placeholders = Map::new();
    for captures in  regex.captures_iter(txt) {
        let mut val = Map::new();
        val.insert(String::from("type"), Value::String(String::from("String")));
        placeholders.insert(String::from(captures.get(1).unwrap().as_str()), Value::Object(val));
    }
    if !placeholders.is_empty() {
        metadata.insert(String::from("placeholders"), Value::Object(placeholders));
    }
    metadata
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
        print!("[extractor] Adding {}", import_statement);
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
                "[extractor] Updated {}.",
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

pub async fn run(p: Project) -> Result<(), String> {
    let lib_dir = p.root_dir.join("lib");

    println!("[extractor] Extractor started, making initial run");

    let mut watcher = DirWatcher::new(&lib_dir, true)?;
    while let Some(path) = watcher.next().await {
        sleep(std::time::Duration::from_millis(300)).await; // Debounce
        if path.is_file() && path.extension().is_some_and(|ext| ext == "dart") && let Err(e) = process_file(&p, &path) {
            println!(
                "[extractor] Error processing change for {}: {}",
                path.display(),
                e
            );
        } else {
            for entry in stringe("Could not list files in lib/ directory", std::fs::read_dir(&lib_dir))?.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "dart") {
                    process_file(&p, path.as_path())?;
                }
            }
        }
    }
    Ok(())
}
