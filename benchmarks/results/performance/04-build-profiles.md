# WASM build configuration comparison

Seven fresh processes per warm cell, three 100 ms target batches after 100 ms warmup. The order is shuffled across configurations, runtimes and operations each round. Every process validates the same 4,096 IDs against native phrases before timing. Brackets show the IQR of process medians; no rounds were discarded. Cold totals use 30 fresh processes per configuration. CPU 19; complete tool, artifact and host records are in the raw data.

All five artifacts were built and qualified from `adabff57ae81fe5d3b526b7c92ab13280d228916`. These configurations jointly change LTO and codegen units; the comparison does not isolate either setting.

| Configuration | WASM bytes | Node encode / decode ns per ID [IQR] | Chromium encode / decode ns per ID [IQR] | Node cold total ms [IQR] |
|---|---:|---:|---:|---:|
| baseline | 130,663 | 902.9 [901.4–913.2] / 1,186.8 [1,184.3–1,188.1] | 924.7 [922.4–945.7] / 1,179.5 [1,174.2–1,194.8] | 4.50 [4.34–4.86] |
| thin | 128,317 | 903.6 [902.3–905.7] / 1,190.4 [1,186.3–1,192.0] | 915.5 [913.2–927.4] / 1,185.6 [1,170.3–1,216.9] | 4.42 [4.32–4.55] |
| fat | 128,317 | 909.8 [904.0–919.9] / 1,193.2 [1,187.5–1,205.4] | 926.2 [915.9–939.9] / 1,176.5 [1,172.3–1,191.3] | 4.51 [4.37–4.98] |
| size | 134,070 | 1,038.2 [1,032.3–1,045.5] / 1,300.9 [1,288.6–1,312.5] | 1,047.5 [1,040.3–1,055.1] / 1,303.9 [1,300.0–1,305.0] | 4.68 [4.57–5.03] |
| fat-opt | 117,500 | 911.3 [907.3–915.6] / 1,209.2 [1,200.2–1,214.8] | 933.1 [927.7–935.7] / 1,223.0 [1,218.8–1,232.1] | 4.39 [4.34–5.07] |

The default remains baseline: thin/fat LTO do not show a throughput benefit. The size profile grows the binary and slows these calls. Fat LTO plus Binaryen reduces WASM bytes by about 10%, with slower measured calls; it remains an explicit payload-size tradeoff rather than the default.

Binaryen 131 came from the official aarch64 Linux release archive: [release asset](https://github.com/WebAssembly/binaryen/releases/download/version_131/binaryen-version_131-aarch64-linux.tar.gz). Archive SHA-256: `ba991f677edd9a21d2bc96c0144bc8ac5b112d4d98a3eb266e075e22e557df2a`. The executable hash and exact optimizer flags are recorded per artifact. The build script checks the version and records the executable hash; it does not authenticate arbitrary supplied binaries.

Each artifact passed packed Node/CJS/Chromium/types/demo qualification. The raw data records each qualification log hash. The archived runner accepts explicit repository and optimizer paths and reproduces this matrix.

## Review and reproduction notes

Opus reviewed the build tooling (`req_review_diff_ef513851eaf215b3`) and this
experiment's actual runner (`req_consult_e674b4e5a61bc0c0`). The default stays
unchanged; small differences between baseline, thin and fat do not establish a
throughput improvement.

The original runner is archived byte-for-byte, including its recorded SHA-256.
It is a historical experiment, not the general benchmark runner. It inherits
`benchmarks/.work/native-phrases.tsv` from a preceding standard run. To reproduce,
use a clean worktree at `adabff5`, install the locked package/benchmark tools, then
prepare that reference before invoking the archived script:

```sh
cargo build --locked --release --manifest-path benchmarks/rust/Cargo.toml
mkdir -p benchmarks/.work
benchmarks/rust/target/release/nwords-comparison vectors 0 100 benchmarks/ids-u32.txt > benchmarks/.work/native-phrases.tsv
python3 /path/to/04-build-profiles.runner.py --root "$PWD" --wasm-opt /absolute/path/to/wasm-opt
```

Use ordinary `python3`, without `-O`: the historical runner uses assertions for
its provenance guards. CPU 19 must be available. The general `run.py` remains the
supported full comparison runner.

The native reference and all five original qualification records are archived
alongside the raw results. Qualification objects were recovered from each saved
stdout log; each log's hash matches the original raw record. The reference hash
was computed after the run:
`10b1bb29ecb086f78f6a77e809fc58bb7d75568c5456c9e1cbafaebf5ce31ee0`.
The native positional/list implementation and reference generator did not change
between this experiment and that archival check. This is a supplemental audit,
not a claim that the original runner captured those fields during measurement.

The summary reuses `report.summarize`, which requires exactly three batches in
each of seven process rounds; all 20 variant/runtime/operation cells pass. It
archives IQR and min–max as well as medians, addressing the runner's insufficient
stdout-only summary guard. The targeted experiment did not capture per-round
host load or CPU busy fractions. Interleaving and the reported spread reduce
ordering concerns, but do not establish that small differences are significant.
The full stage runs separately retain their usual host-load records.

The later API work began after this run's completion marker and successful
end-of-run source/harness checks. No measured source changed during the run.
