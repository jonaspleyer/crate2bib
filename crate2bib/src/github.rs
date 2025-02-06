use std::future::Future;

use crate::{BibLaTeX, PlainBibLaTeX};

async fn response_to_biblatex(
    response: impl Future<Output = Result<reqwest::Response, reqwest::Error>>,
    repository: String,
    filename: String,
) -> crate::Result<Option<crate::BibLaTeX>> {
    let text = response.await?.text().await?;
    if text.to_lowercase().contains("404: not found") {
        return Ok(None);
    }
    let chunks: Vec<_> = filename.split(".").collect();
    let extension = chunks.get(1);
    match extension {
        Some(&"bib") => Ok(Some(BibLaTeX::Plain(PlainBibLaTeX {
            bibliography: biblatex::Bibliography::parse(&text)
                .map_err(crate::Err::BibLaTeXParsing)?,
            repository,
            filename,
        }))),
        Some(&"cff") => Ok(Some(BibLaTeX::CITATIONCFF(citeworks_cff::from_str(&text)?))),
        None => todo!(),
        Some(x) => Err(crate::Err::FiletypeUnsupported(format!(
            "the {x} filetype is currently not supported"
        ))),
    }
}

/// Searches the repository at [github.com] for citation files
pub async fn github_search_files(
    client: &reqwest::Client,
    repository: &str,
    filenames: Vec<&str>,
    branch_name: Option<&str>,
) -> crate::Result<Vec<impl Future<Output = crate::Result<Option<crate::BibLaTeX>>>>> {
    // Check if this is Github
    if !repository.contains("github") {
        return Ok(vec![]);
    }
    if filenames.is_empty() {
        return Ok(Vec::new());
    }

    let segments: Vec<_> = repository.split("github.com/").collect();
    if let Some(tail) = segments.get(1) {
        let segments2: Vec<_> = tail.split("/").collect();
        let owner = segments2.first();
        let repo = segments2.get(1);
        if let (Some(repo), Some(owner)) = (repo, owner) {
            let request_url = format!("https://api.github.com/repos/{owner}/{repo}");

            // If a branch name was specified we search there and nowhere else
            let branch_name = if let Some(branch_name) = branch_name {
                branch_name.to_string()
            } else {
                let respose = client
                    .get(request_url)
                    .send()
                    .await?
                    .json::<serde_json::Value>()
                    .await?;

                if let Some(default_branch) = respose.get("default_branch") {
                    default_branch.to_string().replace("\"", "")
                } else {
                    "main".to_string()
                }
            };

            let request_url_base = format!(
                "https://raw.githubusercontent.com/\
                    {owner}/\
                    {repo}/\
                    refs/heads/\
                    {branch_name}"
            );
            return Ok(filenames
                .into_iter()
                .map(|filename| {
                    let rq = format!("{request_url_base}/{filename}");
                    let file_content = client.get(&rq).send();
                    response_to_biblatex(file_content, rq, filename.to_string())
                })
                .collect());
        }
    }
    Ok(vec![])
}
