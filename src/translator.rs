use super::{arb::ArbFile, project::Project, watcher::DirWatcher};
use std::
    sync::mpsc::channel

;
use jemini::{JeminiClient };



#[derive(Debug)]
struct TranslationJob {
    key: String,
    text: String,
    lang: String,
    arb_file: ArbFile,
}

pub async fn translate(text: &str, lang: &str) -> Result<String, String> {
  let client = JeminiClient::new().map_err(|e| format!("Could not create JeminiClient: {e}"))?;
  let system = format!(
      "You are a highly efficient and accurate language translator.
      Translate the following text into {}.
      Provide only the translated text,
      without any additional conversational content or explanations.",
      lang,
  );
  let response  = client.text_only(&(system + "\n\n" + text)).await.map_err(|e| format!("Could not query gemini: {e}"))?;
  dbg!(&response);
  match response.most_recent() {
      Some(t) => Ok(String::from(t)),
      None => Err(String::from("Jemini sent no response"))
  }

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
pub async fn run(p: Project) -> Result<(), String> {
    println!("[translator] Started. Performing initial scan for untranslated strings...");

    let initial_jobs = find_untranslated_strings(&p)?;
        println!(
            "[translator] Found {} initial job(s). Translating in parallel...",
            initial_jobs.len()
        );
    if !initial_jobs.is_empty() {
        for job in initial_jobs.into_iter() {
            match translate(&job.text, &job.lang).await {
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
        }
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
            let (tx, rx) = channel::<(TranslationJob, String)>();
            let write_thread_handle = std::thread::spawn(move || {
                for (job, translated_text) in rx {
                    if let Err(e) = job.arb_file.add_key(&job.key, &translated_text) {
                        println!(
                            "  [translator] ERROR: Failed to write translation for key '{}': {}",
                            job.key, e
                        );
                    }
                }
            });
            for job in jobs.into_iter() {
                println!("[translator] Translating '{}' to {}", job.key, job.lang);
                match translate(&job.text, &job.lang).await {
                    Ok(translated_text) => {
                        _ = tx.send((job, translated_text));

                    }
                    Err(e) => {
                        println!(
                            "  [translator] ERROR: Failed to translate key '{}': {}",
                            job.key, e
                        );
                    }
                }
            }
            if let Err(e) =  write_thread_handle.join() {
                println!("[translator] Write thread had an error: {e:?}");
            }
            println!("[translator] Translation completed");
        }
    }
    Ok(())
}
