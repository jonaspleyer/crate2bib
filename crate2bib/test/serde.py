import asyncio
from crate2bib import get_biblatex


async def obtain_result():
    results = await get_biblatex(
        "serde", "1.0", "crate2bib-py-testing-serde-user-agent"
    )
    biblatex, origin = results[0]
    expected = "\
@software {Tolnay2024,\n\
    author = {David Tolnay},\n\
    title = {{serde} ({1.0.217}): A generic serialization/deserialization framework},\n\
    url = {https://github.com/serde-rs/serde},\n\
    date = {2024-12-27},\n\
}"
    assert biblatex == expected
    assert origin == 0


async def test_empty_version_async():
    results = await get_biblatex(
        "cellular-raza", user_agent="crate2bib-py-testing-empty-version"
    )
    assert len(results) > 0


def test_serde_1():
    asyncio.run(obtain_result())


def test_empty_version():
    asyncio.run(test_empty_version_async())


if __name__ == "__main__":
    test_serde_1()
    test_empty_version()
