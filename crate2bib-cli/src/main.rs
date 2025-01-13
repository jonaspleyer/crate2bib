use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Creates a BibTeX entry given a crate name and version number.\
        Note: This crate respects semver.",
    long_about = None
)]
struct Args {
    crate_name: String,
    semver: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let bibtex = crate2bib::get_bibtex(&args.crate_name, &args.semver)?;
    println!("Generated bibtex:");
    println!("{bibtex}");
    Ok(())
}
