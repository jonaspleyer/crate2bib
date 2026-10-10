//! Create BibLaTeX entries from items hosted on the Internet Archive
//! [archive.org](https://archive.org).
//!
//! The functionality in this module queries the public
//! [metadata API](https://archive.org/developers/md-read.html) of an item and turns the
//! returned information into a reasonable BibLaTeX entry. The citation type (e.g. `@book`,
//! `@article`, `@video`, ...) is inferred from the item's `mediatype` and a few additional
//! hints contained in the metadata.

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use serde::{Deserialize, Serialize};

/// A BibLaTeX entry generated from an item hosted on
/// [archive.org](https://archive.org).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct BibLaTeXArchiveOrg {
    /// BibLaTeX citation key which can be used in LaTeX `\cite{key}`.
    pub key: String,
    /// The inferred BibLaTeX entry type such as `book`, `article` or `misc`.
    pub entry_type: String,
    /// All creators of the item joined by ` and ` (BibLaTeX name separator).
    pub author: Option<String>,
    /// The title of the item.
    pub title: String,
    /// The publication year.
    pub year: Option<String>,
    /// The place of publication (e.g. `New York`).
    pub location: Option<String>,
    /// The name of the publisher.
    pub publisher: Option<String>,
    /// The language of the item (resolved to a BibLaTeX language name where possible).
    pub language: Option<String>,
    /// Subjects/keywords describing the item, joined by `, `.
    pub keywords: Option<String>,
    /// The archive.org identifier of the item.
    pub identifier: String,
    /// A link to the item on archive.org.
    pub url: String,
}

/// Top-level response returned by the archive.org metadata API.
#[derive(Clone, Debug, Deserialize)]
struct ArchiveOrgResponse {
    #[serde(default)]
    metadata: Option<ArchiveOrgMetadata>,
}

/// Subset of the metadata fields that are relevant for building a BibLaTeX entry.
///
/// Many archive.org fields may be either a single string or a list of strings which is why
/// [StringOrVec] is used for most of them. Unknown fields are ignored during deserialization.
#[derive(Clone, Debug, Deserialize)]
struct ArchiveOrgMetadata {
    title: Option<StringOrVec>,
    creator: Option<StringOrVec>,
    publisher: Option<StringOrVec>,
    date: Option<StringOrVec>,
    language: Option<StringOrVec>,
    mediatype: Option<String>,
    subject: Option<StringOrVec>,
    /// Present on items that are individual issues of a journal/periodical.
    volume: Option<StringOrVec>,
    /// Present on items that are individual articles of a journal/periodical.
    journaltitle: Option<StringOrVec>,
}

/// Helper type that transparently deserializes a field that is either a single string or a
/// list of strings.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    One(String),
    Many(Vec<String>),
}

impl StringOrVec {
    /// Returns the first contained value if any.
    fn first(&self) -> Option<&str> {
        match self {
            StringOrVec::One(s) => Some(s.as_str()),
            StringOrVec::Many(v) => v.first().map(|s| s.as_str()),
        }
    }

