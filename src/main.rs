mod extractor;
mod project;
mod utils;

fn main() -> Result<(), String> {
    println!("arb-util");
    let p = project::Project::load()?;
    println!("{p:#?}");
    Ok(())
}
