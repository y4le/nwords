# Readable names and reversible u32: measured comparison

All warm timings below are **nanoseconds per operation**, median of process medians; brackets show the process-median interquartile range. Lower is faster.

Measured 2026-09-20T00:35:40Z on Linux-6.11.0-1016-nvidia-aarch64-with-glibc2.39 (aarch64), CPU affinity 19, MIDR 0x00000000410fd851, maximum frequency 4004000 kHz, governor performance. Rust: rustc 1.94.0 (4a4ef493e 2026-03-02); Node: v24.14.1. 7 fresh processes per cell, three measured batches per process, 100 ms warmup, 100 ms target batches.

Harness source: `2b18dfcda3fece6aa0bbc1ad830cd034c93e4708` (dirty: `false`). WASM artifact source: `2b18dfcda3fece6aa0bbc1ad830cd034c93e4708`, SHA-256 `bcdfc14fd0b51c87c38bb13b1650b986aadd28f0d411f2bad33fae76187af7a2`. Exact harness/fixture/lock hashes and samples are in the raw JSON.

Browser: Chromium headless shell 153.0.8010.12, published ESM / browserify assets over local HTTP. Compare alternatives **within a runtime**, not Rust vs Node vs Chromium.

## Two-word name generation

Shared rows use the same 749 adjectives and 333 animals (249,417 phrase states). nwords includes the harness RNG; it does not itself generate randomness. Native output uses hyphens and JavaScript output spaces. Shared RNG families are matched, but nwords makes one bounded draw and competitors two.

| Rust operation | ns/op [IQR] |
|---|---:|
| nwords/name/shared-rand08 | 40.9 [40.9–41.0] |
| names/name/shared | 55.2 [55.1–55.3] |
| nwords/name/shared-rand10 | 37.7 [37.7–37.8] |
| petname/name/shared-rand10 | 37.4 [37.3–37.7] |
| names/name/default | 55.1 [54.9–55.1] |
| petname/name/default-rand10 | 42.8 [42.3–42.9] |

| JavaScript operation | Node ns/op [IQR] | Chromium ns/op [IQR] |
|---|---:|---:|
| nwords/name/shared-math-random | 728.6 [722.5–733.8] | 1,384.0 [1,366.0–1,385.9] |
| unique-names-generator/name/shared | 163.5 [162.8–164.6] | 102.0 [101.9–102.2] |
| unique-names-generator/name/default | 165.4 [164.3–166.2] | 102.7 [102.5–103.0] |
| nwords/name/shared-crypto | 752.5 [746.3–757.0] | — |

Default capacities differ: names 1,102,644; petname 1,260,296; unique-names-generator 426,710. Those rows use each library's own vocabulary. The crypto row is informational, with no equivalent UNG row.

## Reversible u32 encoding and decoding

Every implementation roundtrips the same 4,096-ID fixture, including zero and u32::MAX. Output grammars differ. This is an equal-input-domain tradeoff, not an interchangeable-format ranking.

| Runtime / format | Words | Dictionary size | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|---:|---:|
| rust / nwords | 4 | 333 | 82.5 [82.2–82.8] | 252.3 [252.2–252.4] |
| rust / nwords-bip39-list | 3 | 2,048 | 65.5 [65.4–66.6] | 213.3 [213.2–213.8] |
| rust / mnemonic | 3 | 1,626 | 47.9 [47.8–48.3] | 52.3 [52.1–52.3] |
| node / nwords | 4 | 333 | 902.1 [897.6–906.3] | 1,181.4 [1,178.1–1,189.4] |
| node / niceware | 2 | 65,536 | 104.9 [104.8–106.1] | 524.7 [523.9–525.5] |
| chromium / nwords | 4 | 333 | 918.6 [917.1–928.9] | 1,198.6 [1,178.0–1,203.9] |
| chromium / niceware | 2 | 65,536 | 124.3 [123.3–124.3] | 526.8 [522.8–531.2] |

Mean phrase lengths in this fixture: nwords animal×4 28.46; nwords positional BIP-39 list 18.28; mnemonic 19.30; niceware 17.52 characters, including separators.

mnemonic has 1,626 ordinary words plus seven remainder markers; four-byte IDs use the ordinary-word alphabet. nwords uses binary search for sorted adjective/animal/color and English BIP-39 dictionaries and exact case-sensitive parsing. mnemonic uses a lazily built hash map and accepts non-alphabetic separators. niceware lowercases input and binary-searches its larger dictionary. Their error/validation behavior is not equivalent.

