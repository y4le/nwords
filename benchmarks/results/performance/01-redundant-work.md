# Readable names and reversible u32: measured comparison

All warm timings below are **nanoseconds per operation**, median of process medians; brackets show the process-median interquartile range. Lower is faster.

Measured 2026-09-20T00:09:07Z on Linux-6.11.0-1016-nvidia-aarch64-with-glibc2.39 (aarch64), CPU affinity 19, MIDR 0x00000000410fd851, maximum frequency 4004000 kHz, governor performance. Rust: rustc 1.94.0 (4a4ef493e 2026-03-02); Node: v24.14.1. 7 fresh processes per cell, three measured batches per process, 100 ms warmup, 100 ms target batches.

Harness source: `a5c76043a6f23a844a405ae968730102a877f360` (dirty: `false`). WASM artifact source: `a5c76043a6f23a844a405ae968730102a877f360`, SHA-256 `f67088633f8cf10ad70120b8597e72a0ef0cb65a12b1d7f9ba1728bce8519669`. Exact harness/fixture/lock hashes and samples are in the raw JSON.

Browser: Chromium headless shell 153.0.8010.12, published ESM / browserify assets over local HTTP. Compare alternatives **within a runtime**, not Rust vs Node vs Chromium.

## Two-word name generation

Shared rows use the same 749 adjectives and 333 animals (249,417 phrase states). nwords includes the harness RNG; it does not itself generate randomness. Native output uses hyphens and JavaScript output spaces. Shared RNG families are matched, but nwords makes one bounded draw and competitors two.

| Rust operation | ns/op [IQR] |
|---|---:|
| nwords/name/shared-rand08 | 40.7 [40.5–40.8] |
| names/name/shared | 55.1 [54.8–55.2] |
| nwords/name/shared-rand10 | 37.6 [37.5–37.8] |
| petname/name/shared-rand10 | 37.7 [37.7–37.7] |
| names/name/default | 54.4 [54.4–54.5] |
| petname/name/default-rand10 | 43.4 [43.2–43.5] |

| JavaScript operation | Node ns/op [IQR] | Chromium ns/op [IQR] |
|---|---:|---:|
| nwords/name/shared-math-random | 728.2 [721.2–730.2] | 1,389.3 [1,387.4–1,391.2] |
| unique-names-generator/name/shared | 163.6 [162.9–164.3] | 101.9 [101.8–101.9] |
| unique-names-generator/name/default | 165.3 [164.4–165.4] | 102.5 [102.3–102.7] |
| nwords/name/shared-crypto | 761.4 [756.6–764.9] | — |

Default capacities differ: names 1,102,644; petname 1,260,296; unique-names-generator 426,710. Those rows use each library's own vocabulary. The crypto row is informational, with no equivalent UNG row.

## Reversible u32 encoding and decoding

Every implementation roundtrips the same 4,096-ID fixture, including zero and u32::MAX. Output grammars differ. This is an equal-input-domain tradeoff, not an interchangeable-format ranking.

| Runtime / format | Words | Dictionary size | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|---:|---:|
| rust / nwords | 4 | 333 | 84.7 [84.5–85.5] | 392.3 [390.2–394.5] |
| rust / nwords-bip39-list | 3 | 2,048 | 66.8 [66.2–67.0] | 1,613.4 [1,612.6–1,613.9] |
| rust / mnemonic | 3 | 1,626 | 50.4 [50.2–50.5] | 52.3 [52.1–52.3] |
| node / nwords | 4 | 333 | 909.5 [900.1–911.2] | 1,364.5 [1,360.0–1,372.6] |
| node / niceware | 2 | 65,536 | 105.3 [105.0–105.8] | 523.6 [521.0–526.4] |
| chromium / nwords | 4 | 333 | 936.9 [931.2–940.7] | 1,310.7 [1,305.0–1,330.6] |
| chromium / niceware | 2 | 65,536 | 123.7 [123.5–123.8] | 527.6 [526.2–529.1] |

Mean phrase lengths in this fixture: nwords animal×4 28.46; nwords positional BIP-39 list 18.28; mnemonic 19.30; niceware 17.52 characters, including separators.

