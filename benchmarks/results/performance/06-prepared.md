# Readable names and reversible u32: measured comparison

All warm timings below are **nanoseconds per operation**, median of process medians; brackets show the process-median interquartile range. Lower is faster.

Measured 2026-09-20T00:52:20Z on Linux-6.11.0-1016-nvidia-aarch64-with-glibc2.39 (aarch64), CPU affinity 19, MIDR 0x00000000410fd851, maximum frequency 4004000 kHz, governor performance. Rust: rustc 1.94.0 (4a4ef493e 2026-03-02); Node: v24.14.1. 7 fresh processes per cell, three measured batches per process, 100 ms warmup, 100 ms target batches.

Harness source: `cdb5633b0b275f91d70a4a32342ba5fcf4cf4919` (dirty: `false`). WASM artifact source: `cdb5633b0b275f91d70a4a32342ba5fcf4cf4919`, SHA-256 `8b021bbc5781d079e06e59d3985fb3dde732ffbe36876f741b2bbd1eb1d28c6d`. Exact harness/fixture/lock hashes and samples are in the raw JSON.

Browser: Chromium headless shell 153.0.8010.12, published ESM / browserify assets over local HTTP. Compare alternatives **within a runtime**, not Rust vs Node vs Chromium.

## Two-word name generation

Shared rows use the same 749 adjectives and 333 animals (249,417 phrase states). nwords includes the harness RNG; it does not itself generate randomness. Native output uses hyphens and JavaScript output spaces. Shared RNG families are matched, but nwords makes one bounded draw and competitors two.

| Rust operation | ns/op [IQR] |
|---|---:|
| nwords/name/shared-rand08 | 40.7 [40.7–40.8] |
| names/name/shared | 55.3 [55.2–55.4] |
| nwords/name/shared-rand10 | 37.7 [37.6–37.7] |
| petname/name/shared-rand10 | 37.4 [37.2–37.5] |
| names/name/default | 54.8 [54.6–54.8] |
| petname/name/default-rand10 | 42.4 [42.4–43.0] |

| JavaScript operation | Node ns/op [IQR] | Chromium ns/op [IQR] |
|---|---:|---:|
| nwords/name/shared-math-random | 409.5 [407.4–411.8] | 1,004.8 [999.8–1,011.7] |
| unique-names-generator/name/shared | 163.4 [162.5–164.1] | 102.0 [101.9–102.1] |
| unique-names-generator/name/default | 165.3 [164.5–165.5] | 102.7 [102.5–102.9] |
| nwords/name/shared-crypto | 427.1 [424.8–428.4] | — |

Default capacities differ: names 1,102,644; petname 1,260,296; unique-names-generator 426,710. Those rows use each library's own vocabulary. The crypto row is informational, with no equivalent UNG row.

## Reversible u32 encoding and decoding

Every implementation roundtrips the same 4,096-ID fixture, including zero and u32::MAX. Output grammars differ. This is an equal-input-domain tradeoff, not an interchangeable-format ranking.

| Runtime / format | Words | Dictionary size | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|---:|---:|
| rust / nwords | 4 | 333 | 82.5 [82.5–82.7] | 252.0 [251.8–252.1] |
| rust / nwords-bip39-list | 3 | 2,048 | 65.5 [65.5–65.6] | 212.5 [212.4–212.7] |
| rust / mnemonic | 3 | 1,626 | 48.3 [47.9–48.4] | 52.3 [52.0–52.3] |
| node / nwords | 4 | 333 | 602.8 [598.0–603.3] | 718.0 [711.6–720.1] |
| node / niceware | 2 | 65,536 | 105.1 [104.8–105.6] | 525.7 [525.4–528.0] |
| chromium / nwords | 4 | 333 | 686.6 [686.1–700.8] | 705.3 [703.2–706.1] |
| chromium / niceware | 2 | 65,536 | 123.3 [123.1–123.6] | 527.6 [525.3–528.1] |

Mean phrase lengths in this fixture: nwords animal×4 28.46; nwords positional BIP-39 list 18.28; mnemonic 19.30; niceware 17.52 characters, including separators.

mnemonic has 1,626 ordinary words plus seven remainder markers; four-byte IDs use the ordinary-word alphabet. nwords uses binary search for sorted adjective/animal/color and English BIP-39 dictionaries and exact case-sensitive parsing. mnemonic uses a lazily built hash map and accepts non-alphabetic separators. niceware lowercases input and binary-searches its larger dictionary. Their error/validation behavior is not equivalent.

## Binding diagnostics

The native JSON diagnostic includes shape resolution, codec construction and JSON output. The public JS/WASM API uses the binding shipped in the measured artifact; see the stage write-up for changes to preparation and result transport. Ratios do **not** isolate pure WASM overhead.

