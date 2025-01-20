//! Search and create BibLaTeX entries for crates hosted on [crates.io](https://crates.io).
//!
//! This crate is centered around the [get_biblatex] function.
//! It works in multiple steps.
//! 1. Query [crates.io](https://crates.io) and obtain information about crate
//! 2. Search repository for possible `CITATION` files
//! 3. Generate BibLaTeX entry from known properties and return the Origin of this information via
//!    [EntryOrigin]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use chrono::Datelike;
use serde::{de::Error, Deserialize, Serialize};

/// A fully specified BibLaTeX entry generated from a crate hostedn on [crates.io](https://crates.io)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BibLaTeX {
    /// BibLaTeX citation key which can be used in LaTeX `\cite{key}`.
    pub key: String,
    /// All authors of the crate.
    pub author: String,
    /// The title of the crate is a combination of the name, version and description of the crate
    pub title: String,
    /// Contains the repository where the crate is hosted
    pub url: Option<String>,
    /// Version which was automatically found by [semver]
    pub version: semver::Version,
    /// The time at which this version was published
    pub date: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for BibLaTeX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Writes the biblatex entry
        writeln!(f, "@software {{{}", self.key)?;
        writeln!(f, "    author = {{{}}},", self.author)?;
        writeln!(f, "    title = {{{}}},", self.title)?;
        if let Some(u) = &self.url {
            writeln!(f, "    url = {{{u}}},")?;
        };
        writeln!(
            f,
            "    date = {{{:4.0}-{:02}-{:02}}},",
            self.date.year(),
            self.date.month(),
            self.date.day(),
        )?;
        // Closes the entry
        write!(f, "}}")?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct VersionError(String);

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl std::error::Error for VersionError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "Could not find given version number comparator"
    }
}

async fn search_citation_cff(
    client: &reqwest::Client,
    repository: &Option<String>,
) -> Result<Option<BibLaTeX>, Box<dyn std::error::Error>> {
    if let Some(repo) = repository {
        // Check if this is Github
        if !repo.contains("github") {
            return Ok(None);
        }

        // Make API Call
        // See: https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#get-a-repository
        // https://github.com/OWNER/REPO
        // -> https://api-github.com/repos/OWNER/REPO
        let segments: Vec<_> = repo.split("github.com/").collect();
        println!("\n\n");
        println!("{segments:?}");
        if let Some(tail) = segments.get(1) {
            let segments2: Vec<_> = tail.split("/").collect();
            println!("{segments2:?}");
            let owner = segments2[0];
            let repo = segments2[1];
            println!("{owner} {repo}");
            let request_url = format!("https://api.github.com/repos/{owner}/{repo}");
            let response = client
                .get(request_url)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;
            let default_branch = response
                .get("default_branch")
                .ok_or(serde_json::Error::custom("could not find default branch"))?;
            // println!("{response:#}");
            println!("{default_branch}");
            println!("\n\n");
        }
        // panic!();
    }
    Ok(None)
}

/// Describes how the BibLaTeX entry was obtainedj
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub enum EntryOrigin {
    /// Generated from data found on [crates.io](https://crates.io)
    Generated = 0,
    /// Found citation file in repository
    CitationCff = 1,
}

/// Returns a [BibLaTeX] entry for the searched crate.
///
/// ## Note
/// crates.io requires the specification of a user-agent
/// but this may yield errors when calling from a static website due to CORS.
pub async fn get_biblatex(
    crate_name: &str,
    version: &str,
    user_agent: Option<&str>,
) -> Result<(BibLaTeX, EntryOrigin), Box<dyn std::error::Error>> {
    use crates_io_api::AsyncClient;
    use reqwest::header::*;
    let mut headers = HeaderMap::new();

    if let Some(ua) = user_agent {
        headers.insert(USER_AGENT, HeaderValue::from_str(ua)?);
    }

    let client1 = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let client =
        AsyncClient::with_http_client(client1.clone(), web_time::Duration::from_millis(1000));
    let info = client.get_crate(crate_name).await?;
    let mut obtained_versions = info
        .versions
        .iter()
        .enumerate()
        .filter_map(|(n, x)| semver::Version::parse(&x.num).ok().map(|y| (n, y)))
        .collect::<Vec<_>>();
    obtained_versions.sort_by_key(|x| x.1.clone());
    obtained_versions.reverse();
    let version = semver::Comparator::parse(version)?;
    let (index, found_version_semver) = obtained_versions
        .into_iter()
        .find(|x| version.matches(&x.1))
        .ok_or(VersionError(format!("Could not find {}", version)))?;
    let found_version = info.versions[index].clone();

    if let Some(bibtex) = search_citation_cff(&client1, &info.crate_data.repository).await? {
        return Ok((bibtex, EntryOrigin::CitationCff));
    }

    Ok((
        BibLaTeX {
            key: format!("{}{}", crate_name, info.crate_data.updated_at.year()),
            author: found_version
                .published_by
                .map_or_else(|| "".to_owned(), |x| x.name.unwrap_or(x.login)),
            title: info
                .crate_data
                .description
                .map_or(crate_name.to_owned(), |x| {
                    format!("{{{}}} ({{{}}}): {}", crate_name, found_version_semver, x)
                }),
            url: info.crate_data.repository,
            version: found_version_semver,
            date: found_version.updated_at,
        },
        EntryOrigin::Generated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn access_crates_io() -> Result<(), Box<dyn std::error::Error>> {
        let bib_entry = get_biblatex("serde", "1.0.217", Some("crate2bib-testing")).await?;
        let expected = "\
@software {serde2024
    author = {David Tolnay},
    title = {serde (1.0.217): A generic serialization/deserialization framework},
    url = {https://github.com/serde-rs/serde},
    date = {2024-12-27},
}";
        assert_eq!(format!("{}", bib_entry.0), expected);
        assert_eq!(bib_entry.1, EntryOrigin::Generated);
        Ok(())
    }
}
