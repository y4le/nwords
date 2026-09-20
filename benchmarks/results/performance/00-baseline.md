# Readable names and reversible u32: measured comparison

All warm timings below are **nanoseconds per operation**, median of process medians; brackets show the process-median interquartile range. Lower is faster.

Measured 2026-09-19T23:40:41Z on Linux-6.11.0-1016-nvidia-aarch64-with-glibc2.39 (aarch64), CPU affinity 19, MIDR 0x00000000410fd851, maximum frequency 4004000 kHz, governor performance. Rust: rustc 1.94.0 (4a4ef493e 2026-03-02); Node: v24.14.1. 7 fresh processes per cell, three measured batches per process, 100 ms warmup, 100 ms target batches.

Harness source: `5b7b1e646a5505c93a6fc6f37cd23387b38c1f20` (dirty: `false`). WASM artifact source: `83db656230feeb4b6ece2c8984ee69869a701303`, SHA-256 `b9c0f62c12917017517a6ac4b1c2bee7c30c0e150b5a273e5f674d56a48e655a`. Exact harness/fixture/lock hashes and samples are in the raw JSON.

Browser: Chromium headless shell 153.0.8010.12, published ESM / browserify assets over local HTTP. Compare alternatives **within a runtime**, not Rust vs Node vs Chromium.

## Two-word name generation

Shared rows use the same 749 adjectives and 333 animals (249,417 phrase states). nwords includes the harness RNG; it does not itself generate randomness. Native output uses hyphens and JavaScript output spaces. Shared RNG families are matched, but nwords makes one bounded draw and competitors two.

| Rust operation | ns/op [IQR] |
|---|---:|
| nwords/name/shared-rand08 | 40.9 [40.8–40.9] |
| names/name/shared | 55.1 [55.0–55.3] |
| nwords/name/shared-rand10 | 37.7 [37.6–37.8] |
| petname/name/shared-rand10 | 37.8 [37.7–37.8] |
| names/name/default | 54.6 [54.4–54.7] |
| petname/name/default-rand10 | 43.5 [43.4–43.7] |

| JavaScript operation | Node ns/op [IQR] | Chromium ns/op [IQR] |
|---|---:|---:|
| nwords/name/shared-math-random | 772.1 [770.6–777.6] | 1,432.8 [1,430.5–1,436.2] |
| unique-names-generator/name/shared | 163.8 [163.4–164.1] | 102.3 [102.2–102.9] |
| unique-names-generator/name/default | 165.6 [164.7–165.8] | 102.7 [102.4–103.2] |
| nwords/name/shared-crypto | 797.9 [796.8–800.0] | — |

Default capacities differ: names 1,102,644; petname 1,260,296; unique-names-generator 426,710. Those rows use each library's own vocabulary. The crypto row is informational, with no equivalent UNG row.

## Reversible u32 encoding and decoding

Every implementation roundtrips the same 4,096-ID fixture, including zero and u32::MAX. Output grammars differ. This is an equal-input-domain tradeoff, not an interchangeable-format ranking.

| Runtime / format | Words | Dictionary size | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|---:|---:|
| rust / nwords | 4 | 333 | 83.4 [83.1–83.5] | 394.0 [392.2–394.1] |
| rust / nwords-bip39-list | 3 | 2,048 | 66.0 [66.0–66.5] | 1,612.8 [1,611.2–1,614.7] |
| rust / mnemonic | 3 | 1,626 | 49.9 [49.7–50.4] | 52.3 [52.3–52.6] |
| node / nwords | 4 | 333 | 962.9 [959.3–969.6] | 1,802.5 [1,799.4–1,816.9] |
| node / niceware | 2 | 65,536 | 105.9 [105.3–106.2] | 528.5 [527.8–529.7] |
| chromium / nwords | 4 | 333 | 1,010.9 [996.8–1,022.3] | 2,650.5 [2,642.1–2,667.2] |
| chromium / niceware | 2 | 65,536 | 123.4 [123.3–123.6] | 529.5 [528.0–531.0] |

Mean phrase lengths in this fixture: nwords animal×4 28.46; nwords positional BIP-39 list 18.28; mnemonic 19.30; niceware 17.52 characters, including separators.

