mod arb;
mod extractor;
mod project;
mod syncer;
mod translator;
mod utils;
mod watcher;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), String> {
    dotenv().ok();
    println!("arb-util");
    let p = project::Project::load()?;
    println!("{p:#?}");

    let extractor_handle = tokio::spawn(extractor::run(p.clone()));
    let syncer_handle = tokio::spawn(syncer::run(p.clone()));
    let translator_handle = tokio::spawn(translator::run(p.clone()));

    extractor_handle
        .await
        .expect("Extractor task failed")
        .expect("Extractor task failed");
    syncer_handle
        .await
        .expect("Extractor async task failed")
        .expect("Syncer task failed");
    translator_handle
        .await
        .expect("Extractor async task failed")
        .expect("Translator task failed");

    Ok(())
}
