# Readable names and reversible u32: measured comparison

All warm timings below are **nanoseconds per operation**, median of process medians; brackets show the process-median interquartile range. Lower is faster.

Measured 2026-09-20T00:45:18Z on Linux-6.11.0-1016-nvidia-aarch64-with-glibc2.39 (aarch64), CPU affinity 19, MIDR 0x00000000410fd851, maximum frequency 4004000 kHz, governor performance. Rust: rustc 1.94.0 (4a4ef493e 2026-03-02); Node: v24.14.1. 7 fresh processes per cell, three measured batches per process, 100 ms warmup, 100 ms target batches.

Harness source: `18ce12c249394359f87b096c961824cebc7308a8` (dirty: `false`). WASM artifact source: `18ce12c249394359f87b096c961824cebc7308a8`, SHA-256 `5a1d01d2ff37611d49bf2f278b77ef189454db11d39a9d4b5da4f8e39da70211`. Exact harness/fixture/lock hashes and samples are in the raw JSON.

Browser: Chromium headless shell 153.0.8010.12, published ESM / browserify assets over local HTTP. Compare alternatives **within a runtime**, not Rust vs Node vs Chromium.

## Two-word name generation

Shared rows use the same 749 adjectives and 333 animals (249,417 phrase states). nwords includes the harness RNG; it does not itself generate randomness. Native output uses hyphens and JavaScript output spaces. Shared RNG families are matched, but nwords makes one bounded draw and competitors two.

| Rust operation | ns/op [IQR] |
|---|---:|
| nwords/name/shared-rand08 | 40.8 [40.7–40.8] |
| names/name/shared | 55.3 [55.2–55.3] |
| nwords/name/shared-rand10 | 37.7 [37.6–37.7] |
| petname/name/shared-rand10 | 37.7 [37.6–37.7] |
| names/name/default | 54.8 [54.7–54.8] |
| petname/name/default-rand10 | 42.5 [42.5–43.1] |

| JavaScript operation | Node ns/op [IQR] | Chromium ns/op [IQR] |
|---|---:|---:|
| nwords/name/shared-math-random | 411.4 [408.5–413.1] | 1,009.4 [1,004.0–1,022.0] |
| unique-names-generator/name/shared | 163.3 [162.5–163.4] | 101.9 [101.9–101.9] |
| unique-names-generator/name/default | 165.5 [164.9–165.5] | 102.5 [102.3–102.7] |
| nwords/name/shared-crypto | 432.6 [427.8–433.8] | — |

Default capacities differ: names 1,102,644; petname 1,260,296; unique-names-generator 426,710. Those rows use each library's own vocabulary. The crypto row is informational, with no equivalent UNG row.

## Reversible u32 encoding and decoding

Every implementation roundtrips the same 4,096-ID fixture, including zero and u32::MAX. Output grammars differ. This is an equal-input-domain tradeoff, not an interchangeable-format ranking.

| Runtime / format | Words | Dictionary size | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|---:|---:|
| rust / nwords | 4 | 333 | 83.5 [83.0–83.7] | 251.7 [251.7–252.2] |
| rust / nwords-bip39-list | 3 | 2,048 | 67.5 [66.7–67.7] | 213.3 [213.0–213.4] |
| rust / mnemonic | 3 | 1,626 | 47.9 [47.9–48.1] | 52.2 [52.1–52.3] |
| node / nwords | 4 | 333 | 603.4 [599.8–606.9] | 699.1 [697.2–701.3] |
| node / niceware | 2 | 65,536 | 105.0 [104.7–105.2] | 525.1 [522.2–527.6] |
| chromium / nwords | 4 | 333 | 698.5 [689.5–712.2] | 669.9 [668.5–676.0] |
| chromium / niceware | 2 | 65,536 | 123.3 [123.1–123.9] | 529.5 [526.0–531.6] |

Mean phrase lengths in this fixture: nwords animal×4 28.46; nwords positional BIP-39 list 18.28; mnemonic 19.30; niceware 17.52 characters, including separators.

mnemonic has 1,626 ordinary words plus seven remainder markers; four-byte IDs use the ordinary-word alphabet. nwords uses binary search for sorted adjective/animal/color and English BIP-39 dictionaries and exact case-sensitive parsing. mnemonic uses a lazily built hash map and accepts non-alphabetic separators. niceware lowercases input and binary-searches its larger dictionary. Their error/validation behavior is not equivalent.