mnemonic has 1,626 ordinary words plus seven remainder markers; four-byte IDs use the ordinary-word alphabet. nwords uses linear dictionary lookup and exact case-sensitive parsing. mnemonic uses a lazily built hash map and accepts non-alphabetic separators. niceware lowercases input and binary-searches its larger dictionary. Their error/validation behavior is not equivalent.

## Binding diagnostics

The native JSON binding reparses the shape and constructs codecs per call, then creates JSON output. The public JS/WASM path adds validation, conversion, marshaling and parsing. The table localizes costs; ratios do **not** isolate pure WASM overhead.

| Layer (animal×4) | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|
| Reused native Rust codec | 83.4 [83.1–83.5] | 394.0 [392.2–394.1] |
| Native Rust JSON ABI (JSON output) | 276.3 [276.2–279.3] | 550.5 [550.4–550.6] |
| Public Node JS/WASM API | 962.9 [959.3–969.6] | 1,802.5 [1,799.4–1,816.9] |

## Fresh-process Node startup

30 fresh processes per package, warm OS file cache, compile cache disabled. Initial entry resolution is excluded equally. Internal total is module loading/evaluation + explicit initialization + first operation. Whole-process wall time includes Node startup and output. These are different first operations (names versus a reversible four-byte encoding).

| Package | Import ms | Explicit init ms | First op ms | Internal total ms [IQR] | Process wall ms |
|---|---:|---:|---:|---:|---:|
| nwords | 2.22 | 17.55 | 0.937 | 20.74 [20.42–21.09] | 41.13 |
| unique-names-generator | 2.00 | 0.00 | 0.087 | 2.08 [2.00–2.49] | 22.53 |
| niceware | 15.88 | 0.00 | 0.047 | 15.92 [15.24–16.41] | 37.45 |

Empty Node process median wall time: 15.29 ms. Native mnemonic first decode, including lazy index construction: 80.86 µs (30 fresh processes). Neither is subtracted from other timings.

## Actual browser payload and npm package size

These are the bytes served by this benchmark, not minimal bundler output. niceware carries its published Buffer shim; UNG ESM includes its published dictionaries. Harness files are excluded.

| Served implementation | Uncompressed bytes |
|---|---:|
| nwords ESM + WASM | 144,889 |
| unique-names-generator ESM | 68,631 |
| niceware browserify bundle | 874,306 |

The nwords tarball is 95,752 compressed bytes, including licenses. Installed comparator package bytes (excluding transitives): unique-names-generator 850,278, niceware 1,704,575. These package sizes are not comparable to the served-byte column.

## All warm diagnostics and variability

