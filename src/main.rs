mod arb;
mod extractor;
mod project;
mod syncer;
mod translator;
mod utils;
mod watcher;

fn main() -> Result<(), String> {
    println!("arb-util");
    let p = project::Project::load()?;
    println!("{p:#?}");

    let extractor_handle = extractor::spawn(p.clone());
    let syncer_handle = syncer::spawn(p.clone());
    let translator_handle = translator::spawn(p.clone());

    extractor_handle.join().expect("Extractor thread panicked");
    syncer_handle.join().expect("Syncer thread panicked");
    translator_handle.join().expect("Translator thread panicked");
    
    Ok(())
}
