use crate::{arb::ArbFile, project::Project, watcher::DirWatcher};
use rayon::prelude::*;
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[derive(Debug)]
struct TranslationJob {
    key: String,
    text: String,
    lang: String,
    arb_file: ArbFile,
}

/// Executes the external `translate` script for a given text and language.
fn run_translate_script(text: &str, lang: &str) -> Result<String, String> {
    let cmd = format!("echo \"{}\" | translate {}", text, lang);
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("Failed to execute translate script: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Translate script failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Scans all auxiliary ARB files and collects a list of strings that need translation.
fn find_untranslated_strings(project: &Project) -> Result<Vec<TranslationJob>, String> {
    let mut jobs = Vec::new;
    let l10n_dir = project.root_dir.join(&project.l10n_dir);
    let template_path = project.arb_template_path();

    for entry in std::fs::read_dir(l10n_dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if path == template_path || path.extension().map_or(true, |ext| ext != "arb") {
            continue;
        }

        let lang = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split('_')
            .last()
            .unwrap_or_default();

        if lang.is_empty() {
            continue;
        }

        let arb_file = ArbFile::new(path);
        if let Ok(content) = arb_file.read() {
            if let Some(obj) = content.as_object() {
                for (key, value) in obj {
                    if let Some(s_val) = value.as_str() {
                        if s_val.starts_with('#') {
                            jobs.push(TranslationJob {
                                key: key.clone(),
                                text: s_val.strip_prefix('#').unwrap().to_string(),
                                lang: lang.to_string(),
                                arb_file: ArbFile::new(arb_file.path.clone()),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(jobs)
}

/// The main translation loop.
pub fn start(p: Project) -> Result<(), String> {
    println!("[translator] Started. Performing initial scan for untranslated strings...");
    
    let initial_jobs = find_untranslated_strings(&p)?;
    if !initial_jobs.is_empty() {
        println!("[translator] Found {} initial job(s). Translating in parallel...", initial_jobs.len());
        initial_jobs.into_par_iter().for_each(|job| {
            println!("[translator] Translating '{}' to {}...", job.key, job.lang);
            match run_translate_script(&job.text, &job.lang) {
                Ok(translated_text) => {
                    if let Err(e) = job.arb_file.add_key(&job.key, &translated_text) {
                         println!("[translator] ERROR: Failed to write translation for key '{}': {}", job.key, e);
                    }
                }
                Err(e) => {
                     println!("[translator] ERROR: Failed to translate key '{}': {}", job.key, e);
                }
            }
        });
    }
    
    println!("[translator] Initial scan complete. Watching for changes in l10n directory...");

    let l10n_dir = p.root_dir.join(&p.l10n_dir);
    for _ in DirWatcher::new(&l10n_dir)?.flatten() {
        std::thread::sleep(std::time::Duration::from_millis(500)); // Debounce
        
        let jobs = find_untranslated_strings(&p)?;
        if !jobs.is_empty() {
            println!("[translator] Found {} new job(s). Translating in parallel...", jobs.len());
            jobs.into_par_iter().for_each(|job| {
                println!("[translator] Translating '{}' to {}...", job.key, job.lang);
                 match run_translate_script(&job.text, &job.lang) {
                    Ok(translated_text) => {
                        if let Err(e) = job.arb_file.add_key(&job.key, &translated_text) {
                            println!("[translator] ERROR: Failed to write translation for key '{}': {}", job.key, e);
                        }
                    }
                    Err(e) => {
                        println!("[translator] ERROR: Failed to translate key '{}': {}", job.key, e);
                    }
                }
            });
        }
    }
    Ok(())
}

pub fn spawn(p: Project) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = start(p) {
            println!("[translator] Worker thread terminated with error: {{e}}");
        }
    })
}
