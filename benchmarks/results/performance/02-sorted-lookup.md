# Readable names and reversible u32: measured comparison

All warm timings below are **nanoseconds per operation**, median of process medians; brackets show the process-median interquartile range. Lower is faster.

Measured 2026-09-20T00:15:14Z on Linux-6.11.0-1016-nvidia-aarch64-with-glibc2.39 (aarch64), CPU affinity 19, MIDR 0x00000000410fd851, maximum frequency 4004000 kHz, governor performance. Rust: rustc 1.94.0 (4a4ef493e 2026-03-02); Node: v24.14.1. 7 fresh processes per cell, three measured batches per process, 100 ms warmup, 100 ms target batches.

Harness source: `a8bf2a23e7067b6b065b33729db3e53d1253bb61` (dirty: `false`). WASM artifact source: `a8bf2a23e7067b6b065b33729db3e53d1253bb61`, SHA-256 `ec4652bb3fc97127d14e5e9d0bbf29d178f50736e4b715693b8ff2f3b385161a`. Exact harness/fixture/lock hashes and samples are in the raw JSON.

Browser: Chromium headless shell 153.0.8010.12, published ESM / browserify assets over local HTTP. Compare alternatives **within a runtime**, not Rust vs Node vs Chromium.

## Two-word name generation

Shared rows use the same 749 adjectives and 333 animals (249,417 phrase states). nwords includes the harness RNG; it does not itself generate randomness. Native output uses hyphens and JavaScript output spaces. Shared RNG families are matched, but nwords makes one bounded draw and competitors two.

| Rust operation | ns/op [IQR] |
|---|---:|
| nwords/name/shared-rand08 | 40.9 [40.8–41.0] |
| names/name/shared | 55.3 [55.2–55.3] |
| nwords/name/shared-rand10 | 37.7 [37.7–37.9] |
| petname/name/shared-rand10 | 37.3 [37.3–37.8] |
| names/name/default | 54.9 [54.9–55.0] |
| petname/name/default-rand10 | 42.5 [42.4–42.7] |

| JavaScript operation | Node ns/op [IQR] | Chromium ns/op [IQR] |
|---|---:|---:|
| nwords/name/shared-math-random | 732.4 [728.7–738.8] | 1,374.8 [1,363.4–1,391.6] |
| unique-names-generator/name/shared | 163.1 [162.6–163.4] | 101.9 [101.9–102.3] |
| unique-names-generator/name/default | 165.4 [164.4–165.9] | 102.4 [102.1–102.5] |
| nwords/name/shared-crypto | 760.3 [755.3–761.1] | — |

Default capacities differ: names 1,102,644; petname 1,260,296; unique-names-generator 426,710. Those rows use each library's own vocabulary. The crypto row is informational, with no equivalent UNG row.

## Reversible u32 encoding and decoding

Every implementation roundtrips the same 4,096-ID fixture, including zero and u32::MAX. Output grammars differ. This is an equal-input-domain tradeoff, not an interchangeable-format ranking.

| Runtime / format | Words | Dictionary size | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|---:|---:|
| rust / nwords | 4 | 333 | 82.9 [82.7–83.7] | 252.4 [252.2–252.4] |
| rust / nwords-bip39-list | 3 | 2,048 | 65.4 [65.4–65.5] | 213.2 [213.2–213.6] |
| rust / mnemonic | 3 | 1,626 | 48.0 [47.9–48.3] | 52.1 [52.0–52.4] |
| node / nwords | 4 | 333 | 904.8 [900.8–909.1] | 1,180.9 [1,176.7–1,187.1] |
| node / niceware | 2 | 65,536 | 105.5 [105.3–105.8] | 526.9 [525.1–527.2] |
| chromium / nwords | 4 | 333 | 945.3 [933.8–957.1] | 1,197.8 [1,188.3–1,214.2] |
| chromium / niceware | 2 | 65,536 | 123.8 [123.5–124.5] | 529.1 [526.4–531.0] |

Mean phrase lengths in this fixture: nwords animal×4 28.46; nwords positional BIP-39 list 18.28; mnemonic 19.30; niceware 17.52 characters, including separators.

