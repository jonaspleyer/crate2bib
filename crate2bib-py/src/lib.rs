use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn get_bibtex(py: Python, crate_name: String, semver: String) -> PyResult<Bound<PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let bibtex = crate2bib::get_bibtex(&crate_name, &semver)
            .await
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{e}")))?;
        Ok(format!("{}", bibtex))
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn crate2bib_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_bibtex, m)?)?;
    Ok(())
}
