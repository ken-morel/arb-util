use crate::{arb::ArbFile, project::Project, watcher::DirWatcher};
use std::{
    io::{ Write},
    process::{Command, Stdio},
};

#[derive(Debug)]
struct TranslationJob {
    key: String,
    text: String,
    lang: String,
    arb_file: ArbFile,
}

fn translate(text: &str, lang: &str) -> Result<String, String> {
    let mut child = Command::new("translate")
        .arg(lang)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not launch translator {e}"))?;
    
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(text.as_bytes()).expect("Failed to write to stdin");
        // stdin is closed as it is dropped
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    Ok(String::from_utf8_lossy(&output.stdout).to_string().trim().to_string())
}

/// Scans all auxiliary ARB files and collects a list of strings that need translation.
fn find_untranslated_strings(project: &Project) -> Result<Vec<TranslationJob>, String> {
    let mut jobs = Vec::new();
    let l10n_dir = project.root_dir.join(&project.l10n_dir);
    let template_path = project.arb_template_path();

    for entry in std::fs::read_dir(l10n_dir)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let path = entry.path();
        if path == template_path || path.extension().is_none_or(|ext| ext != "arb") {
            continue;
        }

        let lang = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split('_')
            .next_back()
            .unwrap_or_default();

        if lang.is_empty() {
            continue;
        }

        let arb_file = ArbFile::new(path.clone());
        match arb_file.read() {
            Ok(obj) => {
                for (key, value) in obj {
                            if let Some(s_val) = value.as_str() && s_val.starts_with('#') {
                                    jobs.push(TranslationJob {
                                        key: key.clone(),
                                        text: s_val.strip_prefix('#').unwrap().to_string(),
                                        lang: lang.to_string(),
                                        arb_file: ArbFile::new(arb_file.path.clone()),
                                    });
                    
                            }
                        }
            }
            Err(e) => println!("[translator] Error reading arb file {e}"),
        }
    }
    Ok(jobs)
}

/// The main translation loop.
pub fn start(p: Project) -> Result<(), String> {
    println!("[translator] Started. Performing initial scan for untranslated strings...");

    let initial_jobs = find_untranslated_strings(&p)?;
        println!(
            "[translator] Found {} initial job(s). Translating in parallel...",
            initial_jobs.len()
        );
    if !initial_jobs.is_empty() {
        
        initial_jobs.into_iter().for_each(|job| {
            match translate(&job.text, &job.lang) {
                Ok(translated_text) => {
                    println!("[translator] Translated {} to {}...", job.key, job.lang);
                    if let Err(e) = job.arb_file.add_key(&job.key, &translated_text) {
                        println!(
                            "[translator] ERROR: Failed to write translation for key '{}': {}",
                            job.key, e
                        );
                        
                    }
                }
                Err(e) => {
                    println!(
                        "[translator] ERROR: Failed to translate key '{}': {}",
                        job.key, e
                    );
                }
            }
        });
    }

    println!("[translator] Initial scan complete. Watching for changes in l10n directory...");

    let l10n_dir = p.root_dir.join(&p.l10n_dir);
    for _ in DirWatcher::new(&l10n_dir)?.flatten() {
        std::thread::sleep(std::time::Duration::from_millis(1000)); // Debounce

        let jobs = find_untranslated_strings(&p)?;
        if !jobs.is_empty() {
            println!(
                "[translator] Found {} new job(s). Translating in parallel...",
                jobs.len()
            );
            jobs.into_iter().for_each(|job| {
                println!("[translator] Translating '{}' to {}", job.key, job.lang);
                match translate(&job.text, &job.lang) {
                    Ok(translated_text) => {
                        if let Err(e) = job.arb_file.add_key(&job.key, &translated_text) {
                            println!(
                                "  [translator] ERROR: Failed to write translation for key '{}': {}",
                                job.key, e
                            );
                        }
                    }
                    Err(e) => {
                        println!(
                            "  [translator] ERROR: Failed to translate key '{}': {}",
                            job.key, e
                        );
                    }
                }
            });
            println!("[translator] Translation completed");
        }
    }
    Ok(())
}

pub fn spawn(p: Project) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = start(p) {
            println!("[translator] Worker thread terminated with error: {e}");
        }
    })
}
