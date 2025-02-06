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
    ver: String,
    /// The name of the user-agent. Automation tools
    /// should specify this variable to specify which
    /// user generates the requests.
    #[arg(
        short,
        long,
        default_value_t = format!("crate2bib-cli-user-agent"),
        verbatim_doc_comment
    )]
    user_agent: String,
    #[arg(long, default_values_t = [
        "CITATION.cff".to_string(),
        "citation.bib".to_string(),
    ])]
    filenames: Vec<String>,
    #[arg(short, long, default_value = "")]
    branch_name: String,
}

#[async_std::main]
async fn main() -> crate2bib::Result<()> {
    let args = Args::parse();
    let filenames = args.filenames.iter().map(|x| x.as_str()).collect();
    let results = crate2bib::get_biblatex(
        &args.crate_name,
        if args.ver.is_empty() {
            None
        } else {
            Some(&args.ver)
        },
        if args.user_agent.is_empty() {
            None
        } else {
            Some(&args.user_agent)
        },
        if args.branch_name.is_empty() {
            None
        } else {
            Some(&args.branch_name)
        },
        filenames,
    )
    .await?;

    for result in results {
        match result {
            crate2bib::BibLaTeX::CITATIONCFF(ref b) => {
                println!(
                    "Generated from CITATION.cff file in repository {}",
                    b.repository
                        .as_ref()
                        .map_or("".to_string(), |x| format!("{x}"))
                )
            }
            crate2bib::BibLaTeX::CratesIO(_) => {
                println!("Generated enty from crates.io information")
            }
            #[allow(unused)]
            crate2bib::BibLaTeX::Plain(crate2bib::PlainBibLaTeX {
                ref bibliography,
                ref repository,
                ref filename,
            }) => {
                println!(
                    "Obtained bibliography {filename} file directly from repository {repository}"
                )
            }
        }
        println!("{result}");
    }
    Ok(())
}
