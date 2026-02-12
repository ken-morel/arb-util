use super::{arb::ArbFile, project::Project, watcher::DirWatcher};
use tokio::{sync::mpsc::channel,time::sleep};


#[derive(Debug)]
struct TranslationJob {
    key: String,
    text: String,
    lang: String,
    arb_file: ArbFile,
}



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


pub async fn run(p: Project) -> Result<(), String> {
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(k) => k,
        Err(e) => {
            println!("[translator] #################### GEMINI rate limit exceded ####################");
            print!("\x07");
            println!("[translator]                     Closing translation job");
            return Err(format!("{e}"));
        }

    };
    println!("[translator] Translator started, making initial run");
    let l10n_dir = p.root_dir.join(&p.l10n_dir);

    let mut watcher =DirWatcher::new(&l10n_dir, true)?;

    while watcher.next().await.is_some() {
        sleep(std::time::Duration::from_millis(5000)).await;

        let jobs = find_untranslated_strings(&p)?;
        if jobs.is_empty() {
            continue;
        }

        println!("[translator] Found {} new job(s). Translating in parallel...", jobs.len());

        let (tx, mut rx) = channel::<(TranslationJob, String)>(100);

        let writer_handle = tokio::spawn(async move {
            let mut count = 0;
            while let Some((job, translated_text)) = rx.recv().await {
                match job.arb_file.add_key(&job.key, &translated_text) {
                    Ok(_) => count += 1,
                    Err(e) => println!(
                        "  [translator] ERROR: Failed to write key '{}': {}",
                        job.key, e
                    ),
                }
            }
            println!("[translator] Written {} translations to disk.", count);
        });

        for job in jobs.into_iter() {
            let api_key = api_key.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                println!("[translator] Translating '{}' to {}", job.key, job.lang);

                // Do the heavy lifting (Network I/O)
                match translate(&api_key ,&job.text, &job.lang).await {
                    TranslateResult::Translated(translated_text) => {
                        if tx.send((job, translated_text)).await.is_err() {
                            eprintln!("[translator] Failed to send result to writer");
                        }
                    }
                    TranslateResult::Error(e) => {
                        println!(
                            "  [translator] ERROR: Failed to translate key '{}': {}",
                            job.key, e
                        );
                    }
                    TranslateResult::RateLimitExceeded => {}
                }
            });
        }

        drop(tx);

        if let Err(e) = writer_handle.await {
            println!("[translator] Writer task panicked: {}", e);
        }

        println!("[translator] Batch completed");
    }
    Ok(())
}



use std::env;
use reqwest::Client;
use serde_json::{json, Value};

pub enum TranslateResult {
    Translated(String),
    Error(String),
    RateLimitExceeded,
}
pub async fn translate(api_key: &str,txt: &str, lang: &str) -> TranslateResult {
    // Retrieve the Gemini API key from environment variables


    let client = Client::new();
    let api_url = "https://generativelanguage.googleapis.com/v1beta/openai/v1/chat/completions";
    let model_name = "gemini-2.5-flash-lite";

    // Improved system prompt for nuanced translation
    let system_prompt = format!(
        "You are a highly skilled and nuanced language translation AI. Your task is to accurately and idiomatically translate the provided text into {}.
        1. Source Language Detection: Automatically detect the source language of the input text.
        2. Context and Nuance: Preserve the original meaning, tone, and cultural nuances of the text as much as possible.
        3. Output Format: Provide ONLY the full, translated text. Do not include any conversational filler, explanations, quotes around the output, or additional
formatting. Ensure the output is clean and ready for direct use.",
        lang
    );

    // Construct the JSON request body
    let request_body = json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": txt}
        ],
        "temperature": 0.1, // Lower temperature for more deterministic translation
        "max_tokens": 1024  // Limit output tokens to prevent overly verbose responses
    });

    // Make the POST request to the Gemini API
    let response = match client.post(api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await {
            Err(e) => return TranslateResult::Error(format!("Failed to send request to Gemini API: {}", e)),
            Ok(r) => r,
    };

    // Check for HTTP errors
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown API error".to_string());
        if status == 429 && error_text.contains("You exceeded your current quota, please check your plan and billing details. For more information on this error, head to: https://ai.google.dev/gemini-api/docs/rate-limits") {
            return TranslateResult::RateLimitExceeded;
        } else {
            return TranslateResult::Error(format!("Gemini API returned an error status: {} - {}", status, error_text));

        }
    }

    // Parse the JSON response
    let response_body: Value = match response.json().await {
        Err(e) => return TranslateResult::Error(format!("Failed to parse Gemini API response as JSON: {}", e)),
        Ok(v) => v,
    };


    // Extract the translated text from the response
    if let Some(translated_text) = response_body["choices"][0]["message"]["content"].as_str() {
        TranslateResult::Translated(translated_text.to_string())
    } else {
        TranslateResult::Error(format!("Could not find translated content in API response: {}", response_body))
    }
}
