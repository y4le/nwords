# Authored Weather Wordlist

This fixture pins the authored `weather` list used by `nwords`.

Source posture:

- Authored in-repo; no upstream list is transformed into the shipped words.
- Seed references were used only as review aids for common weather descriptors.
- Seed references:
  - National Weather Service glossary:
    <https://forecast.weather.gov/glossary.php>
  - National Weather Service disclaimer and public-domain statement:
    <https://www.weather.gov/index.php/disclaimer>
  - Wikidata licensing for structured data:
    <https://www.wikidata.org/wiki/Wikidata:Licensing>

Policy:

- Keep lowercase ASCII single-token words.
- Freeze words in alphabetical order.
- Prefer common weather and outdoor-condition modifiers.
- Exclude disaster, warning, emergency, lethal, and highly technical
  meteorology terms.
- Avoid terms that imply active danger as a default user-facing phrase.

Count:

- `weather`: 40 authored words.

Regenerating, editing, or reordering this list is a new wordlist version, not
an in-place compatibility edit.
