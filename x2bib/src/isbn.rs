//! Create BibLaTeX entries for books identified by their
//! [ISBN](https://en.wikipedia.org/wiki/ISBN).
//!
//! The functionality in this module resolves an ISBN to bibliographic metadata and turns it into a
//! `@book` BibLaTeX entry. Metadata is primarily obtained from the public
//! [Open Library API](https://openlibrary.org/dev/docs/api/books) (no API key required) and, as a
//! best-effort fallback, from the [Google Books API](https://developers.google.com/books).

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use serde::{Deserialize, Serialize};

/// A BibLaTeX entry generated from an [ISBN](https://en.wikipedia.org/wiki/ISBN).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct BibLaTeXISBN {
    /// BibLaTeX citation key which can be used in LaTeX `\cite{key}`.
    pub key: String,
    /// The BibLaTeX entry type. Books are treated as `book`.
    pub entry_type: String,
    /// All authors of the book joined by ` and ` (BibLaTeX name separator).
    pub author: Option<String>,
    /// The title of the book (combining its title and subtitle where available).
    pub title: String,
    /// The publication year.
    pub year: Option<String>,
    /// The place of publication (e.g. `Cambridge, MA`).
    pub location: Option<String>,
    /// The name of the publisher.
    pub publisher: Option<String>,
    /// Subjects/keywords describing the book, joined by `, `.
    pub keywords: Option<String>,
    /// The normalized ISBN of the book (without separators).
    pub isbn: String,
    /// A link to the book's record if available.
    pub url: Option<String>,
}

/// A named entity (author, publisher, place or subject) as returned by the Open Library API.
#[derive(Clone, Debug, Deserialize)]
struct Named {
    name: Option<String>,
}

/// Subset of the Open Library `jscmd=data` record that is relevant for building a BibLaTeX entry.
///
/// Unknown fields are ignored during deserialization.
#[derive(Clone, Debug, Deserialize)]
struct OpenLibraryBook {
    title: Option<String>,
    subtitle: Option<String>,
    #[serde(default)]
    authors: Vec<Named>,
    #[serde(default)]
    publishers: Vec<Named>,
    publish_date: Option<String>,
    #[serde(default)]
    publish_places: Vec<Named>,
    #[serde(default)]
    subjects: Vec<Named>,
    url: Option<String>,
}

/// Top-level response of the Google Books `volumes` endpoint.
#[derive(Clone, Debug, Deserialize)]
struct GoogleBooksResponse {
    #[serde(default)]
    items: Vec<GoogleBooksItem>,
}

/// A single volume of a Google Books response.
#[derive(Clone, Debug, Deserialize)]
struct GoogleBooksItem {
    #[serde(rename = "volumeInfo")]
    volume_info: GoogleBooksVolumeInfo,
}

/// Subset of the Google Books `volumeInfo` object that is relevant for building a BibLaTeX entry.
#[derive(Clone, Debug, Deserialize)]
struct GoogleBooksVolumeInfo {
    title: Option<String>,
    subtitle: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    publisher: Option<String>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
}

/// Normalizes an ISBN by removing separators and upper-casing a trailing `x` check digit.
fn normalize_isbn(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_digit() || *c == 'x' || *c == 'X')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Validates the check digit of a 10-digit ISBN.
fn is_valid_isbn10(isbn: &str) -> bool {
    if isbn.len() != 10 {
        return false;
    }
    let mut sum = 0u32;
    for (i, c) in isbn.chars().enumerate() {
        let value = match c {
            '0'..='9' => c.to_digit(10).unwrap(),
            'X' if i == 9 => 10,
            _ => return false,
        };
        sum += value * (10 - i as u32);
    }
    sum % 11 == 0
}

/// Validates the check digit of a 13-digit ISBN.
fn is_valid_isbn13(isbn: &str) -> bool {
    if isbn.len() != 13 {
        return false;
    }
    let mut sum = 0u32;
    for (i, c) in isbn.chars().enumerate() {
        let digit = match c.to_digit(10) {
            Some(d) => d,
            None => return false,
        };
        sum += digit * if i % 2 == 0 { 1 } else { 3 };
    }
    sum % 10 == 0
}

