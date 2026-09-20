# Glitch Friendly Words Wordlists

These fixtures pin the source and local filtering policy for built-in English
semantic word lists derived from `glitchdotcom/friendly-words`.

Source:

- Project: `glitchdotcom/friendly-words`
- URL: <https://github.com/glitchdotcom/friendly-words>
- Commit: `f94b4639c71c26875f7684fa86a214c7f30deaad`
- Retrieved: 2026-06-01
- License: MIT, copied to
  `tests/vectors/licenses/glitch-friendly-words-MIT.LICENSE`

Files:

- `glitch-friendly-words-objects.txt` is the upstream `generated/words.json`
  `objects` array extracted to one word per line.
- `glitch-friendly-words-predicates.txt` is the upstream
  `generated/words.json` `predicates` array extracted to one word per line.
- `object-blocklist.txt` and `descriptor-blocklist.txt` are local removals.
- `nwords-objects.txt` and `nwords-descriptors.txt` are the curated lists used
  to generate the Rust arrays.

Policy:

- Keep lowercase ASCII single-token words.
- Preserve upstream order for retained words.
- Use `object` for broad friendly head nouns from the upstream object list.
- Use `descriptor` for upstream predicates that can act as phrase modifiers,
  including adjectives, participles, colors, materials, and other
  attributive-safe words.
- Remove person-role, age, weapon, poison, and other poor user-facing defaults
  from `object`.
- Remove clear adverbs, prepositions, and spelled numeric ordinals/cardinals
  that do not work as pre-noun descriptors from `descriptor`.

Counts:

- `object`: 3051 words from 3064 upstream objects.
- `descriptor`: 1437 words from 1450 upstream predicates.

Regenerating from a newer upstream source, editing a blocklist, or reordering
retained words is a new wordlist version, not an in-place compatibility edit.
