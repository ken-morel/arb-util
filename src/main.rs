mod extractor;
mod project;
mod utils;
mod watcher;

fn main() -> Result<(), String> {
    println!("arb-util");
    let p = project::Project::load()?;
    println!("{p:#?}");
    extractor::spawn(p.clone()).join().expect("Extractor error");
    Ok(())
}
