# Authored Mood Wordlist

This fixture pins the authored `mood` list used by `nwords`.

Source posture:

- Authored in-repo; no upstream list is transformed into the shipped words.
- Seed references were used only as review aids for common affective adjectives.
- Seed references:
  - Princeton WordNet license and lexical database:
    <https://wordnet.princeton.edu/license-and-commercial-use>
  - Wikidata licensing for structured data:
    <https://www.wikidata.org/wiki/Wikidata:Licensing>

Policy:

- Keep lowercase ASCII single-token words.
- Freeze words in alphabetical order.
- Prefer friendly, positive, neutral, or gently reflective moods.
- Exclude insulting, sexual, violent, intoxication, illness, alarm, and severe
  distress terms.
- Avoid terms that primarily describe diagnosis, protected class, age, or
  status rather than a phrase-friendly mood.

Count:

- `mood`: 64 authored words.

Regenerating, editing, or reordering this list is a new wordlist version, not
an in-place compatibility edit.
