# Authored Food Wordlist

This fixture pins the authored `food` list used by `nwords`.

Source posture:

- Authored in-repo; no upstream list is transformed into the shipped words.
- Seed references were used only as review aids for common food, ingredient,
  and dish names.
- Seed references:
  - USDA FoodData Central: <https://fdc.nal.usda.gov/>
  - USDA FoodData Central downloads: <https://fdc.nal.usda.gov/download-datasets>
  - Wikidata licensing for structured data:
    <https://www.wikidata.org/wiki/Wikidata:Licensing>

Policy:

- Keep lowercase ASCII single-token words.
- Freeze words in alphabetical order.
- Prefer common, friendly foods, ingredients, dishes, sauces, spices, grains,
  fruits, vegetables, sweets, and staples.
- Exclude brands, alcohol, drugs, medical or allergen framing terms, dominant
  non-food homographs, very obscure regional items, unpleasant default phrase
  terms, and multiword names.

Count:

- `food`: 128 authored words.

Regenerating, editing, or reordering this list is a new wordlist version, not
an in-place compatibility edit.
