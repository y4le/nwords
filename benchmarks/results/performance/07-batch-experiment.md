# Bounded batch prototype: rejected

The development prototype extends the prepared codec with ordered, bounded arrays.
Both paths materialize output arrays and consume every result. Each process checks
all 4,096 outputs before timing. One operation is an entire array; this table divides
by its size to report **nanoseconds per ID**. Brackets show process-median IQRs.

Three fresh processes per cell and runtime, three calibrated batches per process,
100 ms warmup and target duration, CPU 19. This is an exploratory trial of a
qualified development artifact, not a released API or a seven-round production
stage. The raw data includes qualification, source/diff/harness/reference hashes,
per-round load and all samples. No samples were discarded.

| Runtime | Batch size | Encode batch / loop ns per ID [IQR] | Decode batch / loop ns per ID [IQR] |
|---|---:|---:|---:|
| node | 1 | 479.4 [477.9–484.9] / 280.5 [279.0–282.2] | 602.0 [598.9–602.4] / 425.4 [423.9–429.5] |
| node | 16 | 371.5 [371.5–372.4] / 271.5 [271.0–272.6] | 512.3 [510.5–512.9] / 417.8 [417.0–418.3] |
| node | 256 | 362.5 [361.7–362.5] / 270.7 [270.4–272.5] | 511.2 [508.0–512.4] / 417.8 [415.1–418.3] |
| chromium | 1 | 607.7 [602.7–609.0] / 408.9 [408.4–409.9] | 607.3 [605.0–609.4] / 432.2 [430.1–432.8] |
| chromium | 16 | 491.7 [483.1–491.9] / 390.2 [389.3–393.1] | 514.2 [513.8–515.6] / 424.2 [423.6–425.1] |
| chromium | 256 | 483.3 [482.7–487.3] / 389.4 [387.2–391.4] | 512.3 [512.0–514.0] / 425.2 [424.9–426.7] |

At 256 IDs, batch encode is 34% slower than the prepared loop in Node and 24%
slower in Chromium. Batch decode is 22% and 20% slower respectively. These gaps
justify rejecting this array/externref transport; they do not prove that every
possible batching protocol would be slower. No batch methods enter the public API.

Opus reviewed the prototype in `req_review_diff_56d053ec890dfc67`. Findings addressed
before qualification include error metadata documentation, distinct item/word
position tests, a native envelope assertion, accurate raw bounds errors and a
second disposal guard after user-controlled array access. Seven native binding
tests and the packed Node/CJS/Chromium/types/demo qualification pass.

To reproduce, create an isolated worktree at `cdb5633`, apply the adjacent patch,
install the locked package and benchmark dependencies, then run:

```sh
cargo test -p nwords-js
node packages/nwords-js/scripts/build.mjs --dev
node packages/nwords-js/scripts/test-package.mjs --dev
python3 /absolute/path/to/07-batch-experiment.runner.py --root "$PWD"
```

The runner installs the actual qualified tarball and builds its native reference
before timing. Use ordinary Python without `-O` and make CPU 19 available. The
prototype patch is archived for review/reproduction and is not applied to this branch.

The archived patch includes the actual workload: each case captures its explicit
array size, checks that every batch returns exactly that many outputs before
timing, and slices the shared fixture accordingly. That establishes the per-ID
normalization used here. The summary sidecar records IQR and min–max for every
cell. The prototype changes only existing tracked files; no new source file is
omitted by its diff hash. The only untracked paths were development dependency
symlinks, which are not part of the prototype.
