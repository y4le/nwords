# Browser demo backlog

Working draft, 2026-09-20. This backlog records the first two demos and ideas
that arise during implementation and review.

The [implementation plan](browser-demo-implementation-plan.md) records the
first-release format contract and work needed for the first two demos. It also
requires consumer builds to omit unimported dictionaries and capabilities.

## Shared experience

Keep one compact page with a demo selector. Each demo has two editable sides for
the input and its word form, inline validation, a small explanation of the
mapping, and a copyable JavaScript definition that stays in sync with the
controls. Use real package calls for both the interaction and the code example.
Show a useful example immediately, then let visitors edit it. Put format details
behind a disclosure so the first screen stays focused on the round trip.

The code panel should state the scheme and ordered wordlists. Numeric reports
should distinguish accepted range from phrase capacity; entropy language needs
an explicit sampling model. A phrase cannot recover an original input type
unless the format records that type or the decoder is told which type to use.

## Proposed first demos

### 1. Names: value ↔ `adjective+,animal`

**Moment to show:** `42` becomes `able cardinal`; pasting the phrase recovers
`42` and its exact bit view `01011`. At the next tier, `249417` becomes
`able able aardvark`; every phrase ends in an animal. The user can switch
among unsigned integer, UTF-8 text,
hex bytes, and, if exact bit lengths are supported, binary bits. Show leading-zero
bytes and a malformed phrase as deliberate examples. Keep the type and output
length visible near the result.

**Definition shown:** one `variable-v1` descriptor shared by the integer and
bit views. A later control could edit maximum words and update the copyable JS
snippet. Explain that unique
IDs map to unique phrases within the chosen range; random phrase generation
still needs an application's collision policy.

**Implementation:** The dictionary-free Names WASM uses Rust's wide
`variable-v1` mapping and exact bit view. JS loads the codec and selected
wordsets; the native CLI accepts the same wide values.

**Format decisions:**

- Use one natural-number mapping. A bitstring `b` maps to
  `int('1' + b) - 1`; decoding strips the first `1` from the binary form of
  `id + 1`. Text and hex are interpretations of those bits, selected in the
  browser. The phrase does not carry the original input type.
- For numbers, start with unsigned integers. Decide later whether signed
  integers, decimals, and JSON values belong in this demo.
- Preserve exact bit length and leading zeros; byte input alone cannot show
  that promise.
- Decide the word-count and payload limits that keep the browser interaction
  quick and the output readable.

### 2. BIP-39: entropy ↔ mnemonic

**Moment to show:** open a fresh 128-bit hex entropy value and see the corresponding
12-word English mnemonic; edit a word and see checksum validation or the
recovered entropy. Offer the other legal entropy sizes (160, 192, 224, 256
bits) without making the first screen dense. An expandable strip can show the
entropy bits, checksum bits, and 11-bit word indexes. This is the actual
[BIP-39 mapping](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki).

**Definition shown:** a JS mnemonic API call using the same Rust codec
as the live result, plus its language and version. Start with public test
vectors in tests. The live page starts with browser-generated entropy and labels
it as an educational converter. Ask visitors not to enter real wallet recovery
phrases or use demo phrases to protect funds. Keep wallet setup and seed
derivation out of the initial browser demo; those deserve a separate security
review and product decision.

**Implementation:** The dedicated JS binding calls the Rust English BIP-39
codec. The existing positional `bip39-en` wordlist remains a separate feature.

**Confirmed scope:** “bitcoin phrase” means a standards-compliant BIP-39
wallet mnemonic generated from entropy. The positional ID use of the BIP-39
vocabulary is a separate format and is not this demo.

## Candidates after the first two

### Wordset composer

Select built-in lists or paste a small custom list, arrange a fixed or variable
shape, and see a round trip, capacity, accepted range, and copyable descriptor.
This is the natural home for `mood,descriptor,food`, `eff-long`, and custom
Unicode examples. Keep it behind an “Explore formats” entry so the main page
does not open as a configuration form. The JS package already supports these
descriptors; the site does not expose them.

### Exact bytes with EFF long

