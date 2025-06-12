/// Tries to obtain a bibtex entry from a given [DOI](https://www.doi.org/)
pub async fn get_bibtex_doi(
    doi: &str,
    client: reqwest::Client,
) -> crate::Result<Option<biblatex::Bibliography>> {
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
    if let Ok(bib) = res.text().await {
        Ok(Some(biblatex::Bibliography::parse(&bib)?))
    } else {
        Ok(None)
    }
}
