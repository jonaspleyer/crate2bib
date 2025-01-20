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

impl std::fmt::Display for BibTex {
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

/// Returns a [BibLaTeX] entry for the searched crate.
///
/// ## Note
/// crates.io requires the specification of a user-agent
/// but this may yield errors when calling from a static website due to CORS.
pub async fn get_bibtex(
    crate_name: &str,
    version: &str,
    user_agent: Option<&str>,
) -> Result<BibTex, Box<dyn std::error::Error>> {
    use crates_io_api::AsyncClient;
    use reqwest::header::*;
    let mut headers = HeaderMap::new();

    if let Some(ua) = user_agent {
        headers.insert(USER_AGENT, HeaderValue::from_str(ua)?);
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let client = AsyncClient::with_http_client(client, web_time::Duration::from_millis(1000));
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

    // TODO look into the repository and see if we can find any of the following:
    // 1. file: CITATION.cff/CITATION
    // 2. file: README.md/README containing any of the following words:
    //      1. citation
    //      2. cite
    //      3. reference

    Ok(BibTex {
        key: format!("{}{}", crate_name, info.crate_data.updated_at.year()),
        author: found_version
            .published_by
            .map_or_else(|| "".to_owned(), |x| x.name.unwrap_or(x.login)),
        title: info
            .crate_data
            .description
            .map_or(crate_name.to_owned(), |x| {
                format!("{} ({}): {}", crate_name, found_version_semver, x)
            }),
        url: info.crate_data.repository,
        version: found_version_semver,
        date: found_version.updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn access_crates_io() -> Result<(), Box<dyn std::error::Error>> {
        let bib_entry = get_bibtex("serde", "1.0.217", Some("crate2bib-testing")).await?;
        println!("{}", bib_entry);
        let expected = "\
@software {serde2024
    author = {David Tolnay},
    title = {serde (1.0.217): A generic serialization/deserialization framework},
    url = {https://github.com/serde-rs/serde},
    date = {2024-12-27},
}";
        assert_eq!(format!("{}", bib_entry), expected);
        Ok(())
    }
}