Show hex or UTF-8 ↔ `radix-bytes-v1` using the EFF long list. A short example
with leading-zero bytes makes the byte framing visible. This may be useful as
a separate demo if the name codec stays focused on IDs, or as a comparison
inside the wordset composer if the name codec gains arbitrary-data support.

### Nearby IDs and spread

Keep the current nearby-ID comparison as an advanced example. It teaches how
the spread permutation changes visual adjacency for assigned IDs. The main
name demo should lead with the reversible mapping itself.

### Error and compatibility examples

Offer one-click examples for an out-of-range phrase, a wrong dictionary, a
BIP-39 checksum failure, and exact Unicode text. Put the explanation next to
the failed operation and keep the raw input out of diagnostic logs. A compact
format/version badge should make it clear which decoder owns each phrase.

## Shared UX questions

- Should the two fields update on every edit, on a short debounce, or after an
  explicit Convert action? Invalid partial input should remain editable.
- How should a visitor save the full recipe needed to decode a phrase later:
  scheme version, exact ordered lists, limits, and any type framing? The JS
  `bigint` metadata needs an intentional JSON representation.
- Keep the live demos on the public slim package entries, with copyable
  examples that resolve through the package export map.
- Which examples deserve persistent links or shareable, nonsecret URLs?
- What load and phrase-size limits keep the page responsive on mobile?

## Capability snapshot for later gap planning

| Experience | Rust core | JS package | Browser demo |
| --- | --- | --- | --- |
| `adjective+,animal` numeric IDs | Yes | Yes | Yes |
| Arbitrary bytes/text ending in animal | Yes | Slim Rust WASM binding | Yes |
| Exact binary bit length in that grammar | Yes | Slim Rust WASM binding | Yes |
| BIP-39 entropy ↔ mnemonic | Yes | Dedicated JS/WASM entry | Yes |
| Arbitrary bytes/text with cyclic lists | Yes | Yes | No |
| Custom and mixed wordsets | Yes | Yes | No |
| Import only used codecs and dictionaries | Feature gates exist; CLI embeds presets | Explicit slim WASM entries and wordset modules | Yes |

This table records current reach, not a decision to add every row to the first
release. Revisit it after the demo set and decoding semantics are agreed.

## Findings and later work

Record findings here as the implementation and Fable/Opus reviews continue.

- **Fable and implementation, 2026-09-20:** The initial tier correction assumed
  zero adjectives were allowed. For the requested `adjective+` format the
  minimum is one, so `249417` is `able able aardvark`. For minimum zero,
  the first three-word ID is `249750`. The examples and API descriptor now
  state the minimum explicitly.
- **Fable, 2026-09-20:** A separate 64-bit block format would duplicate the
  name grammar, reject many adjective edits, and skew each block's first word.
  Adopt a wide `variable-v1` integer path plus a bijective bit view instead.
- **Fable, 2026-09-20:** The current `named` feature and dynamic map retain all
  built-in dictionaries, and the full binding's runtime glue import is not
  statically discoverable by bundlers. The slim names entry uses ESM BigInt,
  and the BIP-39 entry uses a static glue import. A dictionary-free owned map
  and Rust wide-integer path remain useful if other bindings need the same
  larger domain. Their first draft was removed from this change to keep the
  shipped surface focused.
- **Fable, 2026-09-20:** Match Rust's Unicode phrase whitespace and unknown-word
  error positions, preserve an initial UTF-8 BOM on text decode, and accept
  zero-width joiners in custom tokens. Keep a differential test over the
  shared `u128` domain; test larger values against an independent tier formula.
- **Fable, 2026-09-20:** Use the existing variable-v1 descriptor shape with
  explicit `min`, put the ID codec at `./variable`, and keep bit/byte/text
  helpers in a separate `./views` entry for smaller ID-only bundles.
- **Implemented, 2026-09-20:** Rust and CLI now accept the page's wide IDs and
  exact bit, byte, and text views. The Names-only Vite bundle emits one Names
  WASM and only selected wordsets; BIP-39-only emits its separate WASM.
  Qualify webpack later.
