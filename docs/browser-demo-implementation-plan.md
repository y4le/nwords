# Browser demo implementation plan

The first release has two compact, bidirectional demos: `adjective+,animal`
for IDs and exact bits, bytes, or UTF-8; and an English BIP-39 entropy ↔
mnemonic converter. The live result and nearby JavaScript sample use the same
package entries. This plan began with Fable's review
`req_browser_plan_fable_20260920`; implementation findings are recorded in the
[backlog](browser-demo-backlog.md).

## Format and API contract

The names demo uses the existing `variable-v1` ordering with one or more
adjectives followed by one animal. With the checked-in 749-adjective and
333-animal lists, `42` is `able cardinal`. The first three-word ID is
`249417`, `able able aardvark`. Changing the maximum word count changes
acceptance only, never the phrase for an ID already in range.

A finite bitstring `b` maps to `int('1' + b) - 1`; decoding writes `id + 1`
in binary and removes the leading `1`. Empty bits, leading zeros, and exact
non-byte-aligned lengths round-trip. Hex bytes and strict UTF-8 text use this
same bit view and require whole bytes on decode. The phrase carries an ID, not
a type tag; the page has an explicit interpretation selector.

The names API is a small ESM `BigInt` implementation of `variable-v1`,
exported at `@y4le/nwords/variable`. It snapshots caller-supplied ordered lists,
validates them, and requires an explicit word bound up to 64 words (32 in the
page). It exposes
`encodeId`/`decodePhrase`; separate tree-shakeable helpers at
`@y4le/nwords/views` provide bit, byte, and text views. Tests compare outputs
with the existing Rust binding through the `u128` domain and check wider bit
seams. This was a deliberate simplification from the original additional
Rust WASM codec proposal: the native JS implementation ships no naming WASM,
and each optional dictionary has an independent ESM import. The full existing
JS binding keeps its API and behavior.

English BIP-39 stays on the Rust codec in a dedicated WASM artifact, exported
as `@y4le/nwords/bip39/{web,node}`. It accepts 16, 20, 24, 28, or 32 entropy
bytes and 12, 15, 18, 21, or 24 words, verifies checksums, and reports stable
errors without echoing input. Seed derivation and random wallet generation
are outside the page.

## Selective package and site

Generate one ESM module per naming wordset from the checked-in ordered text
snapshots. Expose explicit subpaths such as `wordsets/adjective` and
`wordsets/eff-long`; provide no eager barrel. Verify every generated list's
order against the source snapshot and Rust boundary vectors. BIP-39-only
production builds must emit only the BIP-39 WASM, with no naming words.
Names-only builds must emit no WASM and only the chosen wordsets. The combined
page can include both chunks but opens on Names and loads BIP-39 only when
selected.

Build the site through the assembled package's export map with Vite and deploy its
production output on Pages. Pin Rust, wasm-pack, Node, and npm to the package
build toolchain. Test Node and Chromium, source and production bundles, a narrow
viewport, and actual browser asset requests. Record uncompressed and gzip
sizes for the relevant output assets.

The page starts with public examples. BIP-39 input does not enter URLs,
browser storage, analytics, or console output. Fields disable autocomplete,
spellcheck, and autocapitalization. There are no third-party page assets.
The prior site's plan, spread, and `word-bytes-v1` panels move out of the
first page; they remain available through their existing Rust/full JS APIs.

## Review and acceptance

1. Prove BIP-39 against the committed Trezor English vectors in the packed
   Node consumer, then through Chromium and a BIP-39-only Vite build.
2. Compare the slim names codec with Rust vectors and boundary IDs, including
   the 127/128/129-bit seam, leading-zero bytes, Unicode, invalid UTF-8,
   malformed words, and word bounds.
3. Run package, Rust, browser, type, and site checks. Inspect emitted assets
   and requests for unused dictionary and capability code.
4. Ask Opus to review the exact staged diff. Fix material issues, add follow-up
   ideas to the backlog, and commit as Yale Thomas. Qualify a clean committed
   package build before landing.