| Layer (animal×4) | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|
| Reused native Rust codec | 82.5 [82.5–82.7] | 252.0 [251.8–252.1] |
| Native Rust JSON ABI (JSON output) | 252.1 [252.0–252.3] | 392.4 [392.2–392.8] |
| Public Node JS/WASM API | 602.8 [598.0–603.3] | 718.0 [711.6–720.1] |

## Fresh-process Node startup

30 fresh processes per package, warm OS file cache, compile cache disabled. Initial entry resolution is excluded equally. Internal total is module loading/evaluation + explicit initialization + first operation. Whole-process wall time includes Node startup and output. These are different first operations (names versus a reversible four-byte encoding).

| Package | Import ms | Explicit init ms | First op ms | Internal total ms [IQR] | Process wall ms |
|---|---:|---:|---:|---:|---:|
| nwords | 2.24 | 1.58 | 0.776 | 4.62 [4.44–5.42] | 24.53 |
| unique-names-generator | 1.91 | 0.00 | 0.086 | 2.00 [1.95–2.13] | 22.03 |
| niceware | 15.64 | 0.00 | 0.046 | 15.68 [15.13–16.12] | 35.90 |

Empty Node process median wall time: 15.43 ms. Native mnemonic first decode, including lazy index construction: 81.75 µs (30 fresh processes). Neither is subtracted from other timings.

## Actual browser payload and npm package size

These are the bytes served by this benchmark, not minimal bundler output. niceware carries its published Buffer shim; UNG ESM includes its published dictionaries. Harness files are excluded.

| Served implementation | Uncompressed bytes |
|---|---:|
| nwords ESM + WASM | 162,425 |
| unique-names-generator ESM | 68,631 |
| niceware browserify bundle | 874,306 |

The nwords tarball is 102,754 compressed bytes, including licenses. Installed comparator package bytes (excluding transitives): unique-names-generator 850,278, niceware 1,704,575. These package sizes are not comparable to the served-byte column.

## Instrumented Node loader phases

A separate fresh-process probe observes the actual public loader. These timings include instrumentation overhead; the uninstrumented startup table remains the primary measurement. Load includes its component phases, so do not sum these medians. Zero means a path was not invoked.

| Phase | Median ms |
|---|---:|
| publicImport | 2.598 |
| load | 1.639 |
| compile | 0.418 |
| instantiate | 0.000 |
| Instance | 0.085 |
| firstOperation | 0.787 |

## All warm diagnostics and variability

