import asyncio
from crate2bib_py import get_bibtex

bibtex = await get_bibtex("serde", "1.0")
print(bibtex)
