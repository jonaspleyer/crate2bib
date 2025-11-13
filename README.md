[![Crates.io Version](https://img.shields.io/crates/v/crate2bib?style=flat-square)](https://crates.io/crates/crate2bib)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/crate2bib/webapp.yml?style=flat-square&label=Build)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/crate2bib/test.yml?style=flat-square&label=Test)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/crate2bib/maturin.yml?style=flat-square&label=Release)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/crate2bib/pytest.yml?style=flat-square&label=Pytest)
[![PyPI - Version](https://img.shields.io/pypi/v/crate2bib?style=flat-square)](https://pypi.org/project/crate2bib/)

# crate2bib

Converts a given crate and version number to a BibLaTeX entry.

The crate will try to determine if a publication is already available by scanning for possible
`CITATION.cff` files.
It will further generate entries from the information available at [crates.io](https://crates.io).
There are 4 distinct ways of using this package:

1. Webapp [jonaspleyer/.github.io/crate2bib](https://jonaspleyer.github.io/crate2bib/)
2. Python bindings [pypi.org/project/crate2bib](https://pypi.org/project/crate2bib/)
3. CLI tool [crate2bib-cli](https://github.com/jonaspleyer/crate2bib/tree/main/crate2bib-cli)
4. Rust crate [crate2bib](https://crates.io/crate2bib)

## Webapp
The webapp provides a simple interface to generate entries for particular crates with optionally
provided version numbers.
If multiple candidates for bibliography entries are available, they are all displayed.

[![](https://raw.githubusercontent.com/jonaspleyer/crate2bib/refs/heads/main/media/webapp-screenshot.png)](https://jonaspleyer.github.io/crate2bib)

## Python Example
The python bindings consist of only a single function `get_biblatex` which obtains the entry of the
specified crate.
Note that we have to provide a name for a user agent which is mandatory to access the API behind
[crates.io](https://crates.io).

```Python
import asyncio
from crate2bib import get_biblatex

async def obtain_result():
    results = await get_biblatex(
        "serde", "1.0.228", "crate2bib-py-testing-serde-user-agent"
    )
    biblatex = results[0]
    print(biblatex)
```

```
> @software {Tolnay2025,
>     author = {David Tolnay},
>     title = {{serde}: A generic serialization/deserialization framework},
>     url = {https://github.com/serde-rs/serde},
>     date = {2025-09-27},
>     version = {1.0.228},
>     license = {MIT OR Apache-2.0},
> }
```

## CLI Tool
Similarly to the python bindings, the CLI tool offers almost identical functionality.

```
Creates a BibTeX entry given a crate name and version number. Note: This crate respects semver.

Usage: crate2bib [OPTIONS] <CRATE_NAME>

Arguments:
  <CRATE_NAME>  The exact name of the crate. Note that underscores are synonymous to dashes in the API of crates.io

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

## Rust Crate
The rust crate behind the previously listed approaches is directly available at
[crates.io/crate2bib](https://crate.io/crate2bib).
It provides more functionality compared to the previous approaches and separates various sources for
possible BibLaTeX entries such as Github and crates.io.

## Citing

```bibtex
@software{Pleyer2025crate2bib,
      title={crate2bib: Citing Rust crates made easy},
      author={Jonas Pleyer},
      year={2025},
      eprint={2511.07468},
      archivePrefix={arXiv},
      primaryClass={cs.DL},
      url={https://arxiv.org/abs/2511.07468},
}
```
