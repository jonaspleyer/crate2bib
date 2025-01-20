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
    #[arg(short, long, default_value_t = format!("crate2bib-cli-user-agent"))]
    user_agent: String,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let (biblatex, origin) =
        crate2bib::get_biblatex(&args.crate_name, &args.semver, Some(&args.user_agent)).await?;
    match origin {
        crate2bib::EntryOrigin::CitationCff => {
            println!("Obtained from CITATION file in repository")
        }
        crate2bib::EntryOrigin::Generated => println!("Obtained from crates.io information"),
    }
    println!("{biblatex}");
    Ok(())
}
