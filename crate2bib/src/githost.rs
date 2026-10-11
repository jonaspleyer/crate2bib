use std::future::Future;

use crate::{BibLaTeX, PlainBibLaTeX};

/// A git hosting service that can be scanned for citation files.
///
/// Every supported service is simply one variant of this enum. None of them is treated as more
/// important than the others. Adding support for a new git server therefore only requires adding a
/// new variant and filling in the matching arms of the methods below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GitHost {
    /// [github.com](https://github.com)
    GitHub,
    /// [codeberg.org](https://codeberg.org) (powered by Forgejo/Gitea)
    Codeberg,
}

/// Describes how the raw contents of a file are encoded in a git host's response.
#[derive(Clone, Copy)]
enum ContentEncoding {
    /// The response body already is the raw file content (e.g. `raw.githubusercontent.com`).
    Raw,
    /// The response body is a JSON object with a base64-encoded `content` field, as returned by the
    /// Gitea/Forgejo contents API.
    GiteaJsonBase64,
}

impl GitHost {
    /// All git hosts known to this crate.
    const ALL: &'static [GitHost] = &[GitHost::GitHub, GitHost::Codeberg];

    /// The domain (including a trailing slash) identifying this host inside a repository URL.
    fn domain(&self) -> &'static str {
        match self {
            GitHost::GitHub => "github.com/",
            GitHost::Codeberg => "codeberg.org/",
        }
    }

    /// Detects which [GitHost] a repository URL belongs to, if any.
    fn detect(repository: &str) -> Option<Self> {
        GitHost::ALL
            .iter()
            .copied()
            .find(|host| repository.contains(host.domain()))
    }

    /// URL of the API endpoint returning repository metadata (used to determine the default branch).
    fn api_repo_url(&self, owner: &str, repo: &str) -> String {
        match self {
            GitHost::GitHub => format!("https://api.github.com/repos/{owner}/{repo}"),
            GitHost::Codeberg => format!("https://codeberg.org/api/v1/repos/{owner}/{repo}"),
        }
    }

    /// URL from which the raw contents of `filename` on `branch_name` can be retrieved.
    fn content_url(&self, owner: &str, repo: &str, branch_name: &str, filename: &str) -> String {
        match self {
            GitHost::GitHub => format!(
                "https://raw.githubusercontent.com/\
                    {owner}/\
                    {repo}/\
                    refs/heads/\
                    {branch_name}/\
                    {filename}"
            ),
            GitHost::Codeberg => format!(
                "https://codeberg.org/api/v1/repos/\
                    {owner}/\
                    {repo}/\
                    contents/\
                    {filename}/\
                    ?ref={branch_name}"
            ),
        }
    }

    /// How the content returned by [GitHost::content_url] is encoded.
    fn content_encoding(&self) -> ContentEncoding {
        match self {
            GitHost::GitHub => ContentEncoding::Raw,
            GitHost::Codeberg => ContentEncoding::GiteaJsonBase64,
        }
    }
}

/// Extracts the plain file content from a host's response according to the given [ContentEncoding].
///
/// Returns `Ok(None)` if the file does not exist.
async fn extract_content(
    response: impl Future<Output = Result<reqwest::Response, reqwest::Error>>,
    encoding: ContentEncoding,
    repository: &str,
    filename: &str,
) -> crate::Result<Option<String>> {
    match encoding {
        ContentEncoding::Raw => {
            let text = response.await?.text().await?;
            if text.to_lowercase().trim() == "404: not found" {
                #[cfg(feature = "log")]
                log::warn!(
                    "Could not find file \"{filename}\" in repository \"{repository}\". \
                        Skipping this file.",
                );
                let _ = (repository, filename);
                Ok(None)
            } else {
                Ok(Some(text))
            }
        }
        ContentEncoding::GiteaJsonBase64 => {
            use base64::Engine;
            let json = response.await?.json::<serde_json::Value>().await?;
            if let Some(content) = json.get("content") {
                let content = content.as_str().unwrap_or_default();
                let bytes = base64::prelude::BASE64_STANDARD.decode(content)?;
                Ok(Some(String::from_utf8(bytes).unwrap_or_default()))
            } else {
                Ok(None)
            }
        }
    }
}