## Binding diagnostics

The native JSON diagnostic includes shape resolution, codec construction and JSON output. The public JS/WASM API uses the binding shipped in the measured artifact; see the stage write-up for changes to preparation and result transport. Ratios do **not** isolate pure WASM overhead.

| Layer (animal×4) | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|
| Reused native Rust codec | 83.5 [83.0–83.7] | 251.7 [251.7–252.2] |
| Native Rust JSON ABI (JSON output) | 252.3 [251.7–254.2] | 391.8 [391.0–392.3] |
| Public Node JS/WASM API | 603.4 [599.8–606.9] | 699.1 [697.2–701.3] |

## Fresh-process Node startup

30 fresh processes per package, warm OS file cache, compile cache disabled. Initial entry resolution is excluded equally. Internal total is module loading/evaluation + explicit initialization + first operation. Whole-process wall time includes Node startup and output. These are different first operations (names versus a reversible four-byte encoding).

| Package | Import ms | Explicit init ms | First op ms | Internal total ms [IQR] | Process wall ms |
|---|---:|---:|---:|---:|---:|
| nwords | 2.21 | 1.48 | 0.777 | 4.56 [4.43–5.13] | 24.48 |
| unique-names-generator | 1.96 | 0.00 | 0.087 | 2.05 [1.98–2.40] | 21.93 |
| niceware | 15.50 | 0.00 | 0.046 | 15.55 [15.14–16.12] | 36.39 |

Empty Node process median wall time: 14.90 ms. Native mnemonic first decode, including lazy index construction: 80.07 µs (30 fresh processes). Neither is subtracted from other timings.

## Actual browser payload and npm package size

These are the bytes served by this benchmark, not minimal bundler output. niceware carries its published Buffer shim; UNG ESM includes its published dictionaries. Harness files are excluded.

| Served implementation | Uncompressed bytes |
|---|---:|
| nwords ESM + WASM | 151,187 |
| unique-names-generator ESM | 68,631 |
| niceware browserify bundle | 874,306 |

The nwords tarball is 98,164 compressed bytes, including licenses. Installed comparator package bytes (excluding transitives): unique-names-generator 850,278, niceware 1,704,575. These package sizes are not comparable to the served-byte column.

## Instrumented Node loader phases

A separate fresh-process probe observes the actual public loader. These timings include instrumentation overhead; the uninstrumented startup table remains the primary measurement. Load includes its component phases, so do not sum these medians. Zero means a path was not invoked.

| Phase | Median ms |
|---|---:|
| publicImport | 2.276 |
| load | 1.561 |
| compile | 0.405 |
| instantiate | 0.000 |
| Instance | 0.080 |
| firstOperation | 0.791 |

## All warm diagnostics and variability