mnemonic has 1,626 ordinary words plus seven remainder markers; four-byte IDs use the ordinary-word alphabet. nwords uses linear dictionary lookup and exact case-sensitive parsing. mnemonic uses a lazily built hash map and accepts non-alphabetic separators. niceware lowercases input and binary-searches its larger dictionary. Their error/validation behavior is not equivalent.

## Binding diagnostics

The native JSON binding reparses the shape and constructs codecs per call, then creates JSON output. The public JS/WASM path adds validation, conversion, marshaling and parsing. The table localizes costs; ratios do **not** isolate pure WASM overhead.

| Layer (animal×4) | Encode ns/op [IQR] | Decode ns/op [IQR] |
|---|---:|---:|
| Reused native Rust codec | 84.7 [84.5–85.5] | 392.3 [390.2–394.5] |
| Native Rust JSON ABI (JSON output) | 255.4 [255.1–255.7] | 526.7 [525.5–528.4] |
| Public Node JS/WASM API | 909.5 [900.1–911.2] | 1,364.5 [1,360.0–1,372.6] |

## Fresh-process Node startup

30 fresh processes per package, warm OS file cache, compile cache disabled. Initial entry resolution is excluded equally. Internal total is module loading/evaluation + explicit initialization + first operation. Whole-process wall time includes Node startup and output. These are different first operations (names versus a reversible four-byte encoding).

| Package | Import ms | Explicit init ms | First op ms | Internal total ms [IQR] | Process wall ms |
|---|---:|---:|---:|---:|---:|
| nwords | 2.13 | 17.28 | 0.913 | 20.27 [20.03–20.53] | 40.32 |
| unique-names-generator | 1.94 | 0.00 | 0.086 | 2.03 [1.97–2.15] | 22.42 |
| niceware | 15.70 | 0.00 | 0.045 | 15.75 [15.12–16.35] | 36.97 |

Empty Node process median wall time: 15.17 ms. Native mnemonic first decode, including lazy index construction: 81.47 µs (30 fresh processes). Neither is subtracted from other timings.

## Actual browser payload and npm package size

These are the bytes served by this benchmark, not minimal bundler output. niceware carries its published Buffer shim; UNG ESM includes its published dictionaries. Harness files are excluded.

| Served implementation | Uncompressed bytes |
|---|---:|
| nwords ESM + WASM | 145,067 |
| unique-names-generator ESM | 68,631 |
| niceware browserify bundle | 874,306 |

The nwords tarball is 95,803 compressed bytes, including licenses. Installed comparator package bytes (excluding transitives): unique-names-generator 850,278, niceware 1,704,575. These package sizes are not comparable to the served-byte column.

## All warm diagnostics and variability

