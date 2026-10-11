//! Create BibLaTeX entries for packages registered in the Julia
//! [General registry](https://github.com/JuliaRegistries/General).
//!
//! The functionality in this module reads a package's `Package.toml` and `Versions.toml` from the
//! General registry (which is itself a git repository hosted on GitHub) and turns the returned
//! information into a `@software` BibLaTeX entry. Packages can be queried either in their latest
//! version or pinned to a specific version.
//!
//! In addition the source repository declared by the package is scanned for citation files
//! (e.g. `CITATION.cff` or `citation.bib`), mirroring the behaviour of the
//! [PyPI](crate::get_biblatex_pypi) functionality.

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use serde::{Deserialize, Serialize};

/// Base URL of the raw contents of the Julia General registry.
const GENERAL_REGISTRY: &str =
    "https://raw.githubusercontent.com/JuliaRegistries/General/master";

/// A BibLaTeX entry generated from a package registered in the Julia General registry.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct BibLaTeXJulia {
    /// BibLaTeX citation key which can be used in LaTeX `\cite{key}`.
    pub key: String,
    /// The BibLaTeX entry type. Julia packages are treated as `software`.
    pub work_type: String,
    /// The title of the package (its name).
    pub title: String,
    /// A link to the package's source repository.
    pub url: Option<String>,
    /// The resolved version of the package.
    pub version: String,
    /// The universally unique identifier assigned to the package by the registry.
    pub uuid: String,
}

/// Contents of a package's `Package.toml` in the General registry.
#[derive(Clone, Debug, Deserialize)]
struct JuliaPackage {
    name: String,
    uuid: String,
    repo: String,
}

/// A single entry of a package's `Versions.toml` in the General registry.
///
/// Unknown fields (such as `git-tree-sha1`) are ignored during deserialization.
#[derive(Clone, Debug, Deserialize, Default)]
struct JuliaVersion {
    #[serde(default)]
    yanked: bool,
}

/// Determines the registry sub-directory (bucket) a package is stored in.
///
/// Packages in the General registry are grouped into a directory named after the first (upper-cased)
/// character of the package name, e.g. `Example` lives in `E/Example`.
fn registry_bucket(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_default()
}

/// Strips a trailing `.git` suffix from a repository URL so it can be used for browsing and for
/// scanning citation files.
fn clean_repo_url(repo: &str) -> String {
    repo.trim().trim_end_matches(".git").to_string()
}

/// Selects a version from the parsed `Versions.toml`.
///
/// If `version` is `None` (or empty) the highest non-yanked [semver] version is returned, otherwise
/// the requested version is matched exactly.
fn select_version(
    package: &str,
    versions: &std::collections::HashMap<String, JuliaVersion>,
    version: Option<&str>,
) -> crate::Result<String> {
    if let Some(v) = version.map(str::trim).filter(|v| !v.is_empty()) {
        if versions.contains_key(v) {
            return Ok(v.to_string());
        }
        return Err(crate::Err::NotFound(format!(
            "could not find version \"{v}\" of Julia package \"{package}\""
        )));
    }

    let mut parsed: Vec<(semver::Version, &String)> = versions
        .iter()
        .filter(|(_, info)| !info.yanked)
        .filter_map(|(raw, _)| semver::Version::parse(raw).ok().map(|v| (v, raw)))
        .collect();
    parsed.sort_by(|a, b| a.0.cmp(&b.0));

    parsed
        .last()
        .map(|(_, raw)| raw.to_string())
        .ok_or_else(|| {
            crate::Err::NotFound(format!(
                "could not find any version of Julia package \"{package}\""
            ))
        })
}

impl BibLaTeXJulia {
    /// Builds a [BibLaTeXJulia] entry from the registry metadata and the resolved version.
    fn from_registry(package: JuliaPackage, version: String) -> Self {
        #[cfg(feature = "log")]
        log::trace!("Converting Julia registry metadata to BibLaTeX");

        let url = {
            let repo = clean_repo_url(&package.repo);
            if repo.is_empty() {
                None
            } else {
                Some(repo)
            }
        };

        let key_name: String = package
            .name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();

        Self {
            key: key_name,
            work_type: "software".to_string(),
            title: format!("{{{}}}", package.name),
            url,
            version,
            uuid: package.uuid,
        }
    }
}

impl std::fmt::Display for BibLaTeXJulia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "log")]
        log::trace!("Formatting BibLaTeXJulia");
        writeln!(f, "@{} {{{},", self.work_type, self.key)?;
        writeln!(f, "    title = {{{}}},", self.title)?;
        if let Some(u) = &self.url {
            writeln!(f, "    url = {{{u}}},")?;
        }
        writeln!(f, "    version = {{{}}},", self.version)?;
        writeln!(f, "    note = {{UUID: {}}},", self.uuid)?;
        write!(f, "}}")?;
        Ok(())
    }
}