    /// Returns all contained values.
    fn all(&self) -> Vec<&str> {
        match self {
            StringOrVec::One(s) => vec![s.as_str()],
            StringOrVec::Many(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

/// Extracts the archive.org identifier from a details URL or returns the input unchanged if it
/// already looks like a bare identifier.
///
/// For example
/// `https://archive.org/details/variationofan02darw/page/484/mode/2up` yields
/// `variationofan02darw`.
fn extract_identifier(url_or_identifier: &str) -> String {
    let trimmed = url_or_identifier.trim();
    if let Some((_, tail)) = trimmed.split_once("/details/") {
        tail.split('/').next().unwrap_or(tail).to_string()
    } else {
        // Fall back to the last non-empty path segment / the input itself.
        trimmed
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or(trimmed)
            .to_string()
    }
}

/// Removes a trailing life-span such as `, 1809-1882` from a creator string.
///
/// archive.org frequently stores creators as `Family, Given, birth-death`. BibLaTeX only wants
/// the name so the numeric trailing part is dropped while the `Family, Given` ordering is kept.
fn clean_creator(raw: &str) -> String {
    raw.split(", ")
        .filter(|part| {
            !part
                .trim_start()
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        })
        .collect::<Vec<_>>()
        .join(", ")
        .trim()
        .to_string()
}

/// Resolves a (potentially ISO 639) language code to a BibLaTeX language name.
///
/// Unknown codes are passed through unchanged.
fn language_name(code: &str) -> String {
    match code.trim().to_lowercase().as_str() {
        "eng" | "en" => "english",
        "ger" | "deu" | "de" => "german",
        "fre" | "fra" | "fr" => "french",
        "spa" | "es" => "spanish",
        "ita" | "it" => "italian",
        "por" | "pt" => "portuguese",
        "lat" | "la" => "latin",
        "rus" | "ru" => "russian",
        "dut" | "nld" | "nl" => "dutch",
        other => other,
    }
    .to_string()
}

/// Extracts the first four-digit year contained in a date string.
fn extract_year(date: &str) -> Option<String> {
    let bytes = date.as_bytes();
    (0..bytes.len())
        .find(|&start| {
            start + 4 <= bytes.len() && bytes[start..start + 4].iter().all(u8::is_ascii_digit)
        })
        .map(|start| date[start..start + 4].to_string())
}

/// Infers a BibLaTeX entry type from the item's `mediatype` and a few additional hints.
fn infer_entry_type(metadata: &ArchiveOrgMetadata) -> &'static str {
    match metadata.mediatype.as_deref() {
        Some("texts") => {
            // A `texts` item is usually a book. If it carries typical periodical hints we
            // treat it as an article instead.
            if metadata.journaltitle.is_some() || metadata.volume.is_some() {
                "article"
            } else {
                "book"
            }
        }
        Some("audio") | Some("etree") => "audio",
        Some("movies") => "video",
        Some("software") => "software",
        Some("image") => "image",
        Some("web") => "online",
        Some("data") => "dataset",
        _ => "misc",
    }
}

impl BibLaTeXArchiveOrg {
    /// Builds a [BibLaTeXArchiveOrg] entry from the raw metadata of an item.
    fn from_metadata(identifier: &str, metadata: ArchiveOrgMetadata) -> crate::Result<Self> {
        #[cfg(feature = "log")]
        log::trace!("Converting archive.org metadata to BibLaTeX");

        let title = metadata
            .title
            .as_ref()
            .and_then(|t| t.first())
            .map(str::to_string)
            .ok_or_else(|| {
                crate::Err::NotFound(format!(
                    "archive.org item \"{identifier}\" does not provide a title"
                ))
            })?;

        let entry_type = infer_entry_type(&metadata).to_string();

        let authors: Vec<String> = metadata
            .creator
            .as_ref()
            .map(|c| c.all().into_iter().map(clean_creator).collect())
            .unwrap_or_default();
        let author = if authors.is_empty() {
            None
        } else {
            Some(authors.join(" and "))
        };

        let year = metadata
            .date
            .as_ref()
            .and_then(|d| d.first())
            .and_then(extract_year);

        // The publisher field is commonly formatted as `Place : Publisher`.
        let (location, publisher) = match metadata.publisher.as_ref().and_then(|p| p.first()) {
            Some(raw) => match raw.split_once(" : ") {
                Some((loc, pubr)) => (Some(loc.trim().to_string()), Some(pubr.trim().to_string())),
                None => (None, Some(raw.trim().to_string())),
            },
            None => (None, None),
        };

        let language = metadata
            .language
            .as_ref()
            .and_then(|l| l.first())
            .map(language_name);

        let keywords = metadata.subject.as_ref().map(|s| s.all().join(", "));

        // Build a citation key from the first author's family name and the year.
        let key_name = authors
            .first()
            .and_then(|a| a.split(',').next())
            .map(|s| s.replace(char::is_whitespace, ""))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| identifier.to_string());
        let key = format!("{key_name}{}", year.clone().unwrap_or_default());

        Ok(Self {
            key,
            entry_type,
            author,
            title,
            year,
            location,
            publisher,
            language,
            keywords,
            identifier: identifier.to_string(),
            url: format!("https://archive.org/details/{identifier}"),
        })
    }
}

impl std::fmt::Display for BibLaTeXArchiveOrg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "log")]
        log::trace!("Formatting BibLaTeXArchiveOrg");
        writeln!(f, "@{}{{{},", self.entry_type, self.key)?;
        if let Some(author) = &self.author {
            writeln!(f, "    author = {{{author}}},")?;
        }
        writeln!(f, "    title = {{{}}},", self.title)?;
        if let Some(year) = &self.year {
            writeln!(f, "    date = {{{year}}},")?;
        }
        if let Some(location) = &self.location {
            writeln!(f, "    location = {{{location}}},")?;
        }
        if let Some(publisher) = &self.publisher {
            writeln!(f, "    publisher = {{{publisher}}},")?;
        }
        if let Some(language) = &self.language {
            writeln!(f, "    language = {{{language}}},")?;
        }
        if let Some(keywords) = &self.keywords {
            writeln!(f, "    keywords = {{{keywords}}},")?;
        }
        writeln!(f, "    url = {{{}}},", self.url)?;
        write!(f, "}}")?;
        Ok(())
    }
}

