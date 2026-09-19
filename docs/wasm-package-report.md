# WASM package qualification

Completed all four steps of [the package plan](wasm-package-plan.md) on 2026-09-19.
The private `@y4le/nwords` package is distributed as a local tarball. Registry
publication remains deferred. Murmur installs the same pinned artifact, assigns
collision-checked adjective–animal aliases, and persists them alongside canonical IDs.
The consumer integration is Murmur commit `c3dd6bc`.

## Artifact and reproducibility

| Property | Recorded value |
|---|---|
| Package | `@y4le/nwords@0.1.0`, private, no lifecycle scripts |
| Clean source commit | `83db656230feeb4b6ece2c8984ee69869a701303` |
| Tarball bytes | 95752 |
| Unpacked bytes | 725461 |
| WASM bytes | 129312 |
| Tarball SHA-256 | `b9c0f62c12917017517a6ac4b1c2bee7c30c0e150b5a273e5f674d56a48e655a` |
| WASM SHA-256 | `0d776e62dd0a809cacf3bd3d1231c423487f7fb1bb6f1d0f4f4481beb2cce880` |
| Cargo.lock SHA-256 | `ca3425a79ecc37e8fb025059632fa1e4f59b38aaa9c7d12d1187cc9d7c8bc1ef` |
| Rust / wasm-pack / wasm-bindgen | 1.94.0 / 0.14.0 / 0.2.120 |
| Node / npm / TypeScript / Playwright | 24.14.1 / 11.11.0 / 7.0.2 / 1.63.0 |
| Actual test host | Linux arm64, Chromium 153.0.8010.12 |

`build.json` travels inside the tarball. Assembly requires a clean committed tree
unless explicitly marked `--dev`; development artifacts were not vendored in Murmur.
Pinned rustc release optimization is used, with wasm-opt disabled. Two consecutive
clean local builds, plus a fresh checkout with a different Cargo home, produced
identical tarball and WASM hashes. Source path prefixes are remapped. Tests also
confirmed that stale-commit and dirty-tree qualification attempts fail. The report itself
is a later documentation commit, so its HEAD need not equal the artifact's source.

The artifact is retained at
[`y4le-nwords-0.1.0-83db656230fe.tgz`](../../murmur/vendor/y4le-nwords-0.1.0-83db656230fe.tgz); Murmur's npm lockfile records its integrity.
The full project license texts, all compiled named-list provenance and conservative
binding/runtime notices are included. Test dependencies are not shipped.

## Validation

- All Rust workspace tests, clippy with warnings denied, formatting, no-default
  alloc checks/tests, rustdoc and fixture checksums passed.
- The exact tarball was installed with `--ignore-scripts` in a temporary consumer
  outside the source tree. ESM and CJS dynamic import work with Rust, cargo and
  wasm-pack absent from the runtime PATH and with a different working directory.
- Shared committed-CLI vectors cover list ordering, first/last IDs, narrow ranges,
  `2^53 + 1`, and `u128::MAX - 1`. JS and direct ABI tests reject malformed numbers,
  overflow, invalid shapes/phrases and slack without exposing input text.
- Installed declarations pass strict TypeScript 7 with NodeNext and Bundler
  resolution and `skipLibCheck: false`, including negative type assertions.
- Real Chromium loads the installed package. Node/browser tests cover concurrency,
  missing/corrupt/invalid-export WASM, retries, source conflicts, byte snapshots,
  ordinary codec errors, and no asset fetch at import time.
- The separate static demo still passes ID and text roundtrips.
- Murmur's typecheck and all 34 tests passed. Deterministic collision tests exercise
  the registry lock, other scopes, closed names, exhaustion and explicit-name bypass.
  A fresh process reads the persisted alias and canonical ID without loading WASM.

One initial Murmur full-suite run hit an existing tmux shutdown timing failure in an
explicitly named session. That test passed in isolation and the full suite passed
on rerun; no unrelated lifecycle behavior was changed.

## Measurements

Five fresh processes per measurement, warm filesystem cache; timings are observations
on this host, not performance guarantees. Node process startup is excluded.

| Measurement | Median | Observed range |
|---|---:|---:|
| Installed package initialization | 12.02 ms | 11.92–12.45 ms |
| Murmur lazy loader and first candidate | 12.05 ms | 11.73–12.53 ms |

Murmur's `scripts/measure-names.ts` uses the actual installed dependency and real
registry in a disposable home, then rereads the saved session. Package qualification
writes raw samples to `dist/package-qualification.json`; its build and test scripts
reproduce artifact checks and measurements. The measured size/latency does not
justify dictionary splitting for this first consumer.

## Review and support limits

Fable challenged the architecture and reviewed the package diff; Opus reviewed the
Murmur diff through Parley before commits. Review requests were
`req_review_diff_f41e13af3274a86f` (package) and
`req_review_diff_ef701f18f21ba9fc` (Murmur). [Architecture decisions](research/architecture-decisions.md)
record the choices to require an explicit range beyond u128 and report naming failure
rather than silently select a legacy naming scheme.

Node 24.14.1 and ordinary Chromium ESM were exercised. Bundlers, workers, other
browsers and additional Node releases are not claimed. CI repeats the package job
on Linux x64; its remote result has not yet been observed. No native provider session
was launched to qualify generated aliases: real allocation, persistence, resolution
and process restart were exercised, plus the existing simulated-provider suite.
