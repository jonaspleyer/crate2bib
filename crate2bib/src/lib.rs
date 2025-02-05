//! Search and create BibLaTeX entries for crates hosted on [crates.io](https://crates.io).
//!
//! This crate is centered around the [get_biblatex] function.
//! It works in multiple steps.
//! 1. Query [crates.io](https://crates.io) and obtain information about crate
//! 2. Search repository for possible `CITATION` files
//! 3. Generate BibLaTeX entry from known properties and return the Origin of this information via
//!    [EntryOrigin]
//!
//! It can be used in a web version under
//! [jonaspleyer.github.io/crate2bib](https://jonaspleyer.github.io/crate2bib).
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// A fully specified BibLaTeX entry generated from a crate hostedn on [crates.io](https://crates.io)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct BibLaTeXCratesIO {
    /// BibLaTeX citation key which can be used in LaTeX `\cite{key}`.
    pub key: String,
    /// One of BibLaTeX's types. This is usually `software` in our case
    pub work_type: String,
    /// All authors of the crate.
    pub author: String,
    /// The title of the crate is a combination of the name, version and description of the crate
    pub title: String,
    /// Contains the repository where the crate is hosted
    pub url: Option<String>,
    /// The license under which the software is distributed
    pub license: Option<String>,
    /// Version which was automatically found by [semver]
    pub version: Option<semver::Version>,
    /// The time at which this version was published
    pub date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Contains all variants of how a bib entry can be obtained
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BibLaTeX {
    /// Obtained bib entry form [crates.io](https://crates.io)
    CratesIO(BibLaTeXCratesIO),
    /// Obtained bib entry from `CITAIION.cff` inside repository.
    CITATIONCFF(citeworks_cff::Cff),
    /// Obtained bib entry directly from repository.
    Bibliography {
        /// Contains the whole bibliography with all keys.
        bibliography: biblatex::Bibliography,
        /// Link to the repository
        url: String,
        /// Name of the file where the citation was discovered
        filename: String,
    },
}

impl core::fmt::Display for BibLaTeX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BibLaTeX::CratesIO(b) => b.fmt(f),
            BibLaTeX::CITATIONCFF(b) => {
                let bib = BibLaTeXCratesIO::from_citation_cff(b).unwrap();
                bib.fmt(f)
            }
            #[allow(unused)]
            BibLaTeX::Bibliography {
                bibliography,
                url,
                filename,
            } => f.write_str(&bibliography.to_biblatex_string()),
        }
    }
}

impl BibLaTeXCratesIO {
    /// Converts a [CitationCff] file to [BibLaTeX]
    pub fn from_citation_cff(cff: &citeworks_cff::Cff) -> Result<Self, Box<dyn std::error::Error>> {
        #[allow(unused)]
        let citeworks_cff::Cff {
            cff_version,
            message,
            title,
            work_type,
            version,
            commit,
            date_released,
            abstract_text,
            keywords,
            url,
            repository,
            repository_artifact,
            repository_code,
            license,
            license_url,
            authors,
            contact,
            doi,
            identifiers,
            preferred_citation,
            references,
        } = cff.clone();
        let version = version.and_then(|v| semver::Version::parse(&v).ok());
        let date = date_released.and_then(
            |citeworks_cff::Date { year, month, day }| -> Option<chrono::DateTime<chrono::Utc>> {
                Some(
                    chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)?
                        .and_hms_opt(0, 0, 0)?
                        .and_utc(),
                )
            },
        );
        let key = format!(
            "{}{}",
            authors
                .first()
                .and_then(|a| match a {
                    citeworks_cff::names::Name::Person(person_name) =>
                        person_name.family_names.clone(),
                    citeworks_cff::names::Name::Entity(entity_name) => entity_name.name.clone(),
                    citeworks_cff::names::Name::Anonymous => None,
                })
                .unwrap_or(title.clone()),
            date_released
                .map(|d| format!("{:4}", d.year))
                .unwrap_or("".to_owned())
        );
        let author = authors
            .into_iter()
            .map(|author| {
                use citeworks_cff::names::Name::*;
                match author {
                    #[allow(unused)]
                    Person(citeworks_cff::names::PersonName {
                        family_names,
                        given_names,
                        name_particle,
                        name_suffix,
                        affiliation,
                        meta,
                    }) => format!(
                        "{}{}{}{}",
                        given_names.map(|x| format!("{x} ")).unwrap_or_default(),
                        name_particle.map(|x| format!("{x} ")).unwrap_or_default(),
                        family_names.map(|x| format!("{x} ")).unwrap_or_default(),
                        name_suffix.unwrap_or_default(),
                    )
                    .trim_end()
                    .to_string(),
                    #[allow(unused)]
                    Entity(citeworks_cff::names::EntityName {
                        name,
                        date_start,
                        date_end,
                        meta,
                    }) => name.unwrap_or_default(),
                    Anonymous => "Anonymous".to_string(),
                }
            })
            .reduce(|acc, x| format!("{acc}, {x}"))
            .unwrap_or_default();
        Ok(Self {
            key,
            work_type: match work_type {
                Some(citeworks_cff::WorkType::Software) => "software",
                Some(citeworks_cff::WorkType::Dataset) => "dataset",
                None => "software",
            }
            .to_string(),
            author, // authors.into_iter().map(|a| format!("{a}")),
            title: format!(
                "{{{title}}}{}",
                abstract_text.map_or_else(|| "".to_string(), |x| format!(": {x}"))
            ),
            url: repository
                .map(|url| format!("{url}"))
                .or(repository_code.map(|url| format!("{url}")))
                .or(repository_artifact.map(|url| format!("{url}"))),
            license: match license {
                Some(citeworks_cff::License::Single(l)) => Some(format!("{l}")),
                Some(citeworks_cff::License::AnyOf(ll)) => {
                    if ll.is_empty() {
                        None
                    } else {
                        let mut out = String::new();
                        let n = ll.len();
                        for (i, l) in ll.iter().enumerate() {
                            if i < n - 1 {
                                out = format!("{out}, {l}");
                            } else {
                                out = format!("{out} OR {l}")
                            }
                        }
                        Some(out)
                    }
                }
                None => None,
            },
            version,
            date,
        })
    }
}