| Runtime / operation | Median ns/op | IQR | Min–max process median |
|---|---:|---:|---:|
| chromium / niceware/u32/decode | 527.6 | 526.2–529.1 | 523.0–533.3 |
| chromium / niceware/u32/encode | 123.7 | 123.5–123.8 | 123.3–125.0 |
| chromium / nwords/name/shared-math-random | 1,389.3 | 1,387.4–1,391.2 | 1,378.6–1,397.7 |
| chromium / nwords/pair/encode | 741.2 | 738.9–759.1 | 733.2–774.4 |
| chromium / nwords/u32/decode | 1,310.7 | 1,305.0–1,330.6 | 1,297.8–1,361.8 |
| chromium / nwords/u32/encode | 936.9 | 931.2–940.7 | 923.2–956.0 |
| chromium / rng/math-random/one-draw | 3.9 | 3.9–4.0 | 3.9–4.0 |
| chromium / rng/math-random/two-draws | 7.5 | 7.5–7.5 | 7.5–7.6 |
| chromium / unique-names-generator/name/default | 102.5 | 102.3–102.7 | 102.1–102.8 |
| chromium / unique-names-generator/name/shared | 101.9 | 101.8–101.9 | 101.8–102.0 |
| node / niceware/u32/decode | 523.6 | 521.0–526.4 | 516.8–528.2 |
| node / niceware/u32/encode | 105.3 | 105.0–105.8 | 104.5–106.1 |
| node / nwords/name/shared-crypto | 761.4 | 756.6–764.9 | 747.8–769.3 |
| node / nwords/name/shared-math-random | 728.2 | 721.2–730.2 | 718.8–734.5 |
| node / nwords/pair/encode | 678.4 | 675.9–683.6 | 673.5–689.4 |
| node / nwords/u32/decode | 1,364.5 | 1,360.0–1,372.6 | 1,355.5–1,392.2 |
| node / nwords/u32/encode | 909.5 | 900.1–911.2 | 898.1–933.3 |
| node / rng/math-random/one-draw | 4.5 | 4.5–4.6 | 4.5–4.6 |
| node / rng/math-random/two-draws | 9.2 | 9.2–9.2 | 9.2–9.2 |
| node / unique-names-generator/name/default | 165.3 | 164.4–165.4 | 163.1–167.5 |
| node / unique-names-generator/name/shared | 163.6 | 162.9–164.3 | 162.3–164.8 |
| rust / mnemonic/u32/decode | 52.3 | 52.1–52.3 | 52.0–52.4 |
| rust / mnemonic/u32/encode | 50.4 | 50.2–50.5 | 49.8–51.8 |
| rust / names/name/default | 54.4 | 54.4–54.5 | 54.3–54.5 |
| rust / names/name/shared | 55.1 | 54.8–55.2 | 54.8–55.3 |
| rust / names/setup/default | 3.5 | 3.5–3.5 | 3.5–3.5 |
| rust / nwords-abi/u32/decode | 526.7 | 525.5–528.4 | 521.6–536.3 |
| rust / nwords-abi/u32/encode | 255.4 | 255.1–255.7 | 254.9–257.1 |
| rust / nwords-bip39-list/u32/decode | 1,613.4 | 1,612.6–1,613.9 | 1,611.0–1,617.4 |
| rust / nwords-bip39-list/u32/encode | 66.8 | 66.2–67.0 | 65.6–68.0 |
| rust / nwords/name/shared-rand08 | 40.7 | 40.5–40.8 | 40.5–41.5 |
| rust / nwords/name/shared-rand10 | 37.6 | 37.5–37.8 | 37.4–37.9 |
| rust / nwords/pair/encode | 53.7 | 53.5–54.6 | 53.0–58.3 |
| rust / nwords/setup/pair | 20.1 | 20.0–20.6 | 20.0–20.8 |
| rust / nwords/u32/decode | 392.3 | 390.2–394.5 | 388.0–395.8 |
| rust / nwords/u32/encode | 84.7 | 84.5–85.5 | 84.2–86.3 |
| rust / petname/name/default-rand10 | 43.4 | 43.2–43.5 | 43.1–43.8 |
| rust / petname/name/shared-rand10 | 37.7 | 37.7–37.7 | 37.6–37.9 |
| rust / petname/setup/default | 2.3 | 2.0–2.4 | 1.9–2.4 |
| rust / rng/rand08/one-draw | 6.6 | 6.6–6.6 | 6.6–6.7 |
| rust / rng/rand08/two-draws | 21.1 | 21.0–21.2 | 20.8–21.2 |
| rust / rng/rand10/one-draw | 3.4 | 3.4–3.4 | 3.4–3.4 |
| rust / rng/rand10/two-draws | 6.2 | 6.2–6.2 | 6.2–6.2 |

Variability flag (IQR greater than 25% of median): no cells. No rounds were discarded.

| Round | Host load average before → after (1 min) | Selected CPU busy fraction |
|---|---:|---:|
| 1 | 8.99 → 9.05 | 100.0% |
| 2 | 9.05 → 9.70 | 100.0% |
| 3 | 9.70 → 9.77 | 100.0% |
| 4 | 9.77 → 9.66 | 100.0% |
| 5 | 9.66 → 11.14 | 100.0% |
| 6 | 11.14 → 12.37 | 100.0% |
| 7 | 12.37 → 11.51 | 100.0% |

This is one shared aarch64 host. CPU affinity limits migration but does not eliminate contention, frequency changes or GC. Selected-CPU activity includes this benchmark and cannot isolate competing work. 7 process rounds provide a descriptive estimate, not a statistical proof or a portable performance guarantee. No measured operation includes registry collision checking or persistence.

See benchmarks/README.md for methodology and comparator sources. Re-run on the intended deployment host before making a performance-sensitive choice.
