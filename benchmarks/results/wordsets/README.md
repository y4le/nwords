# Flexible wordsets measurements

Focused warm measurements on Linux arm64, Node 24.14.1 and Rust 1.94.0,
with CPU 19 affinity. Nine batches per operation in one process; raw results
retain every sample and quartiles. These short development runs are not the
multi-process peer suite or portable performance guarantees. Benchmarks run
serially after builds, with 20,000 iterations per batch (100 for expensive
stateless custom calls).

The [baseline](before.json) loads the completed performance package at d768fe2.
The [new Node results](after.json) identify the actual loaded development build,
wrapper and WASM hashes. Root sourceCommit identifies the harness checkout;
build.sourceCommit identifies the loaded package. The [native results](rust.jsonl)
have separate [source and binary metadata](rust-metadata.json).

| Existing operation | Before ns | After ns |
|---|---:|---:|
| animal-u32/prepared-encode | 261.5 | 291.0 |
| animal-u32/prepared-decode | 323.5 | 344.8 |
| animal-u32/stateless-encode | 597.8 | 658.3 |

These existing operations are 6.6–11.3% slower in the captured run. Independent
Opus review reproduced the slowdown in back-to-back package runs; its cause
has not been isolated. This is a performance regression alongside the new features.

| Node operation | Median ns | Output words in sample |
|---|---:|---|
| eff-u32/prepared-encode | 283.0 | [3] |
| eff-u32/prepared-decode | 380.9 | [3] |
| eff-u32/stateless-encode | 595.1 | [3] |
| variable-u32/prepared-encode | 332.0 | [1, 3, 4] |
| variable-u32/prepared-decode | 330.3 | [1, 3, 4] |
| variable-u32/stateless-encode | 1742.4 | [1, 3, 4] |
| custom-u32/prepared-encode | 313.3 | [4] |
| custom-u32/prepared-decode | 466.1 | [4] |
| custom-u32/stateless-encode | 121730.6 | [4] |
| bytes-16/eff-long/encode | 351.2 | 11 |
| bytes-16/eff-long/decode | 1217.5 | 11 |
| bytes-16/eff-long,eff-long,eff-long/encode | 347.6 | 11 |
| bytes-16/eff-long,eff-long,eff-long/decode | 1208.2 | 11 |
| bytes-16/adjective,animal/encode | 400.8 | 17 |
| bytes-16/adjective,animal/decode | 1349.6 | 17 |
| niceware-bytes-16/encode | 178.1 | 8 |
| niceware-bytes-16/decode | 734.0 | 8 |

Prepared custom formats amortize vocabulary validation, copying and indexing.
Stateless custom calls rebuild from the supplied descriptor: use `prepare` for
repeated work. The custom ID rows reuse a 256-entry list in four positions.
Variable IDs use adjective repeats plus an animal suffix; EFF IDs use three
words. Different formats do not produce identical-language outputs.

| Native Rust operation | Median ns | Max output words |
|---|---:|---:|
| eff-u32/encode | 71.8 | 3 |
| eff-u32/decode | 269.5 | 3 |
| variable-u32/encode | 83.3 | 4 |
| variable-u32/decode | 205.4 | 4 |
| eff-bytes-16/encode | 274.0 | 11 |
| eff-bytes-16/decode | 906.5 | 11 |
| mnemonic-bytes-16/encode | 211.1 | 12 |
| mnemonic-bytes-16/decode | 166.6 | 12 |

The native variable-ID word count is the maximum for the accepted range;
its sample includes shorter phrases too. Native measurements black-box inputs
and observed outputs to inhibit elimination of the measured operations.

Byte comparisons use the same sixteen input bytes and include phrase joining
and splitting. Niceware 4.0.0 uses its fixed 65,536-word dictionary and eight
words. nwords EFF long uses 7,776 words, ten full-block words and one constant
tail word; a 65,536-entry custom dictionary would use nine words. Identical
repeated EFF slots produce identical mappings. Native mnemonic 1.1.1 uses its
own dictionary/format and twelve words. Its decoder uses a caller-provided stack
buffer; nwords returns a new Vec, so these are consumer-operation comparisons,
not isolated arithmetic measurements. The peers remain faster in these byte
rows. nwords adds mixed/custom dictionaries and exact arbitrary-length framing.
The formats are not wire-compatible, and the tail is not a checksum.

The expanded package WASM is 353,684 bytes uncompressed,
versus 139,984 in the performance package. The new codecs and
complete thirteen-list JS catalog add download size. EFF is feature-selectable
in Rust. Final clean packed qualification is separate from these development
measurements.

Reproduce after building the JS package and installing benchmark dependencies:

```sh
npm ci --prefix benchmarks/js --ignore-scripts --no-audit --no-fund
cargo build --release --manifest-path benchmarks/rust/Cargo.toml --bin wordsets
taskset -c 19 node benchmarks/wordsets.mjs dist/nwords-js/src/node.js /tmp/wordsets.json
taskset -c 19 benchmarks/rust/target/release/wordsets > /tmp/wordsets-rust.jsonl
```

Choose an available CPU on the target host. Do not benchmark concurrently with
builds or other benchmarks. Browser correctness is covered by packed tests;
these JavaScript timings are Node-only.
