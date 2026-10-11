//! Create BibLaTeX entries for packages hosted on the Python Package Index
//! [PyPI](https://pypi.org).
//!
//! The functionality in this module queries the public
//! [JSON API](https://docs.pypi.org/api/json/) of a package and turns the returned information
//! into a `@software` BibLaTeX entry. Packages can be queried either in their latest version or
//! pinned to a specific version.

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// A BibLaTeX entry generated from a package hosted on [PyPI](https://pypi.org).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct BibLaTeXPyPI {
    /// BibLaTeX citation key which can be used in LaTeX `\cite{key}`.
    pub key: String,
    /// The BibLaTeX entry type. Python packages are treated as `software`.
    pub work_type: String,
    /// The author(s) of the package joined by ` and ` (BibLaTeX name separator).
    pub author: Option<String>,
    /// The title of the package, combining its name and summary.
    pub title: String,
    /// A link to the package's homepage, source repository or PyPI project page.
    pub url: Option<String>,
    /// The license under which the package is distributed.
    pub license: Option<String>,
    /// The version of the package.
    pub version: String,
    /// The time at which this version was uploaded to PyPI.
    pub date: Option<chrono::DateTime<chrono::Utc>>,
    /// Keywords describing the package, joined by `, `.
    pub keywords: Option<String>,
}

/// Top-level response returned by the PyPI JSON API.
#[derive(Clone, Debug, Deserialize)]
struct PyPIResponse {
    info: PyPIInfo,
    #[serde(default)]
    urls: Vec<PyPIUrl>,
}

/// Subset of the `info` object that is relevant for building a BibLaTeX entry.
///
/// Unknown fields are ignored during deserialization.
#[derive(Clone, Debug, Deserialize)]
struct PyPIInfo {
    name: String,
    version: String,
    summary: Option<String>,
    author: Option<String>,
    author_email: Option<String>,
    maintainer: Option<String>,
    maintainer_email: Option<String>,
    license: Option<String>,
    license_expression: Option<String>,
    home_page: Option<String>,
    #[serde(default)]
    project_urls: Option<std::collections::HashMap<String, String>>,
    keywords: Option<String>,
}

/// A single uploaded distribution file of a release.
#[derive(Clone, Debug, Deserialize)]
struct PyPIUrl {
    upload_time_iso_8601: Option<String>,
}

/// Extracts human-readable names from an RFC-5322-style email field such as
/// `Kenneth Reitz <me@kennethreitz.org>` or a comma-separated list thereof.
fn parse_email_field(field: &str) -> Vec<String> {
    field
        .split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            // Prefer the display name in front of `<email>` if present.
            if let Some((name, _)) = entry.split_once('<') {
                let name = name.trim().trim_matches('"').trim();
                if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                }
            } else {
                Some(entry.to_string())
            }
        })
        .collect()
}

/// Resolves the author of a package, preferring the explicit `author`/`maintainer` fields and
/// falling back to the names contained in the corresponding email fields.
fn resolve_author(info: &PyPIInfo) -> Option<String> {
    for plain in [&info.author, &info.maintainer] {
        if let Some(a) = plain {
            let a = a.trim();
            if !a.is_empty() {
                return Some(a.to_string());
            }
        }
    }
    for email in [&info.author_email, &info.maintainer_email] {
        if let Some(e) = email {
            let names = parse_email_field(e);
            if !names.is_empty() {
                return Some(names.join(" and "));
            }
        }
    }
    None
}

