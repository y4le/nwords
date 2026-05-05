# nwords

`nwords` is a planned Rust library for bidirectional encoding between structured
payloads or integer IDs and symbolic phrases.

V1 is scoped to:

- BIP-39 English and Japanese compatibility.
- Positional N-word codecs.
- Capacity, dictionary-size, word-count, and ID-range planning helpers.
- Minimal dependencies and no unsafe code.

The current repository contains architecture, standards, execution planning, and
vendored test vectors. Implementation starts from
[`docs/research/execution-plan.md`](docs/research/execution-plan.md).

