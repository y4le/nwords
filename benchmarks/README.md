# Readable-name and reversible-ID benchmarks

These comparisons exercise the current Rust library and its JavaScript/WASM package
against alternatives in their own runtimes. They do not change production code or
add runtime dependencies to nwords. The Rust harness is a separate Cargo workspace.

## Recorded results

[Full report](results/2026-09-19-linux-arm64.md),
[standalone chart](results/2026-09-19-linux-arm64.svg),
[raw observations and provenance](results/2026-09-19-linux-arm64.json), and
[process summaries](results/2026-09-19-linux-arm64.summary.json).

The 2026-09-19 run used one explicitly pinned ARM CPU, seven process rounds per cell,
903 warm batches and 30 startup observations per package. All 4,096-ID roundtrip and
native/WASM parity checks passed. No cell exceeded the 25% relative-IQR flag, and
no rounds were discarded. These are measurements on a shared host, not portable guarantees.

- **Rust names:** nwords with rand 0.8 took 40.9 ns/name versus names at 55.2;
  nwords with rand 0.10 took 37.6 versus petname at 37.7. Dictionaries and RNG
  families match, although nwords draws once and the competitors twice.
- **JS names:** nwords WASM took 777.5 ns/name in Node versus UNG at 163.3;
  Chromium took 1,421.4 versus 101.9. These rows use the same dictionaries and Math.random.
- **Reversible u32:** three-word native nwords encoding took 66.9 ns and decoding
  1,613.7 ns; mnemonic took 48.6 and 52.2. Both use three words, but different
  vocabularies and parsing rules. nwords searches linearly; mnemonic builds a hash
  index whose first decode cost was 81.34 µs.
- **JS reversible u32:** niceware encoded 9.2× faster in Node and 8.1× in Chromium,
  and decoded 3.4× and 5.0× faster respectively. It uses two words from a 65,536-word
  dictionary; nwords uses four animal words from 333. The shipped browser assets
  served here were 874,306 bytes for niceware versus 144,889 for nwords, uncompressed.

The native binding diagnostic also costs more than a reused native codec. Together
with the source's per-call shape construction, validation and JSON work, this suggests
prepared codecs and a leaner JS boundary as optimization candidates; it does not
measure how much either change would save. Indexed decoding is another candidate.
No production behavior or API was changed for these measurements.

## Reproduce

Use Rust 1.94.0, Node 24.14.1, npm 11.11.0 and Python 3. Install benchmark tools:

```sh
npm ci --prefix benchmarks/js --ignore-scripts
cd benchmarks/js && npx playwright install chromium && cd ../..
```

Supply a clean nwords tarball built with the [package instructions](../packages/nwords-js/README.md).
The default is `dist/tarballs/y4le-nwords-0.1.0.tgz`. A previously qualified artifact
is accepted only if all crate and JS wrapper sources match its committed revision. Harness changes
can be uncommitted during development and are recorded with hashes and Git status.
Run from the repository root:

```sh
python3 benchmarks/run.py --tarball /path/to/y4le-nwords-0.1.0.tgz \
  --rounds 7 --sample-ms 100 --output benchmarks/.work/results.json
python3 benchmarks/report.py benchmarks/.work/results.json \
  --output benchmarks/.work/report.md
```

Run serially on an otherwise quiet machine. Linux affinity defaults to the first
available CPU; `--cpu N` chooses another allowed CPU. The whole process tree inherits
affinity. Metadata records CPU details, governor when available, load averages,
versions, dependency trees, lock hashes, source status and artifact provenance.
On heterogeneous hosts, choose the CPU explicitly: the recorded full run uses
`--cpu 19`, with its MIDR and maximum frequency printed in the report. Per-round
selected-CPU busy fractions include benchmark work, so they cannot isolate contention.
The report flags cells whose process-median IQR exceeds 25% of the median; it never
discards slower rounds. This shared host cannot provide dedicated-machine precision.
Warm batch costs include the small result-consumption loop. No timer-overhead
subtraction is performed. Tiny construction/RNG results approach the harness floor.

A quick harness smoke run uses `--rounds 3 --sample-ms 10`; it is not the published
measurement. Build and check the standalone Rust harness with:

```sh
cargo fmt --manifest-path benchmarks/rust/Cargo.toml --check
cargo clippy --manifest-path benchmarks/rust/Cargo.toml --all-targets -- -D warnings
```

## Workloads and scope

| Workload | Rust | JavaScript |
|---|---|---|
| Two-word names, same 749 adjective / 333 animal lists | harness RNG + nwords; names 0.14.0; petname 3.2.0 | harness Math.random + nwords WASM; unique-names-generator 4.7.1 |
| Default-list names | names, petname | unique-names-generator |
| Reversible u32, animal×4 | nwords codec and native binding | nwords WASM |
| Reversible u32, other formats | nwords three positional BIP-39-list words; mnemonic 1.1.1 | niceware 4.0.0, two words |
| Diagnostics | RNG one/two draws; construction; native JSON binding; first mnemonic decode | RNG one/two draws; Node crypto+nwords; fresh-process import/init/first op |

The three-word native nwords row is **positional encoding using the BIP-39 list**,
not a wallet mnemonic or checksum benchmark. The JS package currently exposes only
adjective/animal/color, so four words are needed for a u32. No BIP-39 entropy/seed,
text, networking, collision registry, or allocation-uniqueness guarantee is tested.

