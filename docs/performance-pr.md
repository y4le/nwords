# Rust and JavaScript codec performance

This branch measures and reduces work in the Rust codec and its Node/browser
package while preserving word order, phrase bytes, accepted ranges, exact integer
handling, and typed errors. The experiments are recorded separately so reviewers
can distinguish individual changes from their cumulative effect.

## Measurement method

The fresh [baseline](../benchmarks/results/performance/00-baseline.md) uses the
original qualified WASM artifact and unchanged native sources at `5b7b1e6`.
The [raw observations](../benchmarks/results/performance/00-baseline.json) retain
source/artifact hashes, tool versions, CPU identity, load and all samples.

Each full comparison pins the process tree to CPU 19 on the same shared ARM host,
uses seven fresh processes per cell, warms for 100 ms, and measures three calibrated
batches targeting 100 ms. A process contributes its median batch time; the tables
report the median of the seven process medians. No rounds are discarded. The
4,096-ID fixture includes zero and u32::MAX, and every measured process checks
roundtrips and native/WASM phrase parity. Startup uses 30 fresh Node processes per
package, with a warm OS file cache and no Node compile cache.

Compare alternatives within a runtime. Native Rust, native JSON binding and public
WASM timings describe different layers; subtracting their times does not isolate
the WASM boundary. The existing comparator word counts and parsing semantics also
differ. These shared-host measurements are descriptive, not performance guarantees.

## Incremental results

Warm u32 encode/decode times are nanoseconds per ID. The existing public API uses
four animal words; the native English-list row uses three positional words, not a
BIP-39 wallet mnemonic. Detailed reports contain IQRs and all diagnostic cells.

| Stage | Native animal encode / decode | Node encode / decode | Chromium encode / decode | Native English-list decode |
|---|---:|---:|---:|---:|
| Baseline | 83.4 / 394.0 | 962.9 / 1,802.5 | 1,010.9 / 2,650.5 | 1,612.8 |

## Change log and review

The initial performance triage was challenged by Opus through Parley in the original
checkout. Fable then checked the reusable-codec and ABI design against the original
package's ownership and validation contracts (`req_consult_0106d921ffcab5ee`).
Every substantive implementation diff receives Opus review; mechanical result
records follow measurements from committed source.

## Validation

Required checks cover workspace formatting, Clippy, all-feature tests, the
no-default-feature alloc build, and frozen vector checksums. Package qualification
installs the actual tarball and tests Node, CommonJS dynamic import, TypeScript
NodeNext/Bundler declarations, Chromium, loader recovery and the existing demo.
New tests accompany each changed contract or optimization invariant.
