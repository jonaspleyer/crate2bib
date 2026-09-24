use serde::{Deserialize, Serialize};

/// If parsing fails, return the plain text
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BibReturn {
    /// Prased Bib file
    BibFile(biblatex::Bibliography),
    /// Plain text
    String(String),
}

/// asdf
pub async fn get_bibtex_doi_without_client(
    doi: &str,
    user_agent: Option<&str>,
) -> crate::Result<BibReturn> {
    use reqwest::header::*;
    #[cfg(feature = "log")]
    log::trace!("Prepare Headers and Client");
    let mut headers = HeaderMap::new();
    if let Some(ua) = user_agent {
        headers.insert(USER_AGENT, HeaderValue::from_str(ua)?);
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    get_bibtex_doi(doi, client).await
}

/// Tries to obtain a bibtex entry from a given [DOI](https://www.doi.org/)
pub async fn get_bibtex_doi(doi: &str, client: reqwest::Client) -> crate::Result<BibReturn> {
    // let doi = "10.1021/acs.jpcc.0c05161";
    let rq = format!("https://doi.org/{doi}");

    #[cfg(feature = "log")]
    log::trace!("Sending request to doi.org");
    let res = client
        .request(reqwest::Method::GET, rq)
        .header(reqwest::header::ACCEPT, "application/x-bibtex")
        .send()
        .await?;

    #[cfg(feature = "log")]
    log::trace!("Parsing request to biblatex");
    let bib = res.text().await?;

    // Clean up known problematic abbreviations
    let bib = bib
        .replace("month=January", "month=Jan")
        .replace("month=February", "month=Feb")
        .replace("month=March", "month=Mar")
        .replace("month=April", "month=Apr")
        .replace("month=June", "month=Jun")
        .replace("month=July", "month=Jul")
        .replace("month=August", "month=Aug")
        .replace("month=September", "month=Sep")
        .replace("month=October", "month=Oct")
        .replace("month=November", "month=Nov")
        .replace("month=December", "month=Dec");

    match biblatex::Bibliography::parse(&bib) {
        Ok(bib) => Ok(BibReturn::BibFile(bib)),
        Err(_) => Ok(BibReturn::String(bib)),
    }
}