impl std::fmt::Display for BibLaTeXCratesIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Writes the biblatex entry
        writeln!(f, "@{} {{{},", self.work_type, self.key)?;
        writeln!(f, "    author = {{{}}},", self.author)?;
        writeln!(f, "    title = {{{}}},", self.title)?;
        if let Some(u) = &self.url {
            writeln!(f, "    url = {{{u}}},")?;
        };
        if let Some(date) = self.date {
            writeln!(
                f,
                "    date = {{{:4.0}-{:02}-{:02}}},",
                date.year(),
                date.month(),
                date.day(),
            )?;
        }
        if let Some(version) = &self.version {
            writeln!(f, "    version = {{{version}}},")?;
        }
        if let Some(license) = &self.license {
            writeln!(f, "    license = {{{license}}},")?;
        }
        // Closes the entry
        write!(f, "}}")?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct NotFoundError(String);

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl std::error::Error for NotFoundError {
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

async fn search_citation_files(
    client: &reqwest::Client,
    repository: &Option<String>,
) -> Result<Vec<BibLaTeX>, Box<dyn std::error::Error>> {
    if let Some(repo) = repository {
        // Check if this is Github
        if !repo.contains("github") {
            return Ok(vec![]);
        }

        // Make API Call
        // See: https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#get-a-repository
        // https://github.com/OWNER/REPO
        // -> https://api-github.com/repos/OWNER/REPO
        let segments: Vec<_> = repo.split("github.com/").collect();
        if let Some(tail) = segments.get(1) {
            let segments2: Vec<_> = tail.split("/").collect();
            let owner = segments2[0];
            let repo = segments2[1];
            let request_url = format!("https://api.github.com/repos/{owner}/{repo}");
            let response = client
                .get(request_url)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            if let Some(default_branch) = response.get("default_branch") {
                let default_branch = default_branch.to_string().replace("\"", "");

                enum ConvertHow {
                    Cff,
                    Bib,
                }
                use ConvertHow::*;
                let check_files = vec![
                    ("CITATION.cff", Cff),
                    ("Citation.cff", Cff),
                    ("citation.cff", Cff),
                    ("citation.bib", Bib),
                    ("Citation.bib", Bib),
                    ("bibliography.bib", Bib),
                    ("Bibliography.bib", Bib),
                ];
                let request_url_base = format!(
                    "https://raw.githubusercontent.com/\
                {owner}/\
                {repo}/\
                refs/heads/\
                {default_branch}"
                );
                let requests = check_files.into_iter().map(|(filename, how)| {
                    let rq = format!("{request_url_base}/{filename}");
                    (client.get(rq).send(), how, filename)
                });
                let mut results = vec![];
                for (response, how, filename) in requests {
                    let response = response.await?.text().await?;
                    if !response.to_lowercase().contains("404: not found") {
                        match how {
                            Cff => results
                                .push(BibLaTeX::CITATIONCFF(citeworks_cff::from_str(&response)?)),

                            Bib => results.push(BibLaTeX::Bibliography {
                                bibliography: biblatex::Bibliography::parse(&response)
                                    .map_err::<Box<dyn std::error::Error>, _>(|e| {
                                        format!("{e}").into()
                                    })?,
                                url: request_url_base.clone(),
                                filename: filename.to_string(),
                            }),
                        }
                    }
                }
                return Ok(results);
            }
        }
    }
    Ok(vec![])
}

/// Returns a [BibLaTeX] entry for the searched crate.
///
/// ## Note
/// crates.io requires the specification of a user-agent
/// but this may yield errors when calling from a static website due to CORS.
pub async fn get_biblatex(
    crate_name: &str,
    version: Option<&str>,
    user_agent: Option<&str>,
) -> Result<Vec<BibLaTeX>, Box<dyn std::error::Error>> {
    let mut results = vec![];
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

    let (index, found_version_semver) = if let Some(version) = version {
        let version = semver::Comparator::parse(version)?;
        obtained_versions
            .into_iter()
            .find(|x| version.matches(&x.1))
    } else {
        obtained_versions.first().cloned()
    }
    .ok_or(NotFoundError(
        version.map_or(format!("Could not find crate {crate_name}"), |x| {
            format!("Could not find version {x} for crate {crate_name}")
        }),
    ))?;
    let found_version = info.versions[index].clone();

    results.extend(search_citation_files(&client1, &info.crate_data.repository).await?);

    results.push(BibLaTeX::CratesIO(BibLaTeXCratesIO {
        key: format!(
            "{}{}",
            found_version
                .published_by
                .clone()
                .and_then(|x| x
                    .name
                    .and_then(|x| x.split(" ").nth(1).map(|x| x.to_string())))
                .unwrap_or(crate_name.to_string()),
            info.crate_data.updated_at.year()
        ),
        work_type: "software".to_string(),
        author: found_version
            .published_by
            .map_or_else(|| "".to_owned(), |x| x.name.unwrap_or(x.login)),
        title: info
            .crate_data
            .description
            .map_or(format!("{{{}}}", crate_name), |x| {
                format!("{{{}}}: {}", crate_name, x)
            }),
        url: info.crate_data.repository,
        license: found_version.license,
        version: Some(found_version_semver),
        date: Some(found_version.updated_at),
    }));
    Ok(results)
}

/// Wraps the [crate2bib::get_biblatex] function.
///
/// Args:
///     crate_name(str): Name of the crate to get BibLaTeX entry
///     version (str): A semver-compliant version number for the crate
///     user_agent (:obj:`str`, optional):: The name of the user agent. Defaults to None.
/// Returns:
///     tuple: The formatted BibLaTeX entry and its origin given by [crate2bib::EntryOrigin]
#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(
    name = "get_biblatex",
    signature = (crate_name, semver = None, user_agent = None),
)]
fn get_biblatex_py(
    py: Python,
    crate_name: String,
    semver: Option<String>,
    user_agent: Option<String>,
) -> PyResult<Bound<PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        Ok(
            get_biblatex(&crate_name, semver.as_deref(), user_agent.as_deref())
                .await
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{e}")))?
                .into_iter()
                .map(|(bib_entry, origin)| (format!("{bib_entry}"), origin as isize))
                .collect::<Vec<_>>(),
        )
    })
}

