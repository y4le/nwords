# Add EFF long and flexible wordsets for Rust, Node, and browsers

Applications can now choose EFF long, mix any built-in and custom dictionaries,
encode growing IDs, and round-trip arbitrary bytes or UTF-8 text through those
wordsets. Existing fixed-shape IDs and the legacy `word-bytes-v1` format retain
their mappings.

```js
const words = await loadNwords();
const pets = { name: 'pets', words: ['猫', 'dog', '🦊'] };
const names = words.prepare({
  scheme: 'variable-v1',
  pattern: [{ list: 'adjective', repeat: { min: 0 } }, pets],
  range: 1_000_000n,
});
names.encodeId(0n); // 猫
names.encodeId(3n); // able 猫
names.dispose();

const data = words.prepareBytes({scheme: 'radix-bytes-v1', lists: ['eff-long']});
const phrase = data.encodeBytes(new Uint8Array([0, 255, 0]));
data.decodeBytes(phrase); // Uint8Array [0, 255, 0]
data.decodeText(data.encodeText('hello 世界')); // hello 世界
data.generatePassphrase(16); // sixteen random bytes, eleven EFF words
data.dispose();
```

- EFF long preserves all 7,776 upstream entries and original dice-roll order,
  including hyphenated words. The Rust crate and JS package ship attribution
  and the CC BY 4.0 license. Rust consumers can disable its feature.
- `variable-v1` orders shorter phrase tiers before longer ones. Increasing range
  or maximum words preserves existing mappings. It supports one leading repeat
  (minimum zero or one) followed by any nonempty fixed suffix.
- `radix-bytes-v1` cycles through arbitrary dictionaries, using eight-byte blocks
  and a mandatory variable tail. Empty/odd/leading-zero payloads round-trip.
  It is analogous functionality to Niceware, not Niceware wire compatibility.
- The JS catalog exposes all thirteen named lists. Custom Unicode tokens preserve
  spelling and order; prepared codecs own snapshots and indexed lookups.
  Fixed, variable, and byte descriptors are distinct, serializable plain data.
- Random generation uses platform cryptographic randomness. Integer sampling
  rejects slack without modulo bias; byte generation preserves the drawn bytes.

The [package guide](../packages/nwords-js/README.md) documents APIs, limits and
examples. The [contract](research/flexible-wordsets-plan.md) records mapping,
canonicality and consultation decisions. JS supports 32-word integer formats,
4 KiB byte payloads, and bounded custom vocabularies; this does not extend the
existing exclusive-u128 ID range. Variable CLI grammar remains a future adapter.

Validation includes frozen source/index parity, shared Rust/JS variable vectors,
an independent EFF byte oracle, exhaustive enumeration of 3,243,603 toy byte
phrases, numerical/resource boundaries, and Unicode/ownership/disposal checks.
Packed consumer qualification covers Node ESM, CJS dynamic import, Chromium,
NodeNext/Bundler declarations, loader recovery, and the existing demo. Rust
fmt/clippy/workspace tests, minimal-feature builds and frozen checksums are gates.

[Measurements](../benchmarks/results/wordsets/README.md) record package size and
focused Rust/Node operation timings, including language peers for sixteen-byte
payloads. Existing fixed-ID operations are 6.6–11.3% slower in the captured run;
independent review reproduced the slowdown, and its cause is not yet isolated.
Formats and word counts differ; these short runs are not a replacement
for the original multi-process performance suite. Prepared custom codecs avoid
revalidating entire vocabularies on every call.