| Runtime / operation | Median ns/op | IQR | Min–max process median |
|---|---:|---:|---:|
| chromium / niceware/u32/decode | 529.5 | 528.0–531.0 | 525.3–534.4 |
| chromium / niceware/u32/encode | 123.4 | 123.3–123.6 | 122.8–123.7 |
| chromium / nwords/name/shared-math-random | 1,432.8 | 1,430.5–1,436.2 | 1,404.6–1,440.4 |
| chromium / nwords/pair/encode | 791.9 | 785.1–799.9 | 779.0–804.9 |
| chromium / nwords/u32/decode | 2,650.5 | 2,642.1–2,667.2 | 2,635.2–2,668.8 |
| chromium / nwords/u32/encode | 1,010.9 | 996.8–1,022.3 | 983.4–1,027.7 |
| chromium / rng/math-random/one-draw | 3.9 | 3.9–3.9 | 3.9–4.0 |
| chromium / rng/math-random/two-draws | 7.5 | 7.5–7.6 | 7.5–7.6 |
| chromium / unique-names-generator/name/default | 102.7 | 102.4–103.2 | 102.3–103.6 |
| chromium / unique-names-generator/name/shared | 102.3 | 102.2–102.9 | 102.1–103.4 |
| node / niceware/u32/decode | 528.5 | 527.8–529.7 | 524.5–530.4 |
| node / niceware/u32/encode | 105.9 | 105.3–106.2 | 104.7–107.4 |
| node / nwords/name/shared-crypto | 797.9 | 796.8–800.0 | 794.4–803.2 |
| node / nwords/name/shared-math-random | 772.1 | 770.6–777.6 | 765.2–782.0 |
| node / nwords/pair/encode | 726.7 | 723.9–729.0 | 715.0–732.5 |
| node / nwords/u32/decode | 1,802.5 | 1,799.4–1,816.9 | 1,784.5–1,826.3 |
| node / nwords/u32/encode | 962.9 | 959.3–969.6 | 951.6–979.0 |
| node / rng/math-random/one-draw | 4.6 | 4.5–4.6 | 4.5–4.6 |
| node / rng/math-random/two-draws | 9.2 | 9.2–9.3 | 9.2–9.3 |
| node / unique-names-generator/name/default | 165.6 | 164.7–165.8 | 164.3–166.4 |
| node / unique-names-generator/name/shared | 163.8 | 163.4–164.1 | 162.8–164.8 |
| rust / mnemonic/u32/decode | 52.3 | 52.3–52.6 | 52.0–53.3 |
| rust / mnemonic/u32/encode | 49.9 | 49.7–50.4 | 49.4–50.7 |
| rust / names/name/default | 54.6 | 54.4–54.7 | 54.3–54.9 |
| rust / names/name/shared | 55.1 | 55.0–55.3 | 54.9–55.3 |
| rust / names/setup/default | 3.5 | 3.5–3.5 | 3.5–3.5 |
| rust / nwords-abi/u32/decode | 550.5 | 550.4–550.6 | 549.3–554.6 |
| rust / nwords-abi/u32/encode | 276.3 | 276.2–279.3 | 275.9–305.1 |
| rust / nwords-bip39-list/u32/decode | 1,612.8 | 1,611.2–1,614.7 | 1,609.4–1,616.4 |
| rust / nwords-bip39-list/u32/encode | 66.0 | 66.0–66.5 | 65.6–67.3 |
| rust / nwords/name/shared-rand08 | 40.9 | 40.8–40.9 | 40.7–42.6 |
| rust / nwords/name/shared-rand10 | 37.7 | 37.6–37.8 | 37.5–37.8 |
| rust / nwords/pair/encode | 52.5 | 52.3–52.9 | 52.2–53.2 |
| rust / nwords/setup/pair | 20.8 | 20.3–20.9 | 20.0–20.9 |
| rust / nwords/u32/decode | 394.0 | 392.2–394.1 | 390.7–397.2 |
| rust / nwords/u32/encode | 83.4 | 83.1–83.5 | 82.9–84.6 |
| rust / petname/name/default-rand10 | 43.5 | 43.4–43.7 | 43.1–44.7 |
| rust / petname/name/shared-rand10 | 37.8 | 37.7–37.8 | 37.6–37.8 |
| rust / petname/setup/default | 2.2 | 2.0–2.4 | 1.9–2.5 |
| rust / rng/rand08/one-draw | 6.6 | 6.6–6.6 | 6.6–6.7 |
| rust / rng/rand08/two-draws | 21.1 | 21.0–21.2 | 21.0–23.2 |
| rust / rng/rand10/one-draw | 3.4 | 3.4–3.4 | 3.4–3.4 |
| rust / rng/rand10/two-draws | 6.2 | 6.2–6.2 | 6.2–6.2 |

Variability flag (IQR greater than 25% of median): no cells. No rounds were discarded.

| Round | Host load average before → after (1 min) | Selected CPU busy fraction |
|---|---:|---:|
| 1 | 7.81 → 9.78 | 100.0% |
| 2 | 9.78 → 10.63 | 100.0% |
| 3 | 10.63 → 10.45 | 100.0% |
| 4 | 10.45 → 11.06 | 100.0% |
| 5 | 11.06 → 10.13 | 100.0% |
| 6 | 10.13 → 9.62 | 100.0% |
| 7 | 9.62 → 9.76 | 100.0% |

This is one shared aarch64 host. CPU affinity limits migration but does not eliminate contention, frequency changes or GC. Selected-CPU activity includes this benchmark and cannot isolate competing work. 7 process rounds provide a descriptive estimate, not a statistical proof or a portable performance guarantee. No measured operation includes registry collision checking or persistence.

See benchmarks/README.md for methodology and comparator sources. Re-run on the intended deployment host before making a performance-sensitive choice.