- **Later UX:** Show a loaded-size readout per demo, with measured WASM and
  wordset transfer sizes. Include it only if it helps explain the package
  behavior without distracting from conversion.
- **Later UX:** Add unknown-word highlighting and nearest-word suggestions;
  consider BIP-39 four-letter autocomplete after reviewing input handling.
- **Later formats:** A shareable names recipe with dictionary hashes,
  wordset composer, EFF long byte demo, and nearby-ID comparison. Do not put
  BIP-39 entropy or mnemonics in shareable links.
- **Later compatibility:** Qualify webpack production builds, then decide
  whether to make an umbrella entry tree-shakeable. Keep the existing full
  entry stable while slim imports become available.
- **Later packaging:** Measure the shared codec's unused algorithm cost; split
  further if material. Inspect the release WASM name section only after the
  initial bundle-size measurements.
- **Implemented Rust-owned Names, 2026-09-20:** The slim binding has no
  named wordlists; JS passes imported lists once into a reusable Rust handle.
  The duplicate JS codec was removed after packed bundle and first-use checks.
- **Wordset distribution, 2026-09-20:** Keep browser wordsets as independent
  imports, with no built-in naming lists in the slim Names WASM artifact. The
  twelve naming snapshots total 108,633 bytes of UTF-8 (46,451 bytes when
  gzipped separately); EFF long accounts for 62,144 and 24,761 bytes. Their
  current JS modules total 138,634 bytes (50,316 bytes when gzipped
  separately). Keep the native CLI self-contained for presets and `cargo
  install`; revisit external sidecar files only if an actual distribution-size
  benchmark justifies the install and lookup complexity.
- **Fable consultation, 2026-09-20 (`wrq_5c2a5ef245994259b966bb376bdbca5b`):**
  Treat the shipped JS wide mapping as a compatibility reference and freeze
  cross-language success and error fixtures before widening Rust. Test tier
  boundaries, exact bit lengths, leading zeros, UTF-8, Unicode tokens, range
  and word limits through Rust, CLI, and WASM. Preserve the published phrase
  mapping as the JS codec is replaced.
- **Measured Names bundle, 2026-09-20:** The baseline dictionary-free WASM is
  98,139 bytes (38,394 gzip). A Names-only Vite bundle is 20,606 JS bytes
  (8,550 gzip) plus that WASM; the old JS-only bundle was 13,535 bytes
  (6,546 gzip). Five local Chromium runs took 17.9–24.8 ms from dynamic
  imports through a codec round trip, with about 3–4 ms spent loading and
  initializing WASM. These are local arm64 desktop measurements. Later:
  measure a throttled/mobile device and compare throughput; optimize the
  artifact if this affects real visitors. Fable's earlier size estimates were
  predictions, not measured budgets.
- **Fable packaging and binding details:** Pass each selected wordset through
  one preparation call and reuse an opaque Rust codec handle. Keep BIP-39's
  fixed English list inside its separate artifact. Add per-list Cargo features
  without silently changing existing default-feature behavior; retain the
  single npm package with independent wordset exports and the self-contained
  CLI. Review API parity for Unicode validation, limits, error codes, disposal,
  and bigint/byte return types.
- **Opus, 2026-09-20:** A 64-word custom format can produce a 4,159-byte
  phrase, so the decoder's byte bound must follow the encoder's allowed token
  and word limits. The site must catch a BIP-39 asset failure while a visitor
  edits entropy or mnemonic input. Both are addressed in the implementation.
- **Opus, 2026-09-20:** Keep the repository's usage skill current with the
  production site; it was updated in this change. Package qualification checks
  every generated wordset export and name and identifies the site's actual
  WASM by hash.
- **Later UX from Opus:** Add arrow-key navigation and roving focus to the two
  demo tabs, a dark color scheme, and a clear control for the BIP-39 fields.
  Check the page's console output as part of browser qualification.
- **Later maintenance from Opus:** Decide whether to retire `crates/nwords-web`
  now that Pages builds the package-backed site. Consider splitting Pages
  deployment from broad package qualification if browser timing samples or
  Playwright installation cause deploy flakes.