/// Fetches and parses a TOML file from the General registry for the given package.
async fn fetch_registry_file<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    package: &str,
    file: &str,
) -> crate::Result<T> {
    let bucket = registry_bucket(package);
    let rq = format!("{GENERAL_REGISTRY}/{bucket}/{package}/{file}");

    #[cfg(feature = "log")]
    log::trace!("Requesting Julia registry file \"{rq}\"");
    let res = client.get(&rq).send().await?;
    let status = res.status().as_u16();
    let text = res.text().await?;
    if status == 404 || text.to_lowercase().trim() == "404: not found" {
        return Err(crate::Err::NotFound(format!(
            "could not find Julia package \"{package}\" in the General registry"
        )));
    }

    #[cfg(feature = "log")]
    log::trace!("Parsing Julia registry file \"{file}\"");
    toml::from_str(&text).map_err(crate::Err::from)
}

/// Generates a [BibLaTeXJulia] entry for a package registered in the Julia General registry.
///
/// If `version` is `None` (or empty) the latest non-yanked release is used, otherwise the given
/// version is matched exactly.
pub async fn generate_biblatex_julia(
    package: &str,
    version: Option<&str>,
    client: &reqwest::Client,
) -> crate::Result<BibLaTeXJulia> {
    let name = package.trim();

    let package_info: JuliaPackage = fetch_registry_file(client, name, "Package.toml").await?;
    let versions: std::collections::HashMap<String, JuliaVersion> =
        fetch_registry_file(client, name, "Versions.toml").await?;

    let resolved = select_version(name, &versions, version)?;
    Ok(BibLaTeXJulia::from_registry(package_info, resolved))
}

/// Convenience wrapper around [generate_biblatex_julia] that builds a [reqwest::Client] from an
/// optional `user_agent`.
pub async fn generate_biblatex_julia_without_client(
    package: &str,
    version: Option<&str>,
    user_agent: Option<&str>,
) -> crate::Result<BibLaTeXJulia> {
    let client = crate::create_client(user_agent, None)?;
    generate_biblatex_julia(package, version, &client).await
}

/// Obtains BibLaTeX entries for a package registered in the Julia General registry.
///
/// In addition to the `@software` entry generated from the registry metadata this also scans the
/// package's source repository for citation files (e.g. `CITATION.cff` or `citation.bib`) using
/// [crate::githost_search_files]. This way other citation types declared by the authors (such as a
/// journal `@article`) are gathered as well. The returned entries are sorted by priority.
pub async fn get_biblatex_julia(
    package: &str,
    version: Option<&str>,
    user_agent: Option<&str>,
    branch_name: Option<&str>,
    filenames: Vec<&str>,
) -> crate::Result<Vec<crate::Result<crate::BibLaTeX>>> {
    let client = crate::create_client(user_agent, None)?;

    #[cfg(feature = "log")]
    log::trace!("Obtain entry from the Julia General registry");
    let entry = generate_biblatex_julia(package, version, &client).await?;
    let url = entry.url.clone();

    let mut results = vec![Ok(crate::BibLaTeX::Julia(entry))];

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
    fn bucket_from_name() {
        assert_eq!(registry_bucket("Example"), "E");
        assert_eq!(registry_bucket("plots"), "P");
    }

    #[test]
    fn repo_url_is_cleaned() {
        assert_eq!(
            clean_repo_url("https://github.com/JuliaLang/Example.jl.git"),
            "https://github.com/JuliaLang/Example.jl"
        );
    }

    #[tokio::test]
    async fn requests_pinned_version() -> crate::Result<()> {
        let entry = generate_biblatex_julia_without_client(
            "Example",
            Some("0.5.3"),
            Some("crate2bib-testing"),
        )
        .await?;

        let expected = "\
@software {Example,
    title = {{Example}},
    url = {https://github.com/JuliaLang/Example.jl},
    version = {0.5.3},
    note = {UUID: 7876af07-990d-54b4-ab0e-23690620f79a},
}";
        assert_eq!(format!("{entry}"), expected);
        assert_eq!(entry.work_type, "software");
        Ok(())
    }

    #[tokio::test]
    async fn latest_version_resolves() -> crate::Result<()> {
        let entry =
            generate_biblatex_julia_without_client("Example", None, Some("crate2bib-testing"))
                .await?;
        assert!(!entry.version.is_empty());
        assert_eq!(entry.uuid, "7876af07-990d-54b4-ab0e-23690620f79a");
        Ok(())
    }
}
