# Reusable JavaScript / WASM package

Plan recorded 2026-09-19; all four implementation steps are now complete. See the
[qualification report](wasm-package-report.md) for the clean artifact, tests, measurements
and Murmur integration. The package remains private and has not been published to a registry.

The sections below preserve the original plan and its pre-implementation evidence limits.
The immediate consumer is Murmur, a Node/TypeScript application that wants readable collaborator
names without requiring users to install Rust or a separate executable.

## Outcome and boundary

Distribute compiled WebAssembly, its generated JavaScript glue, TypeScript declarations and a
small public JS interface as one package. Rust continues to own the codecs and word lists.
Package maintainers need the Rust/WASM toolchain; ordinary consumers need only their supported
JavaScript runtime. No install-time compilation, platform-specific native binaries, executable
discovery or remote WASM download is required for Node consumption.

For the first consumer, nwords maps a bounded integer to an ordered phrase shape and back.
Murmur chooses random candidates, checks its registry for collisions, persists the assigned alias
and retains its own canonical session ID. nwords does not allocate names, keep registry state or
promise uniqueness across applications. A short phrase cannot losslessly encode Murmur's much
larger session-ID space. Use a separate bounded naming number instead of truncating an ID and
claiming reversible identity.

## What already exists

Inspection covered the working tree based on `ebfe6801f218b2389de1453594d676e1ee62c7e5`.
There is concurrent uncommitted CLI/word-list work; this plan does not incorporate or freeze it.

| Existing piece | Reuse and missing work |
|---|---|
| [nwords-web](../crates/nwords-web/src/lib.rs) | wasm-bindgen exports for the static demo, returning JSON strings; leave this working consumer intact and reuse its binding experience |
| [NamedWordList / WordListSequence](../crates/nwords-wordlists/src/named.rs) | Existing list lookup, ordered shapes and checked capacity; use directly |
| [MixedPositional](../crates/nwords-schemes/src/positional.rs) | Existing mixed-radix encode/decode and range rejection; keep this implementation authoritative |
| [Browser demo](../site/app.js) | Already loads generated web-target glue; preserve its behavior while adding reusable outputs |
| [Pages build](../.github/workflows/pages.yml) | Already builds with wasm-pack; lacks packed-package consumer tests and reusable artifact assembly |
| [CLI](../crates/nwords-cli/src/lib.rs) | Richer preset/shape features than the demo; full parity is not needed for the first package |

The current bindings use wasm-bindgen 0.2.120 in the lockfile. The Pages workflow selects
wasm-pack 0.14.0. A demo build is evidence of existing infrastructure, not qualification of an
npm package, Node loader or named-shape WASM API.

## Small public API

Start with ordered built-in list names rather than copying the CLI's growing preset table into
JavaScript or introducing a second Rust catalog. The motivating shapes are
`["adjective", "animal"]` and `["adjective", "color", "animal"]`; order is part of the codec.
The CLI's `color-aa` preset uses a different order, so these names are not interchangeable.

Proposed interface, with package name and exact spelling provisional:

```ts
import { loadNwords } from "@y4le/nwords/node";

const words = await loadNwords();
const lists = words.lists(); // supported canonical names and sizes
const shape = { lists: ["adjective", "animal"] as const };
const info = words.describeShape(shape); // exact capacity/range as bigint, or explicit capacity overflow
const phrase = words.encodeId(42n, shape); // canonical space-separated phrase
const id = words.decodePhrase(phrase, shape); // 42n
```

The namespace above is illustrative, not a claim that it is available or published.
The small surface consists of built-in list metadata, shape description, ID encoding and phrase
decoding. A shape has ordered `lists` and an optional exclusive upper bound `range`. Include
canonical list names and sizes in metadata so applications do not hard-code the naming capacity.
Initially expose the canonical `adjective`, `animal` and `color` lists only, in metadata and calls.
Ordered shapes may repeat a list, as the existing WordListSequence already permits. Do not create
a second alias vocabulary or expose all optional Rust features implicitly. The first package
enables named lists, positional codecs and the stats support required by MixedPositional,
without the BIP-39/text features embedded in the demo. Return the resolved range as well as shape;
both affect decoding. Identity permutation is sufficient for the naming use case.

