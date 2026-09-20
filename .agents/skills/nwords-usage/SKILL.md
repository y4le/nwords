---
name: nwords-usage
description: >
  Use when helping someone use nwords as a consumer: running the nwords CLI,
  encoding or decoding IDs, text, or bytes, choosing presets, estimating word
  counts or dictionary sizes for a concrete use case, interpreting capacity,
  range, slack, or acceptance ratio, or integrating the nwords crate into a
  Rust project. For changing nwords internals, APIs, codecs, wordlists, or
  architecture, use nwords-architecture instead.
---

# nwords Usage Skill

Use this skill for consumer-facing guidance: CLI commands, Rust integration,
preset selection, and capacity estimates for concrete use cases.

Before quoting exact commands or APIs, prefer live sources over memory:

- `README.md` for current examples and feature list.
- `cargo run -p nwords-cli -- help` or the help constants in
  `crates/nwords-cli/src/lib.rs` for CLI syntax.
- Rust public exports in `crates/nwords/src/lib.rs` for import paths.
- `docs/research/stats-helper-notes.md` only when the math needs more detail.

## Core Guardrails

- The CLI's `bip39-en-positional` dictionary uses BIP-39 English words as a
  positional/base-2048 dictionary. It is not a BIP-39 wallet mnemonic.
- Spread presets make nearby assigned IDs look less visually similar. They are
  deterministic affine permutations, not encryption, privacy, or extra entropy.
- Capacity is the number of phrase states. Accepted ID range is the domain,
  usually `[0, range)`. Slack is `capacity - range`. Acceptance ratio is
  `range / capacity`.
- Do not call sequential, assigned, user-chosen, or biased IDs secure or
  high-entropy. Entropy bits apply only when IDs are uniformly sampled from the
  stated range.
- Text encoding is byte-exact UTF-8 with no default Unicode normalization.

## CLI Quick Reference

Run from the workspace with `cargo run -p nwords-cli -- ...`. If installed,
replace that prefix with `nwords`.

Encode and decode numeric IDs:

```sh
cargo run -p nwords-cli -- encode 42 --preset dec6
cargo run -p nwords-cli -- decode "<phrase>" --preset dec6
```

Use spread when visual adjacency matters for assigned IDs:

```sh
cargo run -p nwords-cli -- encode 42 --preset dec6-spread
cargo run -p nwords-cli -- decode "<phrase>" --preset dec6-spread
```

Inspect presets and planning data:

```sh
cargo run -p nwords-cli -- presets
cargo run -p nwords-cli -- plan --preset u32
cargo run -p nwords-cli -- plan --range 1000000
cargo run -p nwords-cli -- encode 42 --preset dec6 --explain
```

Encode arbitrary text and bytes:

```sh
cargo run -p nwords-cli -- text encode "hello"
cargo run -p nwords-cli -- text decode "<words>"
cargo run -p nwords-cli -- bytes encode --hex deadbeef
cargo run -p nwords-cli -- bytes decode --hex "<words>"
```

`word-bytes-v1` is:

```text
32-bit big-endian byte length + payload bytes + zero pad bits to 11-bit boundary
```

## Preset Selection

Use the smallest preset whose range covers the application domain:

| Preset | Accepted IDs | Words | Use |
|---|---:|---:|---|
| `dec6` | `0..1_000_000` | 2 | six decimal digits, short tickets |
| `dec9` | `0..1_000_000_000` | 3 | up to nine decimal digits |
| `u32` | `0..2^32` | 3 | 32-bit IDs |
| `u64` | `0..2^64` | 6 | 64-bit IDs |
| `dec18` | `0..10^18` | 6 | 18 decimal digits |

Each has a `-spread` sibling with the same range, word count, capacity, slack,
and acceptance ratio. Use `-spread` only to reduce obvious visual adjacency in
phrases for nearby assigned IDs.

For custom numeric shapes:

```sh
cargo run -p nwords-cli -- encode 123 --range 1000000 --words 2
cargo run -p nwords-cli -- decode "<words>" --range 1000000 --words 2
```

