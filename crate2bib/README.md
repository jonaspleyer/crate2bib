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
3. CLI tool [crate2bib-cli](https://github.com/jonaspleyer/tree/main/crate2bib)
4. Rust crate [crate2bib](https://crates.io/crate2bib)

```Python
def get_biblatex(
        crate_name: str,
        semver: Optional[str] = None,
        user_agent: Optional[str] = None,
        branch_name: Optional[str] = None,
        filenames: Sequence[str] = ['CITATION.cff', 'citation.bib'],
    ) -> typing.Any:
    r"""
    Wraps the [crate2bib::get_biblatex] function.

    Args:
        crate_name: Name of the crate to get BibLaTeX entry
        version: A semver-compliant version number for the crate
        user_agent: The name of the user agent. Defaults to None.
        branch_name: Name of the branch where to look for citaiton files.
        filenames: Filenames to search for within repository.
    Returns:
        list: A list of formatted BibLaTeX entries.
    """
    ...
```

## Example Usage

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

