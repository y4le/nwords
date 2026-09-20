# Authored Plant Wordlist

This fixture pins the authored `plant` list used by `nwords`.

Source posture:

- Authored in-repo; no upstream list is transformed into the shipped words.
- Seed references were used only as review aids for common plant, garden,
  ornamental, flower, tree, and landscape names.
- Seed references:
  - USDA PLANTS Database: <https://plants.usda.gov/home>
  - OpenFarm project: <https://github.com/openfarmcc/OpenFarm>
  - Wikidata licensing for structured data:
    <https://www.wikidata.org/wiki/Wikidata:Licensing>

Policy:

- Keep lowercase ASCII single-token words.
- Freeze words in alphabetical order.
- Prefer common, friendly ornamental plants, flowers, trees, shrubs,
  houseplants, and garden or landscape nouns.
- Exclude Latin binomials, cultivars, drug-associated plants, notably toxic
  default terms, obscure taxonomy, words that primarily read as person names,
  and multiword names.
- Exclude words that primarily read as non-plant homographs in short phrases.
- Exclude exact entries from the `food` list. The sets are disjoint, but their
  meanings can still overlap.

Count:

- `plant`: 128 authored words.

This is a pre-publication revision of the development list added in `36a107e`,
replacing its crop/food-heavy ordering before the first published `plant`
version. It intentionally changes ID mappings: phrases created with that old
development revision must be decoded with the same revision.

After publication, regenerating, editing, or reordering this list requires a
new wordlist name or version, not an in-place compatibility edit.
