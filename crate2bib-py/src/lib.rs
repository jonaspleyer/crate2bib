use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn get_bibtex(crate_name: String, semver: String) -> PyResult<String> {
    let bibtex = crate2bib::get_bibtex(&crate_name, &semver)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{e}")))?;
    Ok(format!("{}", bibtex))
}

/// A Python module implemented in Rust.
#[pymodule]
fn crate2bib_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_bibtex, m)?)?;
    Ok(())
}
