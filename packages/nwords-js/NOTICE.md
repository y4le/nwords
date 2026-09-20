# Packaged content and provenance

nwords code is MIT OR Apache-2.0; both license texts are included. This package is
built from the source commit and tool versions in `build.json`. A development
build can be marked dirty; it is not a qualification/release artifact.

The public API accepts adjective, animal, and color. They derive from the
MIT-licensed unique-names-generator snapshot and local filtering documented in
`notices/wordlists/adjective-animal/README.md`.

The Rust `named` feature also compiles the Glitch friendly-words descriptor and
object lists and the authored mood, material, shape, weather, plant, and food
lists. Their provenance READMEs and upstream MIT notices are included under
`notices/wordlists/` conservatively, regardless of linker elimination. The
authored lists use references as curation aids, not as transformed snapshots.

Generated JavaScript and runtime binding support come from wasm-bindgen
0.2.120 (MIT OR Apache-2.0). `notices/dependencies.json` identifies every crate
in the selected binding build's normal dependency graph, including procedural
macro tooling; corresponding license files are under `notices/dependencies/`.
This conservative notice set does not assert that build tools are linked into
the WASM. Dev-only npm test dependencies are not shipped in this package.

Rust 1.94.0's runtime is MIT OR Apache-2.0. The accompanying Rust license texts,
Unicode data license, LLVM exception, and `rust-1.94.0-stdlib.txt` conservatively
retain the installed toolchain's notices for all dependencies marked as used
in libstd, including other platforms. The notice file records its extraction
source and method; it is not a list of which platform-specific code was linked.

EFF long is Copyright Electronic Frontier Foundation, Joseph Bonneau, licensed
under CC BY 4.0. Source: https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt.
The dice-roll column was removed; every word and its original order are retained.
See notices/wordlists/eff-long/README.md and the bundled CC BY 4.0 license.
BIP-39 English is sourced from Trezor python-mnemonic; see the bundled MIT license.
