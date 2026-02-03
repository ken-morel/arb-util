use crate::utils::{id_string, localization_getter, stringe};

use super::{project::Project, watcher::DirWatcher};
use regex::Regex;
use std::{os::unix::ffi::OsStrExt, path::PathBuf};

fn register_extracted_string(p: &Project, key: &str, value: &str) -> Result<(), String> {
    let content = stringe(
        "could not main arb file content",
        std::fs::read_to_string(p.arb_template_path()),
    )?;
    let mut json: serde_json::Value = stringe(
        "could not decode main arb file yaml",
        serde_json::from_str(content.as_str()),
    )?;
    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        let new_data = stringe(
            "could not serialize modified arb file back to json ",
            serde_json::to_string_pretty(&json),
        )?;
        if let Err(e) = stringe(
            "could not write json content to arb file",
            std::fs::write(p.arb_template_path(), new_data),
        ) {
            Err(e)
        } else {
            Ok(())
        }
    } else {
        Err(String::from(
            "Invalid arb file, arb file should contain json object",
        ))
    }
}

fn extract(p: &Project, file: PathBuf) {
    if let Ok(content) = std::fs::read_to_string(&file) {
        let mut result = content.clone();
        for m in Regex::new("_\"(?:\\\\\"|[^\"])+\"")
            .unwrap()
            .find_iter(content.as_str())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let mut content = m.as_str().to_string();
            content.remove(0);
            content.remove(0);
            content.remove(content.len() - 1);
            let id = id_string(&content);
            result.replace_range(m.start()..m.end(), &localization_getter(&id));
            if let Err(e) = register_extracted_string(p, &id, &content) {
                println!(
                    "Could not add extracted string to file: {0}={1}; {2}",
                    id, content, e
                );
            }
        }
        if !content.eq(&result) {
            // check if the localizations file is already imported
            if !Regex::new(format!("import.+{0}", p.localizations_file).as_str())
                .unwrap()
                .is_match(result.as_str())
            {
                result.insert(0, '\n');
                result.insert_str(
                    0,
                    format!(
                        "import 'package:{0}/{1}/{2}';",
                        p.name,
                        p.l10n_dir
                            .to_str()
                            .expect("Could not convert path to string")
                            .split_at(4)
                            .1,
                        p.localizations_file
                    )
                    .as_str(),
                );
            }
            _ = std::fs::write(file, result);
        }
    }
}
pub fn start(p: Project) -> Result<(), String> {
    for path in DirWatcher::new(p.root_dir.join("lib"))?.flatten() {
        if path.extension().is_some() && path.extension().unwrap().as_bytes().eq("dart".as_bytes())
        {
            std::thread::sleep(std::time::Duration::from_millis(100));
            extract(&p, path);
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