- Accept `bigint` or canonical decimal strings matching `0|[1-9][0-9]*` for IDs and ranges; reject JS `number`, including
  apparently small integers, rather than silently supporting two numeric safety regimes.
- At the Rust boundary, use checked decimal parsing. Reject negative, malformed and overflowing
  values before any narrowing conversion. JS convenience conversion is not the only validation.
- A range is a positive `u128` count and IDs satisfy `0 <= id < range`. An omitted range uses
  exact shape capacity when representable; a larger capacity needs an explicit supported range.
  In particular, a range of `2^128` cannot be represented by the existing `u128` range parameter.
  Shape description distinguishes exact capacity from beyond-u128 capacity; it never substitutes
  the chosen range for the true capacity or claims an exact value it did not compute.
- Return exact numeric values as `bigint` in the public JS API. JSON consumers explicitly convert
  them to decimal strings; the internal JSON boundary keeps its existing lossless string form.
- Parse with Rust's `split_whitespace`, followed by exact case-sensitive lookup, without implicit
  case folding. Output uses ASCII spaces. Murmur may display the result with hyphens; it must
  reverse that rendering itself if decoding, rather than silently expanding the codec's grammar.
- Return stable, typed errors for invalid input, unknown lists, invalid shapes, out-of-range IDs
  and invalid phrases. Error messages must not echo raw input phrases. Loader failures remain
  distinguishable from codec errors. Preserve useful error positions without leaking input text.
- Operations are synchronous after successful initialization. Avoid exported Rust object handles
  requiring callers to manage `.free()` merely to encode a name.

This numeric validation matters even with BigInt: wasm-bindgen supports `u128`, but documents
wrapping conversions for out-of-range BigInt inputs. A direct primitive binding is not a checked
public contract. [wasm-bindgen numeric conversions](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/numbers.html)

Add a small separate binding crate, provisionally `crates/nwords-js`, with named-shape exports and
the explicit feature set above. This shares the Rust codecs and lists without inheriting the demo's
larger payload or its JSON API compatibility obligations. It is a bindings crate, not another codec
implementation. The existing demo crate remains unchanged; migrating it is optional later work.

Keep encoding, parsing and capacity math in Rust; JS handles loading, representation conversion and
public error/type ergonomics. An internal JSON envelope can reuse the existing approach without a
new serialization dependency, but include machine-readable error codes mapped from Rust variants.
The ABI carries IDs and ranges as decimal strings, not primitive `u128` parameters.
Do not infer codes from English error text or expose JSON strings as the normal JS API. Validate
canonical numeric syntax at both the public JS and Rust boundaries, including signs, whitespace
and leading zeros. Malformed input must never become another valid identifier.

## Package and loading design

Prefer one `--target web` WASM/glue pair shared by two explicit loaders, then qualify it in both
environments. The web target is browser-loadable ESM, while the conventional nodejs target emits
CommonJS. This is a proposed simplification to test, not a portability claim established here.
[wasm-bindgen deployment targets](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html)

| Entry | Loading contract |
|---|---|
| `/node` | ESM loader reads packaged WASM relative to its own module, supplies bytes to generated initialization, then returns the API; no dependence on cwd or fetch support for file URLs |
| `/web` | ESM loader initializes from the served package asset or caller-provided URL/bytes; documented explicit async initialization, with no Node imports in its graph |

Treat `loadNwords()` as initialization of a reusable module, not a promise of a new isolated WASM
instance each time. Concurrent calls share initialization and a rejected initialization promise
is cleared for retry. The Node and browser entry points share the same generated module/instance
when imported into the same process; neither promises independent initialization state. After a
successful load, reject an attempt to switch the asset rather than silently ignoring a new source.
Do not add top-level await or hide browser network loading at module import time.

Support Node ESM first, including Murmur's current Node 24. CommonJS consumers can use dynamic
`import()`; synchronous `require()` is not a first-release promise. Do not build a second payload
or loader flavor just for hypothetical consumers. Browser validation starts with ordinary ESM,
not a matrix of every bundler or edge runtime. If the shared web-target approach fails a concrete
consumer test, compare a generated nodejs target before inventing a custom WASM loader.