/// Returns the normalized ISBN if it is a valid ISBN-10 or ISBN-13, otherwise an error.
fn validate_isbn(raw: &str) -> crate::Result<String> {
    let isbn = normalize_isbn(raw);
    if is_valid_isbn10(&isbn) || is_valid_isbn13(&isbn) {
        Ok(isbn)
    } else {
        Err(crate::Err::NotFound(format!(
            "\"{raw}\" is not a valid ISBN-10 or ISBN-13"
        )))
    }
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

/// Combines a title and an optional subtitle into a single title string.
fn combine_title(title: &str, subtitle: Option<&str>) -> String {
    match subtitle.map(str::trim).filter(|s| !s.is_empty()) {
        Some(sub) => format!("{}: {sub}", title.trim()),
        None => title.trim().to_string(),
    }
}

/// Builds a citation key from the first author and the publication year.
fn build_key(author: Option<&str>, year: Option<&str>, isbn: &str) -> String {
    let name = author
        .and_then(|a| a.split(" and ").next())
        .and_then(|a| a.split_whitespace().last())
        .map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| isbn.to_string());
    format!("{name}{}", year.unwrap_or_default())
}

impl BibLaTeXISBN {
    /// Builds a [BibLaTeXISBN] entry from an Open Library record.
    fn from_open_library(isbn: &str, book: OpenLibraryBook) -> Option<Self> {
        let title = combine_title(book.title.as_deref()?, book.subtitle.as_deref());

        let authors: Vec<String> = book
            .authors
            .into_iter()
            .filter_map(|a| a.name)
            .filter(|n| !n.trim().is_empty())
            .collect();
        let author = (!authors.is_empty()).then(|| authors.join(" and "));

        let year = book.publish_date.as_deref().and_then(extract_year);
        let publisher = book
            .publishers
            .into_iter()
            .find_map(|p| p.name)
            .filter(|n| !n.trim().is_empty());
        let location = book
            .publish_places
            .into_iter()
            .find_map(|p| p.name)
            .filter(|n| !n.trim().is_empty());
        let subjects: Vec<String> = book.subjects.into_iter().filter_map(|s| s.name).collect();
        let keywords = (!subjects.is_empty()).then(|| subjects.join(", "));

        Some(Self {
            key: build_key(author.as_deref(), year.as_deref(), isbn),
            entry_type: "book".to_string(),
            author,
            title,
            year,
            location,
            publisher,
            keywords,
            isbn: isbn.to_string(),
            url: book.url,
        })
    }

    /// Builds a [BibLaTeXISBN] entry from a Google Books volume.
    fn from_google_books(isbn: &str, info: GoogleBooksVolumeInfo) -> Option<Self> {
        let title = combine_title(info.title.as_deref()?, info.subtitle.as_deref());

        let authors: Vec<String> = info
            .authors
            .into_iter()
            .filter(|n| !n.trim().is_empty())
            .collect();
        let author = (!authors.is_empty()).then(|| authors.join(" and "));

        let year = info.published_date.as_deref().and_then(extract_year);
        let publisher = info.publisher.filter(|n| !n.trim().is_empty());
        let keywords = (!info.categories.is_empty()).then(|| info.categories.join(", "));

        Some(Self {
            key: build_key(author.as_deref(), year.as_deref(), isbn),
            entry_type: "book".to_string(),
            author,
            title,
            year,
            location: None,
            publisher,
            keywords,
            isbn: isbn.to_string(),
            url: Some(format!("https://books.google.com/books?isbn={isbn}")),
        })
    }
}

