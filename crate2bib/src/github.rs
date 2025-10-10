use std::future::Future;

use crate::{BibLaTeX, PlainBibLaTeX};

#[derive(Clone, Copy)]
enum Host {
    Github,
    Codeberg,
}

impl Host {
    fn as_str(&self) -> &str {
        match self {
            Host::Github => "github.com/",
            Host::Codeberg => "codeberg.org/",
        }
    }
}

async fn response_to_biblatex(
    client: reqwest::Client,
    response: impl Future<Output = Result<reqwest::Response, reqwest::Error>>,
    repository: String,
    filename: String,
    search_doi: bool,
    host: Host,
) -> crate::Result<Vec<crate::BibLaTeX>> {
    let text = match host {
        Host::Github => {
            let text = response.await?.text().await?;
            if text.to_lowercase().trim() == "404: not found" {
                #[cfg(feature = "log")]
                log::warn!(
                    "Could not find file \"{filename}\" in repository \"{repository}\". \
                        Skipping this file.",
                );
                return Ok(vec![]);
            }
            text
        }
        Host::Codeberg => {
            use base64::Engine;
            let json = response.await?.json::<serde_json::Value>().await?;
            if let Some(content) = json.get("content") {
                let content = content.as_str().unwrap_or_default();
                let bytes = base64::prelude::BASE64_STANDARD.decode(content).unwrap();
                String::from_utf8(bytes).unwrap()
            } else {
                return Ok(vec![]);
            }
        }
    };
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
    let (host, api_url) = if repository.contains("github.com/") {
        (Host::Github, "https://api.github.com/repos")
    } else if repository.contains("codeberg.org/") {
        (Host::Codeberg, "https://codeberg.org/api/v1/repos")
    } else {
        #[cfg(feature = "log")]
        log::warn!("Cannot query {repository}");
        #[cfg(feature = "log")]
        log::warn!("Currently only github & codeberg repositories are supported.");
        return Ok(vec![]);
    };

    let content_url_formatter = |owner, repo, branch_name, filename| {
        if repository.contains("github") {
            format!(
                "https://raw.githubusercontent.com/\
                    {owner}/\
                    {repo}/\
                    refs/heads/\
                    {branch_name}/\
                    {filename}"
            )
        } else {
            format!(
                "https://codeberg.org/api/v1/repos/\
                    {owner}/\
                    {repo}/\
                    contents/\
                    {filename}/\
                    ?ref={branch_name}"
            )
        }
    };

    if filenames.is_empty() {
        #[cfg(feature = "log")]
        log::info!("Did not find any matching filenames");
        return Ok(Vec::new());
    }

    let mut results = vec![];
    let segments: Vec<_> = repository.split(host.as_str()).collect();
    if let Some(tail) = segments.get(1) {
        let segments2: Vec<_> = tail.split("/").collect();
        let owner = segments2.first();
        let repo = segments2.get(1);
        if let (Some(repo), Some(owner)) = (repo, owner) {
            let request_url = format!("{api_url}/{owner}/{repo}");

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

            for filename in filenames.iter() {
                let rq = content_url_formatter(owner, repo, &branch_name, filename);
                #[cfg(feature = "log")]
                log::trace!("Requesting {} information for file \"{rq}\"", host.as_str());
                let file_content = client.get(&rq).send();
                #[cfg(feature = "log")]
                log::trace!("Converting response to BibLaTeX");
                let r = response_to_biblatex(
                    client.clone(),
                    file_content,
                    repository.to_string(),
                    filename.to_string(),
                    search_doi,
                    host,
                )
                .await?;
                results.extend(r);
            }
        }
    }
    Ok(results)
}