| Runtime / operation | Median ns/op | IQR | Min–max process median |
|---|---:|---:|---:|
| chromium / niceware/u32/decode | 529.5 | 526.0–531.6 | 525.3–534.8 |
| chromium / niceware/u32/encode | 123.3 | 123.1–123.9 | 122.6–124.4 |
| chromium / nwords/name/shared-math-random | 1,009.4 | 1,004.0–1,022.0 | 974.3–1,052.1 |
| chromium / nwords/pair/encode | 500.1 | 496.5–505.1 | 492.9–529.9 |
| chromium / nwords/u32/decode | 669.9 | 668.5–676.0 | 664.9–680.9 |
| chromium / nwords/u32/encode | 698.5 | 689.5–712.2 | 680.9–722.1 |
| chromium / rng/math-random/one-draw | 3.9 | 3.9–3.9 | 3.9–4.0 |
| chromium / rng/math-random/two-draws | 7.5 | 7.5–7.6 | 7.5–7.6 |
| chromium / unique-names-generator/name/default | 102.5 | 102.3–102.7 | 102.1–102.9 |
| chromium / unique-names-generator/name/shared | 101.9 | 101.9–101.9 | 101.9–102.0 |
| node / niceware/u32/decode | 525.1 | 522.2–527.6 | 520.2–531.0 |
| node / niceware/u32/encode | 105.0 | 104.7–105.2 | 104.0–105.8 |
| node / nwords/name/shared-crypto | 432.6 | 427.8–433.8 | 422.5–435.0 |
| node / nwords/name/shared-math-random | 411.4 | 408.5–413.1 | 407.0–415.4 |
| node / nwords/pair/encode | 394.7 | 392.4–395.4 | 387.7–401.3 |
| node / nwords/u32/decode | 699.1 | 697.2–701.3 | 692.2–707.5 |
| node / nwords/u32/encode | 603.4 | 599.8–606.9 | 596.8–614.4 |
| node / rng/math-random/one-draw | 4.6 | 4.5–4.6 | 4.5–4.6 |
| node / rng/math-random/two-draws | 9.2 | 9.2–9.2 | 9.2–9.3 |
| node / unique-names-generator/name/default | 165.5 | 164.9–165.5 | 164.3–166.1 |
| node / unique-names-generator/name/shared | 163.3 | 162.5–163.4 | 162.3–164.2 |
| rust / mnemonic/u32/decode | 52.2 | 52.1–52.3 | 52.1–52.4 |
| rust / mnemonic/u32/encode | 47.9 | 47.9–48.1 | 47.8–48.3 |
| rust / names/name/default | 54.8 | 54.7–54.8 | 54.5–54.9 |
| rust / names/name/shared | 55.3 | 55.2–55.3 | 55.1–55.4 |
| rust / names/setup/default | 3.5 | 3.5–3.5 | 3.5–3.5 |
| rust / nwords-abi/u32/decode | 391.8 | 391.0–392.3 | 390.5–397.7 |
| rust / nwords-abi/u32/encode | 252.3 | 251.7–254.2 | 251.5–260.0 |
| rust / nwords-bip39-list/u32/decode | 213.3 | 213.0–213.4 | 212.8–213.9 |
| rust / nwords-bip39-list/u32/encode | 67.5 | 66.7–67.7 | 66.5–67.8 |
| rust / nwords/name/shared-rand08 | 40.8 | 40.7–40.8 | 40.6–41.0 |
| rust / nwords/name/shared-rand10 | 37.7 | 37.6–37.7 | 37.6–37.8 |
| rust / nwords/pair/encode | 53.0 | 53.0–53.1 | 52.7–53.4 |
| rust / nwords/setup/pair | 20.8 | 20.8–20.8 | 20.2–20.9 |
| rust / nwords/u32/decode | 251.7 | 251.7–252.2 | 250.6–252.4 |
| rust / nwords/u32/encode | 83.5 | 83.0–83.7 | 82.4–84.3 |
| rust / petname/name/default-rand10 | 42.5 | 42.5–43.1 | 42.4–43.2 |
| rust / petname/name/shared-rand10 | 37.7 | 37.6–37.7 | 37.4–37.8 |
| rust / petname/setup/default | 2.0 | 1.9–2.3 | 1.9–2.4 |
| rust / rng/rand08/one-draw | 6.7 | 6.7–6.7 | 6.6–6.7 |
| rust / rng/rand08/two-draws | 21.2 | 21.2–21.2 | 21.2–21.2 |
| rust / rng/rand10/one-draw | 3.4 | 3.4–3.4 | 3.4–3.4 |
| rust / rng/rand10/two-draws | 6.2 | 6.2–6.2 | 6.2–6.2 |

Variability flag (IQR greater than 25% of median): no cells. No rounds were discarded.

| Round | Host load average before → after (1 min) | Selected CPU busy fraction |
|---|---:|---:|
| 1 | 11.43 → 12.31 | 100.0% |
| 2 | 12.31 → 13.36 | 100.0% |
| 3 | 13.36 → 13.55 | 100.0% |
| 4 | 13.55 → 12.39 | 100.0% |
| 5 | 12.39 → 10.68 | 100.0% |
| 6 | 10.68 → 11.01 | 100.0% |
| 7 | 11.01 → 12.62 | 100.0% |

This is one shared aarch64 host. CPU affinity limits migration but does not eliminate contention, frequency changes or GC. Selected-CPU activity includes this benchmark and cannot isolate competing work. 7 process rounds provide a descriptive estimate, not a statistical proof or a portable performance guarantee. No measured operation includes registry collision checking or persistence.

See benchmarks/README.md for methodology and comparator sources. Re-run on the intended deployment host before making a performance-sensitive choice.