Names use hyphens in Rust (a Formatter for nwords) and spaces in JS. No post-process
separator replacement is timed. `names` uses rand 0.8.8; `petname` uses rand 0.10.2.
Each gets a matching nwords+harness RNG row. nwords samples one mixed-radix integer;
competitors draw once per word. RNG-only rows show these different costs. JS uses
unseeded Math.random; the crypto row uses Node randomInt and is informational.
No cryptographic quality or perfect uniformity claim is made for Math.random.
UNG uses its default style: the selected dictionaries are already lowercase, so
an explicit lowercase pass would add unnecessary work without changing the output.

Reversible rows accept the same full u32 domain. Their phrase lengths, dictionaries,
lookup algorithms and parsing rules differ; use the results as a tradeoff table.
Integer/byte adapters, splitting and string joining are timed. nwords JS inputs are
precomputed bigint values (the public API's native input); native mnemonic and JS
niceware convert numeric u32 values to four **big-endian** bytes. Native decoders use
stack arrays where their APIs permit it. Niceware uses fully overwritten allocUnsafe
buffers. Decode inputs are precomputed outside timing.

The native binding rows call the actual `nwords_js::*_json` functions with precomputed
decimal IDs. They include per-call shape/range resolution and JSON output but exclude
JS validation, JSON.parse, ID-to-decimal conversion and the WASM boundary. They are
**diagnostics**, not equivalent scalar-returning decode rows. A native/WASM ratio
must not be labeled pure WASM overhead. No unshipped pure-JS codec is benchmarked.

## Fixture and measurements

`ids-u32.txt` contains 4096 u32s generated once with the recurrence
`x = (1664525*x + 1013904223) mod 2^32`, starting at `0x12345678`, then replacing
positions 0–2 with 0, u32::MAX and 42. It is synthetic project-authored test data
under the workspace license. All implementations read this same file at runtime.
Its SHA-256 is recorded. This broad sample avoids flattering linear lookup with
only small/sequential IDs. It is not a corpus of real application workloads.

Before every measured cell, every fixture ID roundtrips and the native/WASM phrases
match byte for byte. One fresh process (or Chromium instance) runs one cell per
round. Cases are deterministically shuffled across rounds. Each cell warms for
100 ms, calibrates toward the requested batch duration (capped at 2^24 operations),
and records three batches. Results use the median of each process's three batches,
then median, IQR and min/max across independent process rounds. Calibration samples
are excluded. Output is consumed; Rust uses black_box on inputs and results.

Cold Node measurements use 30 fresh processes per package, with compile cache and
NODE_OPTIONS disabled. Initial entry resolution is excluded equally for all three packages. They separate
module loading/evaluation, explicit initialization and first
operation, and record whole-process wall time plus an empty-Node baseline. OS file
cache is warm. Native mnemonic's first decode separately records its lazy index
construction. Browser cold start is not compared because browser startup, caching,
network delivery and timer precision need a separate experiment.

Runs also collect 30 separate instrumented Node startup probes.
They observe the actual public loader's WebAssembly calls without touching Node's
lazy web globals beforehand. An optional `prewarm-web` control times those globals
separately. Phases overlap and contain observation overhead; uninstrumented cold
measurements remain authoritative. The probes verify the first encoded phrase.

Chromium serves nwords' installed ESM/WASM, UNG's published ESM and niceware's shipped
browserify bundle (including its Buffer shim). Browser payload counts describe those
actual files, not a minimal bundler build. npm installed package bytes include docs,
source maps and other unused files, exclude transitive dependencies, and are a
different metric. No native executable size is compared against npm package bytes.
There are no cross-runtime rankings or CI timing thresholds.

## Comparator sources and methodology review

- [names documentation](https://docs.rs/names/0.14.0/names/) — configurable adjective/noun generator.
- [petname source](https://github.com/allenap/rust-petname) — grammar and configurable lists.
- [unique-names-generator](https://github.com/andreasonny83/unique-names-generator) — dictionary-based names and optional deterministic seed, without reversible-ID semantics.
- [Rust mnemonic](https://github.com/mbrubeck/rust-mnemonic) — reversible bytes to words; lazy hash-map decode index.
- [niceware](https://github.com/diracdeltas/niceware) — 65,536-word byte encoding, binary-search decode and case folding.

Registry versions were checked on 2026-09-19 and pinned in both lockfiles. The benchmark
uses Rust 1.94.0 because comparison dependencies exceed the library's own Rust 1.82
minimum; this does not change the library MSRV. Opus reviewed the method through
Parley (`req_consult_c05f8b66bb77bdb0`). We adopted per-cell isolation, matching RNG
versions, shared fixtures, native binding diagnostics and native separators. The
u32 adapters retain big-endian byte order consistently in both languages.
Opus then reviewed the staged harness (`req_review_diff_4c53a70a4b1b2863`). We removed
UNG's redundant lowercase pass, recorded/reported the selected CPU identity, derived
report/plot labels from metadata, and added activity and variability observations.

For an optional standalone chart, install `requirements-plot.txt` into a separate
virtual environment and run `plot.py` with the same input and `--output chart.svg`.
Plotting dependencies do not enter either measured runtime.

`python3 benchmarks/test_report.py` checks process-level aggregation and rejects incomplete samples.
