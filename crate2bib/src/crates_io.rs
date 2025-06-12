#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// A fully specified BibLaTeX entry generated from a crate hostedn on
/// [crates.io](https://crates.io)
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

impl BibLaTeXCratesIO {
    /// Creates a [BibLaTeXCratesIO] from a given [citeworks_cff::Cff] file
    pub fn from_citation_cff(cff: &citeworks_cff::Cff) -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "log")]
        log::trace!("Converting CITATION.cff to BibLaTeX");
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
                #[cfg(feature = "log")]
                log::trace!("Found release date");
                Some(
                    chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)?
                        .and_hms_opt(0, 0, 0)?
                        .and_utc(),
                )
            },
        );
        #[cfg(feature = "log")]
        log::trace!("Formatting Key");
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
        #[cfg(feature = "log")]
        log::trace!("Formatting Authors");
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
        #[cfg(feature = "log")]
        log::trace!("Finishing Conversion");
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
        #[cfg(feature = "log")]
        log::trace!("Formatting BibLaTeXCratesIO");
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

/// Returns a [BibLaTeXCratesIO] entry for the searched crate.
///
/// ## Note
/// crates.io requires the specification of a user-agent
/// but this may yield errors when calling from a static website due to CORS.
pub async fn generate_biblatex_crates_io(
    crate_name: &str,
    version: Option<&str>,
    client: &crates_io_api::AsyncClient,
) -> crate::Result<BibLaTeXCratesIO> {
    #[cfg(feature = "log")]
    log::trace!("Obtaining Crate Information");
    let info = client.get_crate(crate_name).await?;
    #[cfg(feature = "log")]
    log::trace!("Filter versions");
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
    .ok_or(crate::NotFoundError(
        version.map_or(format!("Could not find crate {crate_name}"), |x| {
            format!("Could not find version {x} for crate {crate_name}")
        }),
    ))?;
    let found_version = info.versions[index].clone();

    #[cfg(feature = "log")]
    log::trace!("Bundling Information into BibLaTeXCratesIO");
    Ok(BibLaTeXCratesIO {
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
    })
}

/// Obtain multiple BibLaTeX entries from various sources such as crates.io, github and doi.org
pub async fn get_biblatex(
    crate_name: &str,
    version: Option<&str>,
    user_agent: Option<&str>,
    branch_name: Option<&str>,
    filenames: Vec<&str>,
) -> crate::Result<Vec<crate::BibLaTeX>> {
    use crates_io_api::AsyncClient;
    use reqwest::header::*;
    #[cfg(feature = "log")]
    log::trace!("Prepare Headers and Client");
    let mut headers = HeaderMap::new();
    if let Some(ua) = user_agent {
        headers.insert(USER_AGENT, HeaderValue::from_str(ua)?);
    }

    let client1 = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let client =
        AsyncClient::with_http_client(client1.clone(), web_time::Duration::from_millis(1000));
    let r1 = generate_biblatex_crates_io(crate_name, version, &client).await?;
    let url = r1.url.clone();

    #[cfg(feature = "log")]
    log::trace!("Obtain entry from crates.io");
    let mut results = vec![crate::BibLaTeX::CratesIO(r1)];
    #[cfg(feature = "log")]
    log::trace!("Obtain other entries");
    if let Some(u) = url {
        results
            .extend(crate::github_search_files(&client1, &u, filenames, branch_name, true).await?);
    }
    #[cfg(feature = "log")]
    log::trace!("Sort obtained entries by priority");
    results.sort_by_key(|x| u8::MAX - x.priority());

    Ok(results)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn access_crates_io() -> crate::Result<()> {
        let bib_entry = get_biblatex(
            "serde",
            Some("1.0.219"),
            Some("crate2bib-testing"),
            None,
            vec![],
        )
        .await?[0]
            .clone();
        let expected = "\
@software {Tolnay2025,
    author = {David Tolnay},
    title = {{serde}: A generic serialization/deserialization framework},
    url = {https://github.com/serde-rs/serde},
    date = {2025-03-09},
    version = {1.0.219},
    license = {MIT OR Apache-2.0},
}";
        assert_eq!(format!("{}", bib_entry), expected);
        if let BibLaTeX::CratesIO(_) = bib_entry {
        } else {
            panic!("got wrong return type");
        }
        Ok(())
    }

    #[tokio::test]
    async fn find_citation_cff() -> crate::Result<()> {
        let results = get_biblatex(
            "cellular-raza",
            Some("0.1"),
            Some("crate2bib-testing"),
            None,
            vec!["CITATION.cff"],
        )
        .await?;
        let bib_entry = &results[0];
        match bib_entry {
            BibLaTeX::Plain(_) => (),
            _ => panic!("Got wrong entry type 1"),
        }
        let bib_entry = &results[1];
        match bib_entry {
            BibLaTeX::CratesIO(_) => (),
            _ => panic!("Got wrong return type 2"),
        }
        let bib_entry = &results[2];
        match bib_entry {
            BibLaTeX::CITATIONCFF(_) => (),
            _ => panic!("Got wrong return type 3"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn find_crate_without_version() -> crate::Result<()> {
        let results = get_biblatex(
            "cellular-raza",
            None,
            Some("crate2bib-testing"),
            None,
            vec![],
        )
        .await?;
        assert!(!results.is_empty());
        Ok(())
    }
}
