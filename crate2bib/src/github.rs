use std::future::Future;

use crate::{BibLaTeX, PlainBibLaTeX};

async fn response_to_biblatex(
    client: reqwest::Client,
    response: impl Future<Output = Result<reqwest::Response, reqwest::Error>>,
    repository: String,
    filename: String,
    search_doi: bool,
) -> crate::Result<Vec<crate::BibLaTeX>> {
    let text = response.await?.text().await?;
    if text.to_lowercase().trim() == "404: not found" {
        #[cfg(feature = "log")]
        log::warn!(
            "Could not find file \"{filename}\" in repository \"{repository}\". \
            Skipping this file.",
        );
        return Ok(vec![]);
    }
    let chunks: Vec<_> = filename.split(".").collect();
    let extension = chunks.get(1);
    #[cfg(feature = "log")]
    log::trace!("Checking file extensions in repository");
    let mut results = vec![];
    match extension {
        Some(&"bib") => results.push(BibLaTeX::Plain(PlainBibLaTeX {
            bibliography: biblatex::Bibliography::parse(&text)
                .map_err(crate::Err::BibLaTeXParsing)?,
            repository,
            filename,
        })),
        Some(&"cff") => {
            // Try to obtain plain BibLaTeX entry from doi
            let citation_cff = citeworks_cff::from_str(&text)?;
            if search_doi {
                if let Some(doi) = citation_cff
                    .preferred_citation
                    .as_ref()
                    .and_then(|p| p.doi.as_ref())
                {
                    match crate::get_bibtex_doi(doi, client).await {
                        Ok(Some(bib)) => results.push(crate::BibLaTeX::Plain(PlainBibLaTeX {
                            bibliography: bib,
                            repository,
                            filename,
                        })),
                        Ok(None) => (),
                        Err(e) => {
                            #[cfg(feature = "log")]
                            log::warn!("Received error: \"{e}\" during doi.org request.");
                        }
                    }
                }
            }

            results.push(BibLaTeX::CITATIONCFF(citation_cff))
        }
        None => (),
        Some(x) => {
            return Err(crate::Err::FiletypeUnsupported(format!(
                "the {x} filetype is currently not supported"
            )))
        }
    }

    Ok(results)
}

/// Searches the repository at [github.com](https://github.com) for citation files
pub async fn github_search_files(
    client: &reqwest::Client,
    repository: &str,
    filenames: Vec<&str>,
    branch_name: Option<&str>,
    search_doi: bool,
) -> crate::Result<Vec<crate::BibLaTeX>> {
    // Check if this is Github
    if !repository.contains("github") {
        #[cfg(feature = "log")]
        log::warn!("Cannot query {repository}");
        #[cfg(feature = "log")]
        log::warn!("Currently only github repositories are supported.");
        return Ok(vec![]);
    }
    if filenames.is_empty() {
        #[cfg(feature = "log")]
        log::info!("Did not find any matching filenames");
        return Ok(Vec::new());
    }

    let mut results = vec![];
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
                    #[cfg(feature = "log")]
                    log::trace!("Determined default branch {default_branch}");
                    default_branch.to_string().replace("\"", "")
                } else {
                    #[cfg(feature = "log")]
                    log::info!("Automatically chose default branch \"main\"");
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
            for filename in filenames.iter() {
                let rq = format!("{request_url_base}/{filename}");
                #[cfg(feature = "log")]
                log::trace!("Requesting github information for file \"{rq}\"");
                let file_content = client.get(&rq).send();
                #[cfg(feature = "log")]
                log::trace!("Converting response to BibLaTeX");
                let r = response_to_biblatex(
                    client.clone(),
                    file_content,
                    repository.to_string(),
                    filename.to_string(),
                    search_doi,
                )
                .await?;
                results.extend(r);
            }
        }
    }
    Ok(results)
}