/// Generates a [BibLaTeXArchiveOrg] entry for an item hosted on
/// [archive.org](https://archive.org).
///
/// The `url_or_identifier` may either be a full details URL such as
/// `https://archive.org/details/variationofan02darw/page/484/mode/2up` or a bare identifier
/// like `variationofan02darw`.
pub async fn generate_biblatex_archive_org(
    url_or_identifier: &str,
    client: &reqwest::Client,
) -> crate::Result<BibLaTeXArchiveOrg> {
    let identifier = extract_identifier(url_or_identifier);
    let rq = format!("https://archive.org/metadata/{identifier}");

    #[cfg(feature = "log")]
    log::trace!("Requesting archive.org metadata for \"{identifier}\"");
    let res = client.get(&rq).send().await?;

    #[cfg(feature = "log")]
    log::trace!("Parsing archive.org metadata response");
    let response = res.json::<ArchiveOrgResponse>().await?;
    let metadata = response.metadata.ok_or_else(|| {
        crate::Err::NotFound(format!(
            "could not find metadata for archive.org item \"{identifier}\""
        ))
    })?;

    BibLaTeXArchiveOrg::from_metadata(&identifier, metadata)
}

/// Convenience wrapper around [generate_biblatex_archive_org] that builds a
/// [reqwest::Client] from an optional `user_agent`.
pub async fn generate_biblatex_archive_org_without_client(
    url_or_identifier: &str,
    user_agent: Option<&str>,
) -> crate::Result<BibLaTeXArchiveOrg> {
    let client = crate::create_client(user_agent, None)?;
    generate_biblatex_archive_org(url_or_identifier, &client).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_from_details_url() {
        assert_eq!(
            extract_identifier("https://archive.org/details/variationofan02darw/page/484/mode/2up"),
            "variationofan02darw"
        );
        assert_eq!(
            extract_identifier("variationofan02darw"),
            "variationofan02darw"
        );
    }

    #[test]
    fn creator_is_cleaned() {
        assert_eq!(
            clean_creator("Darwin, Charles, 1809-1882"),
            "Darwin, Charles"
        );
        assert_eq!(clean_creator("Gray, Asa, 1810-1888"), "Gray, Asa");
    }

    #[tokio::test]
    async fn variation_of_animals_and_plants() -> crate::Result<()> {
        let entry = generate_biblatex_archive_org_without_client(
            "https://archive.org/details/variationofan02darw/page/484/mode/2up",
            Some("crate2bib-testing"),
        )
        .await?;

        let expected = "\
@book{Darwin1868,
    author = {Darwin, Charles and Gray, Asa},
    title = {The variation of animals and plants under domestication},
    date = {1868},
    location = {New York},
    publisher = {Orange Judd & Co.},
    language = {english},
    keywords = {Variation (Biology), Domestic animals, Plants, Cultivated},
    url = {https://archive.org/details/variationofan02darw},
}";
        assert_eq!(format!("{entry}"), expected);
        assert_eq!(entry.entry_type, "book");
        Ok(())
    }
}
