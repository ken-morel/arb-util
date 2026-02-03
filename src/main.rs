mod arb;
mod extractor;
mod project;
mod syncer;
mod utils;
mod watcher;

fn main() -> Result<(), String> {
    println!("arb-util");
    let p = project::Project::load()?;
    println!("{p:#?}");

    // let arb_mutex = std::sync::Arc::new(std::sync::Mutex::new(0));

    let eh = extractor::spawn(p.clone());
    let es = syncer::spawn(p.clone());

    eh.join().expect("Extractor error");
    es.join().expect("Syncer error");
    Ok(())
}
