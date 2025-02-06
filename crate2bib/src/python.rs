use crate::*;

use pyo3::prelude::*;

/// Wraps the [crate2bib::get_biblatex] function.
///
/// Args:
///     crate_name(str): Name of the crate to get BibLaTeX entry
///     version (str): A semver-compliant version number for the crate
///     user_agent (:obj:`str`, optional):: The name of the user agent. Defaults to None.
/// Returns:
///     tuple: The formatted BibLaTeX entry and its origin given by [crate2bib::EntryOrigin]
#[pyfunction]
#[pyo3(
    name = "get_biblatex",
    signature = (
        crate_name,
        semver = None,
        user_agent = None,
        branch_name = None,
        filenames = vec![
            "CITATION.cff".to_string(),
            "citation.bib".to_string()
        ],
    ),
)]
fn get_biblatex_py(
    py: Python,
    crate_name: String,
    semver: Option<String>,
    user_agent: Option<String>,
    branch_name: Option<String>,
    filenames: Vec<String>,
) -> PyResult<Bound<PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let filenames = filenames.iter().map(|x| x.as_str()).collect();
        let results = get_biblatex(
            &crate_name,
            semver.as_deref(),
            user_agent.as_deref(),
            branch_name.as_deref(),
            filenames,
        )
        .await?;
        Ok(results
            .into_iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>())
    })
}

/// Wrapper of the [crate2bib] crate
#[pymodule]
fn crate2bib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_biblatex_py, m)?)?;
    m.add_class::<BibLaTeXCratesIO>()?;
    Ok(())
}