## Binding diagnostics

The native JSON diagnostic includes shape resolution, codec construction and JSON output. The public JS/WASM API uses the binding shipped in the measured artifact; see the stage write-up for changes to preparation and result transport. Ratios do **not** isolate pure WASM overhead.

| Layer (animal×4) | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|
| Reused native Rust codec | 82.5 [82.2–82.8] | 252.3 [252.2–252.4] |
| Native Rust JSON ABI (JSON output) | 252.0 [252.0–252.3] | 391.4 [390.8–391.5] |
| Public Node JS/WASM API | 902.1 [897.6–906.3] | 1,181.4 [1,178.1–1,189.4] |

## Fresh-process Node startup

30 fresh processes per package, warm OS file cache, compile cache disabled. Initial entry resolution is excluded equally. Internal total is module loading/evaluation + explicit initialization + first operation. Whole-process wall time includes Node startup and output. These are different first operations (names versus a reversible four-byte encoding).

| Package | Import ms | Explicit init ms | First op ms | Internal total ms [IQR] | Process wall ms |
|---|---:|---:|---:|---:|---:|
| nwords | 2.30 | 1.48 | 0.902 | 4.73 [4.44–5.35] | 24.51 |
| unique-names-generator | 1.98 | 0.00 | 0.086 | 2.09 [1.98–2.35] | 22.06 |
| niceware | 15.69 | 0.00 | 0.047 | 15.73 [15.07–16.28] | 35.81 |

Empty Node process median wall time: 15.02 ms. Native mnemonic first decode, including lazy index construction: 81.05 µs (30 fresh processes). Neither is subtracted from other timings.

## Actual browser payload and npm package size

These are the bytes served by this benchmark, not minimal bundler output. niceware carries its published Buffer shim; UNG ESM includes its published dictionaries. Harness files are excluded.

| Served implementation | Uncompressed bytes |
|---|---:|
| nwords ESM + WASM | 146,607 |
| unique-names-generator ESM | 68,631 |
| niceware browserify bundle | 874,306 |

The nwords tarball is 96,070 compressed bytes, including licenses. Installed comparator package bytes (excluding transitives): unique-names-generator 850,278, niceware 1,704,575. These package sizes are not comparable to the served-byte column.

## Instrumented Node loader phases

A separate fresh-process probe observes the actual public loader. These timings include instrumentation overhead; the uninstrumented startup table remains the primary measurement. Load includes its component phases, so do not sum these medians. Zero means a path was not invoked.

| Phase | Median ms |
|---|---:|
| publicImport | 2.292 |
| load | 1.547 |
| compile | 0.385 |
| instantiate | 0.000 |
| Instance | 0.079 |
| firstOperation | 0.914 |

## All warm diagnostics and variability

