use crate::*;

use pyo3::prelude::*;

/// Wraps the [crate2bib::get_biblatex] function.
///
/// Args:
///     crate_name(str): Name of the crate to get BibLaTeX entry
///     version (str): A semver-compliant version number for the crate
///     user_agent (:obj:`str`, optional): The name of the user agent. Defaults to None.
///     branch_name(:obj:`str`, optional): Name of the branch where to look for citaiton files.
///     filenames(:obj:`list[str]`, optional): Filenames to search for within repository.
/// Returns:
///     list: A list of formatted BibLaTeX entries.
#[pyo3_stub_gen::derive::gen_stub_pyfunction]
#[pyfunction]
#[pyo3(
    name = "get_biblatex",
    signature = (
        crate_name,
        version = None,
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
    version: Option<String>,
    user_agent: Option<String>,
    branch_name: Option<String>,
    filenames: Vec<String>,
) -> PyResult<Bound<PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let filenames = filenames.iter().map(|x| x.as_str()).collect();
        let results = get_biblatex(
            &crate_name,
            version.as_deref(),
            user_agent.as_deref(),
            branch_name.as_deref(),
            filenames,
        )
        .await?;
        Ok(results
            .into_iter()
            .filter_map(|x| x.ok().map(|x| format!("{x}")))
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
