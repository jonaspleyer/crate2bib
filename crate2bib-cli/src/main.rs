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
    semver: Option<String>,
    #[arg(short, long, default_value_t = format!("crate2bib-cli-user-agent"))]
    user_agent: String,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let results = crate2bib::get_biblatex(
        &args.crate_name,
        args.semver.as_deref(),
        Some(&args.user_agent),
    )
    .await?;
    for (biblatex, origin) in results {
        match origin {
            crate2bib::EntryOrigin::CitationCff => {
                println!("Obtained from CITATION file in repository")
            }
            crate2bib::EntryOrigin::CratesIO => println!("Obtained from crates.io information"),
        }
        println!("{biblatex}");
    }
    Ok(())
}
