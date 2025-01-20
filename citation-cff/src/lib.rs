// #![deny(missing_docs)]
// TODO enable this
//! This is a parser crate for the `.cff` citation file format.
//! See https://citation-file-format.github.io/

use serde::{Deserialize, Serialize};

/// Describes all possible entries fora single author
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Author {
    pub family_names: String,
    pub given_names: String,
    pub orcid: String,
}

/// Contains all keys of a CFF file
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CitationCff {
    pub cff_version: String,
    pub message: String,
    pub authors: Vec<Author>,
}

impl CitationCff {
    pub fn parse(input: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO this is just a placeholder; the format is not really yaml but something else!
        Ok(serde_yaml::from_str(input)?)
    }
}

#[test]
fn test_cellular_raza() {}
