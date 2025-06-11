//! TODO

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::*;

/// Type-alias for the [std::result::Result] type with custom [Err] type.
pub type Result<T> = std::result::Result<T, Err>;

/// The result of parsing a bibliography file
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlainBibLaTeX {
    /// Contains the whole bibliography with all keys.
    pub bibliography: biblatex::Bibliography,
    /// Link to the repository
    pub repository: String,
    /// Name of the file where the citation was discovered
    pub filename: String,
}

/// Envoked if a certain file or entity can not be found which should be there.
#[derive(Clone, Debug)]
pub struct NotFoundError(pub(crate) String);

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl std::error::Error for NotFoundError {}

/// Contains all errors of this [crate]
#[derive(Error, Debug)]
pub enum Err {
    /// Wraps [std::fmt::Error]
    #[error("error in formatting")]
    Format(#[from] std::fmt::Error),
    /// Wraps [crates_io_api::Error]
    #[error("error related to the crates_io_api")]
    CratesIOApi(#[from] crates_io_api::Error),
    /// Wraps [semver::Error]
    #[error("semver error; probably invalid version specification")]
    Semver(#[from] semver::Error),
    /// Wraps [reqwest::Error]
    #[error("error related to a failed request")]
    Request(#[from] reqwest::Error),
    /// Wraps [reqwest::header::InvalidHeaderValue]
    #[error("invalid header value")]
    HeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    /// Wraps [NotFoundError]
    #[error("value not found")]
    NotFound(#[from] crate::NotFoundError),
    /// Custom error for unsupported filetypes when parsing citation files.
    #[error("filetype not supported")]
    FiletypeUnsupported(String),
    /// Wraps [serde_yaml::Error]
    #[error("error during parsing of cff file")]
    CiteworksCff(#[from] serde_yaml::Error),
    /// Wraps [biblatex::ParseError]
    #[error("error during parsing of BibLaTeX file")]
    BibLaTeXParsing(biblatex::ParseError),
}

#[cfg(feature = "pyo3")]
impl From<Err> for PyErr {
    fn from(value: Err) -> Self {
        pyo3::exceptions::PyValueError::new_err(format!("{value}"))
    }
}

impl From<biblatex::ParseError> for Err {
    fn from(value: biblatex::ParseError) -> Self {
        Err::BibLaTeXParsing(value)
    }
}

/// Contains all variants of how a bib entry can be obtained
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BibLaTeX {
    /// Obtained bib entry form [crates.io](https://crates.io)
    CratesIO(BibLaTeXCratesIO),
    /// Obtained bib entry from `CITAIION.cff` inside repository.
    CITATIONCFF(citeworks_cff::Cff),
    /// Obtained bib entry directly from repository.
    Plain(PlainBibLaTeX),
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
            BibLaTeX::Plain(PlainBibLaTeX {
                bibliography,
                repository: url,
                filename,
            }) => f.write_str(&bibliography.to_biblatex_string()),
        }
    }
}

impl BibLaTeX {
    pub(crate) const fn priority(&self) -> u8 {
        use BibLaTeX::*;
        match self {
            CratesIO(_) => 20,
            CITATIONCFF(_) => 10,
            Plain(_) => 50,
        }
    }
}
