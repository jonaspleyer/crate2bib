import asyncio
from crate2bib import get_biblatex


async def obtain_result():
    bibtex, origin = await get_biblatex(
        "serde", "1.0", "crate2bib-py-testing-serde-user-agent"
    )
    expected = "\
@software {serde2024\n\
    author = {David Tolnay},\n\
    title = {{serde} ({1.0.217}): A generic serialization/deserialization framework},\n\
    url = {https://github.com/serde-rs/serde},\n\
    date = {2024-12-27},\n\
}"
    assert bibtex == expected
    assert origin == 0


def test_serde_1():
    asyncio.run(obtain_result())
