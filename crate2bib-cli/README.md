[![Crates.io Version](https://img.shields.io/crates/v/crate2bib-cli?style=flat-square)](https://crates.io/crates/crate2bib-cli)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/crate2bib/webapp.yml?style=flat-square&label=Build)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/crate2bib/test.yml?style=flat-square&label=Test)

# crate2bib - cli

Converts a given crate and version number to a BibTeX entry.
See also the [rust library](https://crates.io/crates/crate2bib) which also provides
[python bindings](https://pypi.org/project/crate2bib/).

## Installation

```bash
cargo install crate2bib-cli
```

## Usage

```bash
$ crate2bib cellular_raza -v 0.2.3
Generated enty from crates.io information
@software {Pleyer2025,
    author = {Jonas Pleyer},
    title = {{cellular_raza}: Cellular Agent-based Modeling from a Clean Slate},
    url = {https://github.com/jonaspleyer/cellular_raza},
    date = {2025-03-13},
    version = {0.2.3},
    license = {GPL-2.0},
}
Generated from CITATION.cff file in repository 
@software {Pleyer2025,
    author = {Jonas Pleyer},
    title = {{cellular_raza}},
    url = {https://github.com/jonaspleyer/cellular_raza},
    date = {2025-02-23},
    version = {0.2.3},
    license = {GPL-2.0},
}
```

### Options

```text
Creates a BibTeX entry given a crate name and version number.Note: This crate respects semver.

Usage: crate2bib-cli [OPTIONS] <CRATE_NAME>

Arguments:
  <CRATE_NAME>  The exact name of the crate. Note that underscores are synonymous as dashes in the API of crates.io

Options:
  -v, --ver <VER>                  A semver compliant version number (eg. "1", 0.1", "0.3.38") [default: ]
  -u, --user-agent <USER_AGENT>    The name of the user-agent. Automation tools
                                   should specify this variable to specify which
                                   user generates the requests. [default: crate2bib-cli-user-agent]
      --filenames <FILENAMES>      [default: CITATION.cff citation.bib]
  -b, --branch-name <BRANCH_NAME>  [default: ]
  -h, --help                       Print help
  -V, --version                    Print version
```
