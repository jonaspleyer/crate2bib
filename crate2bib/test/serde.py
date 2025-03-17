import asyncio
from crate2bib import get_biblatex


async def obtain_result():
    results = await get_biblatex(
        "serde", "1.0.219", "crate2bib-py-testing-serde-user-agent"
    )
    biblatex = results[0]
    expected = "\
@software {Tolnay2025,\n\
    author = {David Tolnay},\n\
    title = {{serde}: A generic serialization/deserialization framework},\n\
    url = {https://github.com/serde-rs/serde},\n\
    date = {2025-03-09},\n\
    version = {1.0.219},\n\
    license = {MIT OR Apache-2.0},\n\
}"
    assert biblatex == expected


async def empty_version_async():
    results = await get_biblatex(
        "cellular-raza", user_agent="crate2bib-py-testing-empty-version"
    )
    assert len(results) > 0


def test_serde_1():
    asyncio.run(obtain_result())


def test_empty_version():
    asyncio.run(empty_version_async())


if __name__ == "__main__":
    test_serde_1()
    test_empty_version()