Publish explicit package exports and TypeScript declarations for supported entries. Keep generated
glue private, and allow an asset subpath only where browser consumers need to locate the WASM file.
Include `.wasm`, declarations, licenses and examples in the packed artifact. Consumers must not
depend on repository-relative paths, a sibling checkout, `target/`, or `site/pkg/`.

Generated bindings provide TypeScript declarations for their Rust exports, but those declarations
alone cannot describe the typed results of JSON-returning functions. Test the wrapper's declarations
against real calls. [wasm-bindgen TypeScript generation](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/skip_typescript.html)

## Compatibility, build and distribution

Preserve existing word ordering and codec behavior. Adding a wrapper does not permit changing a
shipped list. Pin the package version for consumers that need reproducible encoding; record shape
and version when persisting reversible encodings. Murmur's persisted display phrase remains stable
without re-encoding it after an update. Any ongoing list edits need their own compatibility decision
before inclusion in a release; they are not approved by this plan. Record source commit and build
tool versions in package metadata. Qualification/release artifacts must come from a clean committed
revision. An explicit development build may use a dirty tree, marked as such in its metadata; it
must not be confused with the reproducible release artifact or used to establish published parity.

Keep build glue ordinary: a checked-in script and package template can assemble an ignored output
directory from the new binding crate, generated bindings and thin JS sources. One package payload
does not mean one WASM build for the entire repository: the existing demo has its own build.
Ignore the assembled package directory and keep it separate from the Pages upload directory.
Renaming that demo or creating a generic bindings framework does not help the first consumer.
Pin builder versions and use the lockfile; ensure the bindgen tool matches the resolved crate.
No new dependency belongs in the Rust core for packaging. If a binding dependency becomes necessary,
justify its cost under [code standards](code_standards.md).

Include the license texts corresponding to the workspace's declared license and the notices for
every included third-party word list and redistributed component. Existing word-list provenance
is available in the [adjective/animal fixtures](../tests/vectors/adjective-animal/README.md) and
[friendly-word fixtures](../tests/vectors/friendly-words/README.md), with license texts in
[the fixture licenses](../tests/vectors/licenses/). The `named` feature compiles additional lists;
also account for [authored-list provenance](../tests/vectors/semantic-wordlists/), including each
list's README and seed references. Derive notices from all included content, not just the three
names the public API accepts. Inspect the actual packaged payload: a WASM binary can embed lists
that are not visible as separate data files. Do not assume optimization removed them.

Measure packed bytes, WASM bytes and cold initialization time on the actual first consumer.
Record results rather than assuming WASM is faster or smaller. Avoid splitting dictionaries into
many downloadable modules unless these measurements establish a worthwhile cost.

First distribute a local `npm pack` tarball, with CI artifacts as useful. Package publication,
registry naming and release credentials are separate follow-up work; no registry publication or
Murmur dependency update is implied by this planning document. Keep the development package private
until it is deliberately prepared for publication, using `"private": true` in the package template
and asserting that value during the development packing check.

## Implementation sequence and acceptance

| Step | Focused change | Evidence needed |
|---|---|---|
| 1. Named-shape bindings | Add a small nwords-js crate with bounded shape describe/encode/decode operations, explicit features and coded errors using existing Rust APIs | Shared boundary vectors match Rust and CLI for selected two/three-word shapes; the existing demo remains intact |
| 2. Reusable package | Add typed JS facade, Node/browser initialization and a repeatable packed output | Node imports the installed tarball from a different directory, encodes/decodes and finds its WASM without Rust, wasm-pack, a source checkout or install scripts |
| 3. Qualify and document | Add package tests to CI, consumer examples and a small distribution report; use Playwright with Chromium as dev-only browser test tooling | Browser ESM loads the same artifact, TypeScript accepts documented calls and rejects invalid types, packed contents include required assets/notices |
| 4. Exercise Murmur | Separately integrate the pinned artifact into the naming feature | Murmur allocates and persists collision-checked aliases while retaining canonical IDs; restarting does not require regenerating names |