impl std::fmt::Display for BibLaTeXISBN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "log")]
        log::trace!("Formatting BibLaTeXISBN");
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
        writeln!(f, "    isbn = {{{}}},", self.isbn)?;
        if let Some(keywords) = &self.keywords {
            writeln!(f, "    keywords = {{{keywords}}},")?;
        }
        if let Some(url) = &self.url {
            writeln!(f, "    url = {{{url}}},")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

/// Queries the Open Library API for the given (already normalized) ISBN.
async fn query_open_library(
    isbn: &str,
    client: &reqwest::Client,
) -> crate::Result<Option<BibLaTeXISBN>> {
    let rq = format!("https://openlibrary.org/api/books?bibkeys=ISBN:{isbn}&format=json&jscmd=data");
    #[cfg(feature = "log")]
    log::trace!("Requesting Open Library metadata for ISBN \"{isbn}\"");
    let res = client.get(&rq).send().await?;
    let mut records = res
        .json::<std::collections::HashMap<String, OpenLibraryBook>>()
        .await?;
    // The response is keyed by `ISBN:<isbn>`; there is at most one record.
    Ok(records
        .remove(&format!("ISBN:{isbn}"))
        .and_then(|book| BibLaTeXISBN::from_open_library(isbn, book)))
}

/// Queries the Google Books API for the given (already normalized) ISBN.
async fn query_google_books(
    isbn: &str,
    client: &reqwest::Client,
) -> crate::Result<Option<BibLaTeXISBN>> {
    let rq = format!("https://www.googleapis.com/books/v1/volumes?q=isbn:{isbn}");
    #[cfg(feature = "log")]
    log::trace!("Requesting Google Books metadata for ISBN \"{isbn}\"");
    let res = client.get(&rq).send().await?;
    if !res.status().is_success() {
        return Ok(None);
    }
    let response = res.json::<GoogleBooksResponse>().await?;
    Ok(response
        .items
        .into_iter()
        .next()
        .and_then(|item| BibLaTeXISBN::from_google_books(isbn, item.volume_info)))
}

/// Generates a [BibLaTeXISBN] entry for a book identified by its ISBN.
///
/// The ISBN may contain hyphens or spaces and is validated (ISBN-10 or ISBN-13 check digit) before
/// any request is made. Metadata is looked up on [Open Library](https://openlibrary.org) first and,
/// if nothing is found there, on [Google Books](https://books.google.com).
pub async fn generate_biblatex_isbn(
    isbn: &str,
    client: &reqwest::Client,
) -> crate::Result<BibLaTeXISBN> {
    let isbn = validate_isbn(isbn)?;

    if let Some(entry) = query_open_library(&isbn, client).await? {
        return Ok(entry);
    }

    #[cfg(feature = "log")]
    log::trace!("Open Library had no record, falling back to Google Books");
    if let Some(entry) = query_google_books(&isbn, client).await? {
        return Ok(entry);
    }

    Err(crate::Err::NotFound(format!(
        "could not find any book with ISBN \"{isbn}\""
    )))
}

/// Convenience wrapper around [generate_biblatex_isbn] that builds a [reqwest::Client] from an
/// optional `user_agent`.
pub async fn generate_biblatex_isbn_without_client(
    isbn: &str,
    user_agent: Option<&str>,
) -> crate::Result<BibLaTeXISBN> {
    let client = crate::create_client(user_agent, None)?;
    generate_biblatex_isbn(isbn, &client).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_validates() {
        assert_eq!(normalize_isbn("978-0-262-03384-8"), "9780262033848");
        assert!(is_valid_isbn13("9780262033848"));
        assert!(is_valid_isbn10("0262033844"));
        assert!(is_valid_isbn10("080442957X"));
        assert!(!is_valid_isbn13("9780262033840"));
        assert!(!is_valid_isbn10("0262033840"));
    }

    #[test]
    fn invalid_isbn_is_rejected() {
        assert!(validate_isbn("1234567890").is_err());
        assert!(validate_isbn("not-an-isbn").is_err());
        assert!(validate_isbn("978-0-262-03384-8").is_ok());
    }

    #[test]
    fn key_uses_surname_and_year() {
        assert_eq!(
            build_key(Some("Thomas H. Cormen and Charles E. Leiserson"), Some("2009"), "x"),
            "Cormen2009"
        );
    }

    #[tokio::test]
    async fn introduction_to_algorithms() -> crate::Result<()> {
        let entry =
            generate_biblatex_isbn_without_client("978-0-262-03384-8", Some("x2bib-testing"))
                .await?;
        assert_eq!(entry.entry_type, "book");
        assert_eq!(entry.isbn, "9780262033848");
        assert!(entry.title.starts_with("Introduction to Algorithms"));
        assert!(entry
            .author
            .as_deref()
            .unwrap_or_default()
            .contains("Cormen"));
        assert_eq!(entry.year.as_deref(), Some("2009"));
        Ok(())
    }
}