| Runtime / operation | Median ns/op | IQR | Min–max process median |
|---|---:|---:|---:|
| chromium / niceware/u32/decode | 527.6 | 525.3–528.1 | 523.8–533.7 |
| chromium / niceware/u32/encode | 123.3 | 123.1–123.6 | 122.9–124.6 |
| chromium / nwords-prepared/name/shared-math-random | 851.4 | 847.6–854.1 | 833.9–872.0 |
| chromium / nwords-prepared/pair/encode | 359.5 | 357.6–362.8 | 355.1–365.8 |
| chromium / nwords-prepared/setup/pair | 305.9 | 305.5–307.6 | 304.6–308.6 |
| chromium / nwords-prepared/u32/decode | 413.5 | 412.8–414.7 | 411.6–416.6 |
| chromium / nwords-prepared/u32/encode | 400.9 | 399.0–402.1 | 395.2–404.4 |
| chromium / nwords/name/shared-math-random | 1,004.8 | 999.8–1,011.7 | 994.1–1,032.3 |
| chromium / nwords/pair/encode | 511.2 | 508.9–514.8 | 507.0–519.2 |
| chromium / nwords/u32/decode | 705.3 | 703.2–706.1 | 697.7–711.4 |
| chromium / nwords/u32/encode | 686.6 | 686.1–700.8 | 684.0–723.6 |
| chromium / rng/math-random/one-draw | 3.9 | 3.9–3.9 | 3.9–4.0 |
| chromium / rng/math-random/two-draws | 7.5 | 7.5–7.5 | 7.5–7.6 |
| chromium / unique-names-generator/name/default | 102.7 | 102.5–102.9 | 102.2–105.6 |
| chromium / unique-names-generator/name/shared | 102.0 | 101.9–102.1 | 101.7–102.1 |
| node / niceware/u32/decode | 525.7 | 525.4–528.0 | 525.1–529.7 |
| node / niceware/u32/encode | 105.1 | 104.8–105.6 | 104.5–112.2 |
| node / nwords-prepared/name/shared-math-random | 238.2 | 237.1–239.3 | 235.3–240.4 |
| node / nwords-prepared/pair/encode | 223.5 | 222.4–226.6 | 220.8–233.0 |
| node / nwords-prepared/setup/pair | 405.7 | 404.1–407.4 | 401.9–436.8 |
| node / nwords-prepared/u32/decode | 380.8 | 380.5–382.9 | 379.7–386.5 |
| node / nwords-prepared/u32/encode | 276.2 | 274.5–277.4 | 271.4–280.3 |
| node / nwords/name/shared-crypto | 427.1 | 424.8–428.4 | 422.0–433.4 |
| node / nwords/name/shared-math-random | 409.5 | 407.4–411.8 | 404.7–426.7 |
| node / nwords/pair/encode | 390.5 | 387.8–394.8 | 386.8–396.7 |
| node / nwords/u32/decode | 718.0 | 711.6–720.1 | 710.2–721.9 |
| node / nwords/u32/encode | 602.8 | 598.0–603.3 | 595.8–608.9 |
| node / rng/math-random/one-draw | 4.6 | 4.5–4.6 | 4.5–4.6 |
| node / rng/math-random/two-draws | 9.2 | 9.2–9.2 | 9.2–9.3 |
| node / unique-names-generator/name/default | 165.3 | 164.5–165.5 | 163.5–166.4 |
| node / unique-names-generator/name/shared | 163.4 | 162.5–164.1 | 162.1–166.9 |
| rust / mnemonic/u32/decode | 52.3 | 52.0–52.3 | 51.9–52.6 |
| rust / mnemonic/u32/encode | 48.3 | 47.9–48.4 | 47.8–48.4 |
| rust / names/name/default | 54.8 | 54.6–54.8 | 54.5–54.9 |
| rust / names/name/shared | 55.3 | 55.2–55.4 | 55.0–58.0 |
| rust / names/setup/default | 3.5 | 3.5–3.5 | 3.5–3.5 |
| rust / nwords-abi/u32/decode | 392.4 | 392.2–392.8 | 391.7–393.4 |
| rust / nwords-abi/u32/encode | 252.1 | 252.0–252.3 | 251.7–252.6 |
| rust / nwords-bip39-list/u32/decode | 212.5 | 212.4–212.7 | 212.3–214.0 |
| rust / nwords-bip39-list/u32/encode | 65.5 | 65.5–65.6 | 65.4–66.8 |
| rust / nwords/name/shared-rand08 | 40.7 | 40.7–40.8 | 40.6–40.9 |
| rust / nwords/name/shared-rand10 | 37.7 | 37.6–37.7 | 37.6–37.8 |
| rust / nwords/pair/encode | 52.5 | 52.4–52.7 | 52.3–52.8 |
| rust / nwords/setup/pair | 20.8 | 20.5–20.8 | 20.2–20.8 |
| rust / nwords/u32/decode | 252.0 | 251.8–252.1 | 251.4–252.2 |
| rust / nwords/u32/encode | 82.5 | 82.5–82.7 | 82.4–83.7 |
| rust / petname/name/default-rand10 | 42.4 | 42.4–43.0 | 42.4–44.8 |
| rust / petname/name/shared-rand10 | 37.4 | 37.2–37.5 | 37.1–37.6 |
| rust / petname/setup/default | 2.2 | 1.9–2.3 | 1.9–2.4 |
| rust / rng/rand08/one-draw | 6.7 | 6.7–6.7 | 6.7–6.7 |
| rust / rng/rand08/two-draws | 21.1 | 21.1–21.2 | 20.9–21.3 |
| rust / rng/rand10/one-draw | 3.4 | 3.4–3.4 | 3.4–3.4 |
| rust / rng/rand10/two-draws | 6.2 | 6.2–6.2 | 6.2–6.4 |

Variability flag (IQR greater than 25% of median): no cells. No rounds were discarded.

| Round | Host load average before → after (1 min) | Selected CPU busy fraction |
|---|---:|---:|
| 1 | 13.19 → 11.75 | 100.0% |
| 2 | 11.75 → 12.38 | 100.0% |
| 3 | 12.38 → 11.69 | 100.0% |
| 4 | 11.69 → 13.99 | 100.0% |
| 5 | 13.99 → 11.59 | 100.0% |
| 6 | 11.59 → 12.94 | 100.0% |
| 7 | 12.94 → 12.21 | 100.0% |

This is one shared aarch64 host. CPU affinity limits migration but does not eliminate contention, frequency changes or GC. Selected-CPU activity includes this benchmark and cannot isolate competing work. 7 process rounds provide a descriptive estimate, not a statistical proof or a portable performance guarantee. No measured operation includes registry collision checking or persistence.

See benchmarks/README.md for methodology and comparator sources. Re-run on the intended deployment host before making a performance-sensitive choice.