Steps 1–3 make nwords independently useful. Step 4 is a consumer follow-up, not a dependency of
the package's own release. Scope the initial supported Node floor and browser behavior to what
the consumer tests actually qualify; do not infer compatibility across runtimes from WASM alone.

Tests should cover actual failure boundaries rather than duplicating implementation details:

- Known first/last entries, representative round trips, differing list orders, narrowed ranges,
  capacity overflow, invalid list names, wrong word count and unknown words.
- Use seven repetitions of `animal` for the precision canary `2^53 + 1`; its capacity is
  `333^7 = 454056225438947877`. Use sixteen repetitions for beyond-u128 capacity and an explicit
  range of `u128::MAX`: accept ID `u128::MAX - 1`, reject ID `u128::MAX`, and reject range `2^128`.
  Pin reference vectors against the committed Rust/CLI revision, not a changing local executable.
- Reject `"+1"`, `"01"`, `" 1"`, `"1 "`, `""`, `"1e3"`, `"-0"`, overflowing decimal strings,
  `-1n`, `2n ** 128n`, fractional inputs and accidental JS numbers. Test string validation directly
  at the ABI as well as through the wrapper; no truncation or wrapping.
- Multiple initialization calls, failed initialization and retry, missing/corrupt WASM and ordinary
  codec errors after initialization. A failure must not poison later calls without explanation.
- Install the exact packed tarball into a temporary consumer using `--ignore-scripts`; exercise
  Node ESM and CommonJS dynamic import with no Rust tools available to that consumer. The packed
  package must contain no install hook that would ordinarily compile or download a replacement.
- Serve assets from the packed package to a browser test. Verify initialization, representative
  encode/decode, custom asset location and errors; a Node test of the browser module is insufficient.
- Validate TypeScript module resolution and declared results against installed package entries.
  Verify existing static-demo calls still work and preserve established codec vectors.

Apply the existing Rust checks from [code standards](code_standards.md) to implementation changes.
Add a small package CI job rather than multiplying every Rust job across all operating systems.
Broaden the runtime/OS matrix only when adding an explicit support claim or investigating a failure.

## Deliberately deferred

Full CLI parity, user-supplied dictionary files, additional cryptographic bindings, a shared preset
catalog refactor, synchronous CommonJS loading, workers, bundler-specific integrations, WASI,
platform-native Node addons and automatic publishing are outside this first slice. If named presets
are later exposed across CLI and WASM, extract their definitions into a shared Rust module rather
than copying them. This plan does not need to reopen nwords' broader codec architecture.

## Consultation and evidence limits

This is source-based planning, not a built or qualified package. Official binding documentation
supports the loading/type observations above; the single-payload package architecture is a proposal.
An Opus consultation through Murmur recommended a separate minimal binding crate, explicit feature
selection, stateless Rust exports, canonical numeric input and error codes. Those changes are
incorporated. We retain explicit Node/browser subpaths rather than conditional environment guessing,
and require an actual browser runtime check before claiming browser support; bundling alone does
not prove initialization. We defer mandatory per-list fingerprints and custom separators: existing
versioned-list rules and canonical output are sufficient for the first consumer. Persisted reversible
encodings retain package version, ordered lists and resolved range; optional drift metadata can
follow a real need. The literal stored display alias needs none of that machinery to stay stable.

We also defer a JS codec-object factory, sync loaders and inline base64 payloads. The stateless
surface and external asset already satisfy the first use case, with less API and loader behavior
to maintain. No claim about broad bundler compatibility from the consultation has been adopted.
Murmur session `490efa68df715721e90f` recorded planning request `wasm_plan_consult`.
Opus reviewed this candidate in `wasm_plan_doc_review`. Its substantive findings were complete
embedded-list notices, committed source provenance, a declared initial list set, concrete numeric
boundary vectors and a named browser harness; those are incorporated. Initialization retry and
the decimal-string ABI are explicit as well. This was document review only: no implementation,
WASM build or runtime qualification was performed.

## Performance follow-up: lean success ABI

The performance branch preserves the checked decimal input boundary, while adding
raw phrase and u128/BigInt success returns. Structured errors, metadata and legacy
`*_json` diagnostic exports keep their envelopes. The public API and its validation
contract are unchanged; no unchecked primitive u128 input is introduced.