/// Resolves the most useful URL for a package.
///
/// The source repository or homepage declared in `project_urls` is preferred, followed by the
/// legacy `home_page` field and finally the package's PyPI project page.
fn resolve_url(info: &PyPIInfo) -> Option<String> {
    if let Some(urls) = &info.project_urls {
        for wanted in ["source", "repository", "source code", "homepage", "home"] {
            if let Some((_, v)) = urls.iter().find(|(k, _)| k.to_lowercase() == wanted) {
                if !v.trim().is_empty() {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    if let Some(hp) = &info.home_page {
        if !hp.trim().is_empty() {
            return Some(hp.trim().to_string());
        }
    }
    Some(format!("https://pypi.org/project/{}/", info.name))
}

/// Resolves the license of a package, preferring the SPDX `license_expression` over the free-form
/// `license` field. Overly long `license` values (usually full license texts) are ignored.
fn resolve_license(info: &PyPIInfo) -> Option<String> {
    if let Some(le) = &info.license_expression {
        let le = le.trim();
        if !le.is_empty() {
            return Some(le.to_string());
        }
    }
    if let Some(l) = &info.license {
        let l = l.trim();
        if !l.is_empty() && l.len() <= 64 && !l.contains('\n') {
            return Some(l.to_string());
        }
    }
    None
}

/// Normalizes the comma- or whitespace-separated `keywords` field into a `, `-joined list.
fn resolve_keywords(info: &PyPIInfo) -> Option<String> {
    let raw = info.keywords.as_ref()?.trim();
    if raw.is_empty() {
        return None;
    }
    let parts: Vec<&str> = raw
        .split([',', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

/// Parses the date part of an ISO-8601 timestamp such as `2023-05-22T15:12:42.313790Z`.
fn parse_date(iso: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let date_part = iso.get(0..10)?;
    let mut it = date_part.split('-');
    let year: i32 = it.next()?.parse().ok()?;
    let month: u32 = it.next()?.parse().ok()?;
    let day: u32 = it.next()?.parse().ok()?;
    Some(
        chrono::NaiveDate::from_ymd_opt(year, month, day)?
            .and_hms_opt(0, 0, 0)?
            .and_utc(),
    )
}

impl BibLaTeXPyPI {
    /// Builds a [BibLaTeXPyPI] entry from a parsed PyPI JSON response.
    fn from_response(response: PyPIResponse) -> Self {
        #[cfg(feature = "log")]
        log::trace!("Converting PyPI metadata to BibLaTeX");
        let info = response.info;

        let author = resolve_author(&info);
        let url = resolve_url(&info);
        let license = resolve_license(&info);
        let keywords = resolve_keywords(&info);

        let date = response
            .urls
            .iter()
            .find_map(|u| u.upload_time_iso_8601.as_deref())
            .and_then(parse_date);

        let title = match &info.summary {
            Some(s) if !s.trim().is_empty() => format!("{{{}}}: {}", info.name, s.trim()),
            _ => format!("{{{}}}", info.name),
        };

        let key_name: String = info.name.chars().filter(|c| c.is_alphanumeric()).collect();
        let key = format!(
            "{key_name}{}",
            date.map(|d| d.year().to_string()).unwrap_or_default()
        );

        Self {
            key,
            work_type: "software".to_string(),
            author,
            title,
            url,
            license,
            version: info.version,
            date,
            keywords,
        }
    }
}

impl std::fmt::Display for BibLaTeXPyPI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "log")]
        log::trace!("Formatting BibLaTeXPyPI");
        writeln!(f, "@{} {{{},", self.work_type, self.key)?;
        if let Some(author) = &self.author {
            writeln!(f, "    author = {{{author}}},")?;
        }
        writeln!(f, "    title = {{{}}},", self.title)?;
        if let Some(u) = &self.url {
            writeln!(f, "    url = {{{u}}},")?;
        }
        if let Some(date) = self.date {
            writeln!(
                f,
                "    date = {{{:4.0}-{:02}-{:02}}},",
                date.year(),
                date.month(),
                date.day(),
            )?;
        }
        writeln!(f, "    version = {{{}}},", self.version)?;
        if let Some(license) = &self.license {
            writeln!(f, "    license = {{{license}}},")?;
        }
        if let Some(keywords) = &self.keywords {
            writeln!(f, "    keywords = {{{keywords}}},")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

/// Generates a [BibLaTeXPyPI] entry for a package hosted on [PyPI](https://pypi.org).
///
/// If `version` is `None` (or empty) the latest release is used, otherwise the given version is
/// queried directly.
pub async fn generate_biblatex_pypi(
    package: &str,
    version: Option<&str>,
    client: &reqwest::Client,
) -> crate::Result<BibLaTeXPyPI> {
    let name = package.trim();
    let rq = match version {
        Some(v) if !v.trim().is_empty() => {
            format!("https://pypi.org/pypi/{name}/{}/json", v.trim())
        }
        _ => format!("https://pypi.org/pypi/{name}/json"),
    };

    #[cfg(feature = "log")]
    log::trace!("Requesting PyPI metadata for \"{name}\"");
    let res = client.get(&rq).send().await?;
    let status = res.status().as_u16();
    if status == 404 {
        return Err(crate::Err::NotFound(format!(
            "could not find package \"{name}\"{} on PyPI",
            version
                .filter(|v| !v.trim().is_empty())
                .map(|v| format!(" with version \"{v}\""))
                .unwrap_or_default()
        )));
    }

    #[cfg(feature = "log")]
    log::trace!("Parsing PyPI metadata response");
    let response = res.json::<PyPIResponse>().await?;
    Ok(BibLaTeXPyPI::from_response(response))
}

/// Convenience wrapper around [generate_biblatex_pypi] that builds a [reqwest::Client] from an
/// optional `user_agent`.
pub async fn generate_biblatex_pypi_without_client(
    package: &str,
    version: Option<&str>,
    user_agent: Option<&str>,
) -> crate::Result<BibLaTeXPyPI> {
    let client = crate::create_client(user_agent, None)?;
    generate_biblatex_pypi(package, version, &client).await
}

/// Obtains BibLaTeX entries for a package hosted on [PyPI](https://pypi.org).
///
/// In addition to the `@software` entry generated from the PyPI metadata this also scans the
/// package's source repository for citation files (e.g. `CITATION.cff` or `citation.bib`) using
/// [crate::githost_search_files]. This way other citation types declared by the authors (such as
/// a journal `@article`) are gathered as well. The returned entries are sorted by priority.
pub async fn get_biblatex_pypi(
    package: &str,
    version: Option<&str>,
    user_agent: Option<&str>,
    branch_name: Option<&str>,
    filenames: Vec<&str>,
) -> crate::Result<Vec<crate::Result<crate::BibLaTeX>>> {
    let client = crate::create_client(user_agent, None)?;

    #[cfg(feature = "log")]
    log::trace!("Obtain entry from PyPI");
    let entry = generate_biblatex_pypi(package, version, &client).await?;
    let url = entry.url.clone();

    let mut results = vec![Ok(crate::BibLaTeX::PyPI(entry))];

    #[cfg(feature = "log")]
    log::trace!("Scan source repository for citation files");
    if let Some(u) = url {
        results
            .extend(crate::githost_search_files(&client, &u, filenames, branch_name, true).await?);
    }

    #[cfg(feature = "log")]
    log::trace!("Sort obtained entries by priority");
    results.sort_by_key(|x| u8::MAX - x.as_ref().map(|x| x.priority()).unwrap_or_default());

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_email_with_display_name() {
        assert_eq!(
            parse_email_field("Kenneth Reitz <me@kennethreitz.org>"),
            vec!["Kenneth Reitz".to_string()]
        );
        assert_eq!(
            parse_email_field("A Author <a@x.org>, B Author <b@y.org>"),
            vec!["A Author".to_string(), "B Author".to_string()]
        );
    }

    #[tokio::test]
    async fn requests_pinned_version() -> crate::Result<()> {
        let entry =
            generate_biblatex_pypi_without_client("requests", Some("2.31.0"), Some("crate2bib-testing"))
                .await?;

        let expected = "\
@software {requests2023,
    author = {Kenneth Reitz},
    title = {{requests}: Python HTTP for Humans.},
    url = {https://github.com/psf/requests},
    date = {2023-05-22},
    version = {2.31.0},
    license = {Apache 2.0},
}";
        assert_eq!(format!("{entry}"), expected);
        assert_eq!(entry.work_type, "software");
        Ok(())
    }

    #[tokio::test]
    async fn numpy_gathers_github_citation() -> crate::Result<()> {
        let results = get_biblatex_pypi(
            "numpy",
            None,
            Some("crate2bib-testing"),
            None,
            vec!["CITATION.cff", "CITATION.bib", "citation.bib"],
        )
        .await?;

        // The PyPI metadata yields a `@software` entry ...
        assert!(results
            .iter()
            .any(|r| matches!(r, Ok(crate::BibLaTeX::PyPI(_)))));
        // ... and numpy's CITATION.bib yields a parsed bibliography (here an `@article`).
        assert!(results
            .iter()
            .any(|r| matches!(r, Ok(crate::BibLaTeX::Plain(_)))));
        Ok(())
    }
}