mnemonic has 1,626 ordinary words plus seven remainder markers; four-byte IDs use the ordinary-word alphabet. nwords uses binary search for sorted adjective/animal/color and English BIP-39 dictionaries and exact case-sensitive parsing. mnemonic uses a lazily built hash map and accepts non-alphabetic separators. niceware lowercases input and binary-searches its larger dictionary. Their error/validation behavior is not equivalent.

## Binding diagnostics

The native JSON diagnostic includes shape resolution, codec construction and JSON output. The public JS/WASM API uses the binding shipped in the measured artifact; see the stage write-up for changes to preparation and result transport. Ratios do **not** isolate pure WASM overhead.

| Layer (animal×4) | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|
| Reused native Rust codec | 82.9 [82.7–83.7] | 252.4 [252.2–252.4] |
| Native Rust JSON ABI (JSON output) | 252.5 [252.2–253.4] | 391.6 [390.9–392.1] |
| Public Node JS/WASM API | 904.8 [900.8–909.1] | 1,180.9 [1,176.7–1,187.1] |

## Fresh-process Node startup

30 fresh processes per package, warm OS file cache, compile cache disabled. Initial entry resolution is excluded equally. Internal total is module loading/evaluation + explicit initialization + first operation. Whole-process wall time includes Node startup and output. These are different first operations (names versus a reversible four-byte encoding).

| Package | Import ms | Explicit init ms | First op ms | Internal total ms [IQR] | Process wall ms |
|---|---:|---:|---:|---:|---:|
| nwords | 2.24 | 17.18 | 0.915 | 20.63 [20.27–20.76] | 40.65 |
| unique-names-generator | 1.95 | 0.00 | 0.087 | 2.05 [1.92–2.62] | 22.24 |
| niceware | 15.87 | 0.00 | 0.046 | 15.92 [15.15–16.24] | 36.93 |

Empty Node process median wall time: 14.68 ms. Native mnemonic first decode, including lazy index construction: 80.06 µs (30 fresh processes). Neither is subtracted from other timings.

## Actual browser payload and npm package size

These are the bytes served by this benchmark, not minimal bundler output. niceware carries its published Buffer shim; UNG ESM includes its published dictionaries. Harness files are excluded.

| Served implementation | Uncompressed bytes |
|---|---:|
| nwords ESM + WASM | 146,469 |
| unique-names-generator ESM | 68,631 |
| niceware browserify bundle | 874,306 |

The nwords tarball is 96,016 compressed bytes, including licenses. Installed comparator package bytes (excluding transitives): unique-names-generator 850,278, niceware 1,704,575. These package sizes are not comparable to the served-byte column.

## All warm diagnostics and variability

