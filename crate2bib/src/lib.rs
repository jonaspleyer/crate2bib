//! Search and create BibLaTeX entries for crates hosted on [crates.io](https://crates.io)
//! or retrieve them from their github repository.
//!
//! This crate can be used in a web version under
//! [jonaspleyer.github.io/crate2bib](https://jonaspleyer.github.io/crate2bib).
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod crates_io;
mod github;
#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
#[cfg(feature = "pyo3")]
mod python;
mod types;

pub use crates_io::*;
pub use github::*;
pub use types::*;