async fn response_to_biblatex(
    client: reqwest::Client,
    response: impl Future<Output = Result<reqwest::Response, reqwest::Error>>,
    repository: String,
    filename: String,
    search_doi: bool,
    encoding: ContentEncoding,
) -> crate::Result<Vec<crate::Result<crate::BibLaTeX>>> {
    let text = match extract_content(response, encoding, &repository, &filename).await? {
        Some(text) => text,
        None => return Ok(vec![]),
    };
    let chunks: Vec<_> = filename.split(".").collect();
    let extension = chunks.get(1);
    #[cfg(feature = "log")]
    log::trace!("Checking file extensions in repository");
    let mut results = vec![];
    match extension {
        Some(&"bib") => results.push(
            biblatex::Bibliography::parse(&text)
                // .map_err(crate::Err::BibLaTeXParsing)
                .map(|x| {
                    BibLaTeX::Plain(PlainBibLaTeX {
                        bibliography: crate::BibReturn::BibFile(x),
                        repository,
                        filename,
                    })
                })
                .map_err(crate::Err::from),
        ),
        Some(&"cff") => {
            // Try to obtain plain BibLaTeX entry from doi
            let citation_cff = citeworks_cff::from_str(&text);
            if search_doi {
                if let Some(doi) = citation_cff
                    .as_ref()
                    .ok()
                    .and_then(|x| x.preferred_citation.as_ref().and_then(|p| p.doi.as_ref()))
                {
                    match crate::get_bibtex_doi(doi, client).await {
                        Ok(bib) => results.push(Ok(crate::BibLaTeX::Plain(PlainBibLaTeX {
                            bibliography: bib,
                            repository,
                            filename,
                        }))),
                        Err(e) => {
                            #[cfg(feature = "log")]
                            log::warn!("Received error: \"{e}\" during doi.org request.");
                            let _ = e;
                        }
                    }
                }
            }

            results.push(
                citation_cff
                    .map(BibLaTeX::CITATIONCFF)
                    .map_err(crate::Err::from),
            )
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

/// Searches a git-hosted repository for citation files.
///
/// The git hosting service is detected automatically from the repository URL. Currently
/// [github.com](https://github.com) and [codeberg.org](https://codeberg.org) are supported and all
/// treated equally.
pub async fn githost_search_files(
    client: &reqwest::Client,
    repository: &str,
    filenames: Vec<&str>,
    branch_name: Option<&str>,
    search_doi: bool,
) -> crate::Result<Vec<crate::Result<crate::BibLaTeX>>> {
    let host = match GitHost::detect(repository) {
        Some(host) => host,
        None => {
            #[cfg(feature = "log")]
            {
                log::warn!("Cannot query {repository}");
                log::warn!("Currently only github & codeberg repositories are supported.");
            }
            return Ok(vec![]);
        }
    };

    if filenames.is_empty() {
        #[cfg(feature = "log")]
        log::info!("Did not find any matching filenames");
        return Ok(Vec::new());
    }

    let mut results = vec![];
    let segments: Vec<_> = repository.split(host.domain()).collect();
    if let Some(tail) = segments.get(1) {
        let segments2: Vec<_> = tail.split("/").collect();
        let owner = segments2.first();
        let repo = segments2.get(1);
        if let (Some(repo), Some(owner)) = (repo, owner) {
            // If a branch name was specified we search there and nowhere else
            let branch_name = if let Some(branch_name) = branch_name {
                branch_name.to_string()
            } else {
                let respose = client
                    .get(host.api_repo_url(owner, repo))
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
                let rq = host.content_url(owner, repo, &branch_name, filename);
                #[cfg(feature = "log")]
                log::trace!("Requesting {} information for file \"{rq}\"", host.domain());
                let file_content = client.get(&rq).send();
                #[cfg(feature = "log")]
                log::trace!("Converting response to BibLaTeX");
                let r = response_to_biblatex(
                    client.clone(),
                    file_content,
                    repository.to_string(),
                    filename.to_string(),
                    search_doi,
                    host.content_encoding(),
                )
                .await?;
                results.extend(r);
            }
        }
    }
    Ok(results)
}