/// Wrapper of the [crate2bib] crate
#[cfg(feature = "pyo3")]
#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
#[pymodule]
fn crate2bib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_biblatex_py, m)?)?;
    m.add_class::<BibLaTeXCratesIO>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn access_crates_io() -> Result<(), Box<dyn std::error::Error>> {
        let results = get_biblatex("serde", Some("1.0.217"), Some("crate2bib-testing")).await?;
        let bib_entry = &results[0];
        let expected = "\
@software {Tolnay2024,
    author = {David Tolnay},
    title = {{serde}: A generic serialization/deserialization framework},
    url = {https://github.com/serde-rs/serde},
    date = {2024-12-27},
    version = {1.0.217},
    license = {MIT OR Apache-2.0},
}";
        assert_eq!(format!("{}", bib_entry), expected);
        if let &BibLaTeX::CratesIO(_) = bib_entry {
        } else {
            panic!("got wrong return type");
        }
        Ok(())
    }

    #[tokio::test]
    async fn find_citation_cff() -> Result<(), Box<dyn std::error::Error>> {
        let results = get_biblatex("cellular-raza", Some("0.1"), Some("crate2bib-testing")).await?;
        let bib_entry = &results[0];
        match bib_entry {
            BibLaTeX::CITATIONCFF(_) => (),
            _ => panic!("Got wrong entry type 1"),
        }
        let bib_entry = &results[1];
        println!("{bib_entry}");
        match bib_entry {
            BibLaTeX::CratesIO(_) => (),
            _ => panic!("Got wrong return type 2"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn find_crate_without_version() -> Result<(), Box<dyn std::error::Error>> {
        let results = get_biblatex("cellular-raza", None, Some("crate2bib-testing")).await?;
        assert!(!results.is_empty());
        Ok(())
    }
}
