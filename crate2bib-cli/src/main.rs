use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Creates a BibTeX entry given a crate name and version number.\
        Note: This crate respects semver.",
    long_about = None,
    author,
)]
struct Args {
    /// The exact name of the crate. Note that underscores are synonymous as dashes in the API of
    /// crates.io.
    crate_name: String,
    /// A semver compliant version number (eg. "1", 0.1", "0.3.38").
    #[arg(short, long, default_value = "")]
    version: String,
    /// The name of the user-agent. Automation tools
    /// should specify this variable to specify which
    /// user generates the requests.
    #[arg(short, long, default_value_t = format!("crate2bib-cli-user-agent"), verbatim_doc_comment)]
    user_agent: String,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let results = crate2bib::get_biblatex(
        &args.crate_name,
        if args.version.is_empty() {
            None
        } else {
            Some(args.version.as_str())
        },
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
