---
title: 'crate2bib: Citing Rust crates made easy'
tags:
  - Rust
  - BibLaTeX
authors:
  - name: Jonas Pleyer
    orcid: 0009-0001-0613-7978
    affiliation: 1
affiliations:
  - index: 1
    name: Freiburg Center for Data Analysis and Modelling and AI (FDMAI), University of Freiburg, Freiburg, Germany
    ror: 0245cg223
date: 5 November 2025
bibliography: paper.bib
header-includes:
  - \usepackage{booktabs}

---

# Summary
`crate2bib` is a collection of tools designed to convert Rust crates hosted on
[crates.io](https://crates.io) into bibliography entries.
It queries the server, extracts metadata from the given crate and also searches for possible
`CITATION.cff` files within the repository that hosts the code of the crate in interest.
From these informations, it formats the provided information such as name, version, authors and
generates entries for all available candidates.
With this approach, crates can be cited easily and existing citations for published crates can be
found.
The tool can be used as a webapp, python package, command-line utility or Rust crate.

![The various components of `crate2bib` are built on a Rust crate which is reused to provide a python library, a webapp and a CLI tool.](media/logo.svg){width="60%"}

# Statement of Need
`crate2bib` aims to provide a simple, user-friendly platform to cite Rust software in research
papers, theses or technical documentation.
As the field of scientific computing continues to grow, it's of best scientific practice to
attribute contributions of libraries that underpin novel results and approaches.
Instead of having to manually compile bibliography entries or manually search for the necessary
information, which is prone to errors, `crate2bib` utilizes the published metadata which was already
provided by the authors of the software.
Furthermore, the provided methods can also be used for automation processes.
The result is saved time as well as concisely structured citations which can be readily used within
scientific publishing workflows.
Incidently, this paper already uses the provided methods extensively.

Over the course of the past years, multiple citation strategies have emerged within the \LaTeX
ecosystem.
The BibTeX format has been the de-facto standard and still remains widely used.
It was extended by natbib [@Daly2025] with more citation styles and citing commands.
Another alternative is BibLaTeX which provides a "[..] complete reimplementation of
the bibliographic facilities provided by \LaTeX" [@biblatex2025].
Tools such as [@Gerosa2017] filltex target specific areas of research, enabling queries to ADS and
INSPIRE databases.
Within the Github ecosystem, the [`CITATION.cff`](https://citation-file-format.github.io/) format
[@Druskat_Citation_File_Format_2021] was introduced that is being adapted by many maintainers.
It is only natural that conversion tools have emerged, allowing the mapping of both directions.
BibTeX2CFF [@Hahn_Convert_from_bibtex_2023] converts from BibTeX to the CFF format, while
cffconvert [@Spaaks_cffconvert_2021] acts in the reverse direction.
This project also provides tools such as an initializer [@Spaaks_cffinit_2023] to quickly create
new `CITATION.cff` files.

In order to connect the rapidly growing landscape of Rust-based software with academic publishing
standards, it is utmost helpful to have methods available which can generate
bibliography entries and aid in automation processes.

## Library Details

The `crate2bib` Rust crate is the foundational component which enables the remaining tools (webapp,
python bindings, cli utility).
It holds the core components and logic and uses the `reqwest` [@McArthur2025] crate together with
`crates_io_api` [@Herzog2025] to access information from [crates.io](https://crates.io) and
repository hosting platforms such as [github.com](https://github.com) and
[codeberg](https://codeberg.org/).
The latter crate was forked by the author [@Pleyer2025] in order to provide WASM support which was
required for the webapp but previously missing in its original version.
To parse the obtained information, `crate2bib` relies on biblatex [@Haug2025] and citeworks-cff
[@Saparelli2022].
Python bindings are produced with pyo3 [@pyo32025] and published with maturin [@maturin2025] while
the webapp is built with dioxus [@Kelley2025].

\begin{table}[H]
    \centering
    \begin{tabular}{ll}
        \toprule
        Tool & Link\\
        \midrule
        Webapp & \href{https://jonaspleyer.github.io/crate2bib/}{jonaspleyer.github.io/crate2bib/}\\
        Python Bindings & \href{https://pypi.org/project/crate2bib/}{pypi.org/project/crate2bib/}\\
        CLI tool & \href{https://crates.io/crates/crate2bib-cli}{crates.io/crates/crate2bib-cli}\\
        Rust crate & \href{https://crates.io/crates/crate2bib}{crates.io/crates/crate2bib}\\
        \bottomrule
    \end{tabular}
    \caption{List of provided tools by \texttt{crate2bib}.}
\end{table}

# References
Image credit: Python Software Foundation, Dioxus Labs, Wikimedia Commons