| Runtime / operation | Median ns/op | IQR | Min–max process median |
|---|---:|---:|---:|
| chromium / niceware/u32/decode | 529.1 | 526.4–531.0 | 524.5–532.5 |
| chromium / niceware/u32/encode | 123.8 | 123.5–124.5 | 122.7–125.5 |
| chromium / nwords/name/shared-math-random | 1,374.8 | 1,363.4–1,391.6 | 1,352.7–1,416.8 |
| chromium / nwords/pair/encode | 745.4 | 739.7–749.6 | 737.4–752.6 |
| chromium / nwords/u32/decode | 1,197.8 | 1,188.3–1,214.2 | 1,186.4–1,217.7 |
| chromium / nwords/u32/encode | 945.3 | 933.8–957.1 | 930.0–967.4 |
| chromium / rng/math-random/one-draw | 3.9 | 3.9–4.0 | 3.9–4.0 |
| chromium / rng/math-random/two-draws | 7.6 | 7.5–7.6 | 7.5–7.6 |
| chromium / unique-names-generator/name/default | 102.4 | 102.1–102.5 | 102.1–103.1 |
| chromium / unique-names-generator/name/shared | 101.9 | 101.9–102.3 | 101.9–105.2 |
| node / niceware/u32/decode | 526.9 | 525.1–527.2 | 516.8–528.4 |
| node / niceware/u32/encode | 105.5 | 105.3–105.8 | 104.8–105.9 |
| node / nwords/name/shared-crypto | 760.3 | 755.3–761.1 | 754.0–765.9 |
| node / nwords/name/shared-math-random | 732.4 | 728.7–738.8 | 719.7–740.2 |
| node / nwords/pair/encode | 685.1 | 683.0–692.6 | 676.3–694.3 |
| node / nwords/u32/decode | 1,180.9 | 1,176.7–1,187.1 | 1,169.6–1,196.3 |
| node / nwords/u32/encode | 904.8 | 900.8–909.1 | 897.6–921.7 |
| node / rng/math-random/one-draw | 4.5 | 4.5–4.6 | 4.5–4.6 |
| node / rng/math-random/two-draws | 9.2 | 9.2–9.2 | 9.2–9.3 |
| node / unique-names-generator/name/default | 165.4 | 164.4–165.9 | 163.9–168.1 |
| node / unique-names-generator/name/shared | 163.1 | 162.6–163.4 | 162.4–164.7 |
| rust / mnemonic/u32/decode | 52.1 | 52.0–52.4 | 52.0–94.5 |
| rust / mnemonic/u32/encode | 48.0 | 47.9–48.3 | 47.8–48.4 |
| rust / names/name/default | 54.9 | 54.9–55.0 | 54.6–55.3 |
| rust / names/name/shared | 55.3 | 55.2–55.3 | 55.1–55.5 |
| rust / names/setup/default | 3.5 | 3.5–3.5 | 3.5–3.5 |
| rust / nwords-abi/u32/decode | 391.6 | 390.9–392.1 | 390.5–393.2 |
| rust / nwords-abi/u32/encode | 252.5 | 252.2–253.4 | 251.9–253.4 |
| rust / nwords-bip39-list/u32/decode | 213.2 | 213.2–213.6 | 213.0–213.7 |
| rust / nwords-bip39-list/u32/encode | 65.4 | 65.4–65.5 | 65.2–65.9 |
| rust / nwords/name/shared-rand08 | 40.9 | 40.8–41.0 | 40.6–41.2 |
| rust / nwords/name/shared-rand10 | 37.7 | 37.7–37.9 | 37.6–38.2 |
| rust / nwords/pair/encode | 52.3 | 52.3–52.6 | 52.2–53.3 |
| rust / nwords/setup/pair | 20.8 | 20.5–20.8 | 20.1–21.3 |
| rust / nwords/u32/decode | 252.4 | 252.2–252.4 | 252.1–252.5 |
| rust / nwords/u32/encode | 82.9 | 82.7–83.7 | 82.4–84.3 |
| rust / petname/name/default-rand10 | 42.5 | 42.4–42.7 | 42.2–43.2 |
| rust / petname/name/shared-rand10 | 37.3 | 37.3–37.8 | 37.2–38.4 |
| rust / petname/setup/default | 2.0 | 2.0–2.3 | 1.9–2.4 |
| rust / rng/rand08/one-draw | 6.7 | 6.6–6.7 | 6.6–6.7 |
| rust / rng/rand08/two-draws | 21.1 | 20.9–21.2 | 20.8–21.2 |
| rust / rng/rand10/one-draw | 3.4 | 3.4–3.4 | 3.4–3.4 |
| rust / rng/rand10/two-draws | 6.2 | 6.2–6.2 | 6.2–6.2 |

Variability flag (IQR greater than 25% of median): no cells. No rounds were discarded.

| Round | Host load average before → after (1 min) | Selected CPU busy fraction |
|---|---:|---:|
| 1 | 10.87 → 9.91 | 100.0% |
| 2 | 9.91 → 13.52 | 100.0% |
| 3 | 13.52 → 12.06 | 100.0% |
| 4 | 12.06 → 10.98 | 100.0% |
| 5 | 10.98 → 10.56 | 100.0% |
| 6 | 10.56 → 12.16 | 100.0% |
| 7 | 12.16 → 11.73 | 100.0% |

This is one shared aarch64 host. CPU affinity limits migration but does not eliminate contention, frequency changes or GC. Selected-CPU activity includes this benchmark and cannot isolate competing work. 7 process rounds provide a descriptive estimate, not a statistical proof or a portable performance guarantee. No measured operation includes registry collision checking or persistence.

See benchmarks/README.md for methodology and comparator sources. Re-run on the intended deployment host before making a performance-sensitive choice.
