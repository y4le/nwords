# Unique Names Generator Wordlists

These fixtures pin the source and local filtering policy for the built-in
English named word lists derived from `unique-names-generator`.

Source:

- Project: `andreasonny83/unique-names-generator`
- URL: <https://github.com/andreasonny83/unique-names-generator>
- Commit: `10ff70b131c8a080e88c315a55e45a0f5caadd24`
- Retrieved: 2026-05-21
- License: MIT, copied to
  `tests/vectors/licenses/unique-names-generator-MIT.LICENSE`

Files:

- `unique-names-generator-adjectives.txt` is the upstream
  `src/dictionaries/adjectives.ts` list extracted to one word per line.
- `unique-names-generator-animals.txt` is the upstream
  `src/dictionaries/animals.ts` list extracted to one word per line.
- `unique-names-generator-colors.txt` is the upstream
  `src/dictionaries/colors.ts` list extracted to one word per line.
- `adjective-blocklist.txt` and `animal-blocklist.txt` are the local removals.
- `nwords-adjectives.txt` and `nwords-animals.txt` are the curated lists used
  to generate the Rust arrays. The adjective list applies the length policy
  below before applying `adjective-blocklist.txt`; the animal list applies
  `animal-blocklist.txt`.
- The color list is used without local filtering.

Policy:

- Keep lowercase ASCII single-token words.
- Keep adjectives with length 3 through 11.
- Remove identity, political, religious, sexual, medical, legal, corporate, and
  administrative terms.
- Remove insults, body-shaming terms, and labels that are poor defaults for
  user-facing identifiers.
- Remove very abstract, technical, or arcane adjectives when they are unlikely
  to make a clear adjective-animal phrase.
- Remove animal terms that are taxonomy-only, awkwardly ambiguous, or fictional.
- Preserve upstream order for all retained words. Reordering changes encoded
  IDs and must be treated as a new wordlist version.