| Runtime / operation | Median ns/op | IQR | Min–max process median |
|---|---:|---:|---:|
| chromium / niceware/u32/decode | 526.8 | 522.8–531.2 | 521.1–534.4 |
| chromium / niceware/u32/encode | 124.3 | 123.3–124.3 | 122.9–124.6 |
| chromium / nwords/name/shared-math-random | 1,384.0 | 1,366.0–1,385.9 | 1,351.9–1,390.8 |
| chromium / nwords/pair/encode | 731.3 | 725.2–738.9 | 720.2–754.5 |
| chromium / nwords/u32/decode | 1,198.6 | 1,178.0–1,203.9 | 1,170.3–1,229.9 |
| chromium / nwords/u32/encode | 918.6 | 917.1–928.9 | 915.5–943.0 |
| chromium / rng/math-random/one-draw | 3.9 | 3.9–3.9 | 3.9–3.9 |
| chromium / rng/math-random/two-draws | 7.5 | 7.5–7.5 | 7.5–7.6 |
| chromium / unique-names-generator/name/default | 102.7 | 102.5–103.0 | 102.1–105.9 |
| chromium / unique-names-generator/name/shared | 102.0 | 101.9–102.2 | 101.7–102.9 |
| node / niceware/u32/decode | 524.7 | 523.9–525.5 | 523.0–527.0 |
| node / niceware/u32/encode | 104.9 | 104.8–106.1 | 104.5–111.5 |
| node / nwords/name/shared-crypto | 752.5 | 746.3–757.0 | 742.3–763.7 |
| node / nwords/name/shared-math-random | 728.6 | 722.5–733.8 | 714.9–739.1 |
| node / nwords/pair/encode | 679.4 | 675.9–680.7 | 675.5–692.9 |
| node / nwords/u32/decode | 1,181.4 | 1,178.1–1,189.4 | 1,173.9–1,191.3 |
| node / nwords/u32/encode | 902.1 | 897.6–906.3 | 892.6–907.1 |
| node / rng/math-random/one-draw | 4.5 | 4.5–4.5 | 4.5–4.6 |
| node / rng/math-random/two-draws | 9.3 | 9.2–9.3 | 9.2–9.3 |
| node / unique-names-generator/name/default | 165.4 | 164.3–166.2 | 163.4–166.7 |
| node / unique-names-generator/name/shared | 163.5 | 162.8–164.6 | 161.9–166.7 |
| rust / mnemonic/u32/decode | 52.3 | 52.1–52.3 | 51.9–52.5 |
| rust / mnemonic/u32/encode | 47.9 | 47.8–48.3 | 47.8–48.4 |
| rust / names/name/default | 55.1 | 54.9–55.1 | 54.8–55.3 |
| rust / names/name/shared | 55.2 | 55.1–55.3 | 55.0–55.6 |
| rust / names/setup/default | 3.5 | 3.5–3.5 | 3.5–3.5 |
| rust / nwords-abi/u32/decode | 391.4 | 390.8–391.5 | 390.5–391.5 |
| rust / nwords-abi/u32/encode | 252.0 | 252.0–252.3 | 250.8–253.1 |
| rust / nwords-bip39-list/u32/decode | 213.3 | 213.2–213.8 | 212.9–214.2 |
| rust / nwords-bip39-list/u32/encode | 65.5 | 65.4–66.6 | 65.4–66.7 |
| rust / nwords/name/shared-rand08 | 40.9 | 40.9–41.0 | 40.8–41.1 |
| rust / nwords/name/shared-rand10 | 37.7 | 37.7–37.8 | 37.7–38.6 |
| rust / nwords/pair/encode | 52.6 | 52.5–52.7 | 52.1–53.3 |
| rust / nwords/setup/pair | 20.8 | 20.0–20.8 | 19.9–20.9 |
| rust / nwords/u32/decode | 252.3 | 252.2–252.4 | 252.1–252.6 |
| rust / nwords/u32/encode | 82.5 | 82.2–82.8 | 82.0–83.0 |
| rust / petname/name/default-rand10 | 42.8 | 42.3–42.9 | 42.1–43.0 |
| rust / petname/name/shared-rand10 | 37.4 | 37.3–37.7 | 37.2–37.9 |
| rust / petname/setup/default | 2.3 | 1.9–2.3 | 1.9–2.4 |
| rust / rng/rand08/one-draw | 6.6 | 6.6–6.7 | 6.6–6.7 |
| rust / rng/rand08/two-draws | 21.1 | 21.0–21.2 | 20.8–21.2 |
| rust / rng/rand10/one-draw | 3.4 | 3.4–3.4 | 3.4–3.4 |
| rust / rng/rand10/two-draws | 6.2 | 6.2–6.2 | 6.2–6.2 |

Variability flag (IQR greater than 25% of median): no cells. No rounds were discarded.

| Round | Host load average before → after (1 min) | Selected CPU busy fraction |
|---|---:|---:|
| 1 | 11.47 → 11.76 | 100.0% |
| 2 | 11.76 → 10.52 | 100.0% |
| 3 | 10.52 → 10.62 | 100.0% |
| 4 | 10.62 → 10.42 | 100.0% |
| 5 | 10.42 → 10.76 | 100.0% |
| 6 | 10.76 → 13.23 | 100.0% |
| 7 | 13.23 → 13.80 | 100.0% |

This is one shared aarch64 host. CPU affinity limits migration but does not eliminate contention, frequency changes or GC. Selected-CPU activity includes this benchmark and cannot isolate competing work. 7 process rounds provide a descriptive estimate, not a statistical proof or a portable performance guarantee. No measured operation includes registry collision checking or persistence.

See benchmarks/README.md for methodology and comparator sources. Re-run on the intended deployment host before making a performance-sensitive choice.
