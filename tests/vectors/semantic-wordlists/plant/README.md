# Authored Plant Wordlist

This fixture pins the authored `plant` list used by `nwords`.

Source posture:

- Authored in-repo; no upstream list is transformed into the shipped words.
- Seed references were used only as review aids for common plant, crop, garden,
  and produce names.
- Seed references:
  - USDA PLANTS Database: <https://plants.usda.gov/home>
  - OpenFarm project: <https://github.com/openfarmcc/OpenFarm>
  - Wikidata licensing for structured data:
    <https://www.wikidata.org/wiki/Wikidata:Licensing>

Policy:

- Keep lowercase ASCII single-token words.
- Freeze words in alphabetical order.
- Prefer common, friendly plant, crop, herb, flower, tree, and garden nouns.
- Exclude Latin binomials, cultivars, drug-associated plants, notably toxic
  default terms, obscure taxonomy, person names, and multiword names.
- Exclude words that primarily read as non-plant homographs in short phrases.

Count:

- `plant`: 128 authored words.

Regenerating, editing, or reordering this list is a new wordlist version, not
an in-place compatibility edit.