## Stats Workflow

State which inputs are fixed:

- fixed range and dictionary size -> use `required_words`;
- fixed range and word count -> use `required_dictionary_size`;
- fixed dictionary size and word count -> use `representable_range`;
- several unknowns -> use `tradeoffs` or present a small candidate table.

CLI estimates:

```sh
cargo run -p nwords-cli -- plan --range 1000000
cargo run -p nwords-cli -- plan --range 1000000 --words 2
```

Rust stats APIs:

```rust
use nwords::stats::{self, PlanSolution, PlanTarget};

let solution = stats::required_words(PlanTarget::Range(1_000_000), 2048)?;
assert!(matches!(solution, PlanSolution::RequiredWords { word_count: 2, .. }));
```

For decimal or bit targets, prefer `PlanTarget::DecimalDigits(n)` or
`PlanTarget::StateBits(n)` over hand-written powers when that matches the user
question.

## Rust Integration

Use default features for the standard CLI-like surface:

```toml
nwords = { version = "0.1" }
```

For dependency-minimal `no_std + alloc` consumers, disable defaults and enable
only the needed surfaces:

```toml
nwords = { version = "0.1", default-features = false, features = ["alloc", "positional", "stats"] }
```

Positional IDs over a custom dictionary:

```rust
use nwords::{core::Linear, positional::Positional};

const WORDS: &[&str] = &["zero", "one", "two", "three", "four", "five"];
let codec = Positional::new(Linear::new(WORDS), 3, 216)?;
let phrase = codec.encode(42)?;
let id = codec.decode_words(&phrase.split_whitespace().collect::<Vec<_>>())?;
assert_eq!(id, 42);
```

Built-in BIP-39 English as a positional dictionary:

```rust
use nwords::{positional::Positional, wordlists::bip39::English};

let codec = Positional::new(English, 3, 1u128 << 32)?;
let phrase = codec.encode(42)?;
```

Arbitrary text/bytes:

```rust
use nwords::{word_bytes::WordBytes, wordlists::bip39::English};

let codec = WordBytes::new(English);
let phrase = codec.encode_text("hello")?;
let text = codec.decode_text(&phrase.split_whitespace().collect::<Vec<_>>())?;
assert_eq!(text, "hello");
```

## BIP-39 Usage

Use `nwords::bip39` only for real BIP-39 entropy/mnemonic behavior. BIP-39
word counts are fixed at 12, 15, 18, 21, and 24 words. Do not recommend
non-spec BIP-39 word counts.

When the goal is a short assigned ID phrase, use positional presets instead of
BIP-39 mnemonic APIs.

## Static Web Demo

The static demo source lives in `site/`. Names use the package's ESM
`variable-v1` BigInt codec and selected wordset modules; English BIP-39 uses
the package's dedicated Rust WebAssembly binding. Both panels show their
package imports beside the conversion.

Build it before serving locally:

```sh
npm ci --prefix packages/nwords-js --ignore-scripts
npm run build --prefix packages/nwords-js
npm run build:site --prefix packages/nwords-js
python3 -m http.server 8787 --bind 127.0.0.1 --directory dist/site
```

The GitHub Pages workflow builds and tests the package and uploads `dist/site`.
The Bitcoin panel uses actual BIP-39 entropy and checksum, while the full JS
entry's `bip39-en` list remains a positional dictionary. Names preserve exact
bits; byte and text views require whole bytes and strict UTF-8. Encoding does
not encrypt, assign uniqueness, or add entropy.

## Response Checklist

When answering a use-case question:

1. Clarify or infer the ID range, whether IDs are uniformly random or assigned,
   and whether text/bytes or numeric IDs are being encoded.
2. Recommend a preset or custom shape.
3. Report range, word count, capacity, slack, and acceptance ratio when useful.
4. Include the BIP-39 positional caveat when BIP-39 words are involved.
5. Avoid security claims unless the sampling model actually supports entropy
   language.
