# nwords-tools

Development-only curation harness for `nwords` wordlists.

Commands:

```sh
cargo run -p nwords-tools -- check-existing
cargo run -p nwords-tools -- emit-array WORDS tests/vectors/example.txt
cargo run -p nwords-tools -- sample-shape tests/vectors/list-a.txt tests/vectors/list-b.txt
cargo run -p nwords-tools -- check-sha256 tests/vectors/SHA256SUMS
```

`check-existing` is the regression oracle for committed built-in wordlist
vectors. It re-derives direct-source lists from upstream snapshots and
blocklists, validates authored lists for token policy and ordering, and checks
the committed Rust arrays against the vector files.

The harness validates syntax and reproducibility. It does not decide whether a
word is friendly or phrase-appropriate; that remains a human review gate.
