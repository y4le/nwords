# nwords-tools

Development-only curation harness for `nwords` wordlists.

Commands:

```sh
cargo run -p nwords-tools -- check-existing
cargo run -p nwords-tools -- emit-array WORDS tests/vectors/example.txt
cargo run -p nwords-tools -- sample-shape tests/vectors/list-a.txt tests/vectors/list-b.txt
cargo run -p nwords-tools -- check-sha256 tests/vectors/SHA256SUMS
```

`check-existing` is the regression oracle for the current
`unique-names-generator`-derived lists. It re-derives the curated adjective and
animal lists from their upstream snapshots and blocklists, validates the color
snapshot, and checks the committed Rust arrays against the vector files.

The harness validates syntax and reproducibility. It does not decide whether a
word is friendly or phrase-appropriate; that remains a human review gate.
