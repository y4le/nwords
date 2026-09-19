# nwords

`nwords` is a small Rust workspace for bidirectional word and phrase codecs.
V1 focuses on BIP-39 compatibility, positional N-word IDs, and exact planning
helpers for dictionary size, word count, capacity, and accepted ID range.

The implementation is dependency-light and forbids unsafe code in every crate.

## Examples

### CLI ID Phrases

The workspace includes a small `nwords` binary for positional ID phrases:

```sh
cargo run -p nwords-cli -- encode 42 --preset u32
cargo run -p nwords-cli -- decode "<phrase>" --preset u32
cargo run -p nwords-cli -- encode 42 --preset aa
cargo run -p nwords-cli -- encode 42 --preset descriptor-object
cargo run -p nwords-cli -- encode 42 --preset mood-descriptor-object
cargo run -p nwords-cli -- encode 42 --preset weather-descriptor-plant
cargo run -p nwords-cli -- encode 4384286 --shape descriptor,object
cargo run -p nwords-cli -- encode 1337 --range 1e6 --shape color,adjective,animal
cargo run -p nwords-cli -- encode 42 --preset dec6-spread
cargo run -p nwords-cli -- encode 5 --range 12 --shape project,animal --list project=words.txt
cargo run -p nwords-cli -- text encode "hello"
cargo run -p nwords-cli -- bytes encode --hex deadbeef
cargo run -p nwords-cli -- presets
cargo run -p nwords-cli -- plan --preset u32
cargo run -p nwords-cli -- plan --range 1e6
cargo run -p nwords-cli -- plan --shape descriptor,object
cargo run -p nwords-cli -- plan --shape material,shape,object
cargo run -p nwords-cli -- plan --shape mood,descriptor,food
cargo run -p nwords-cli -- plan --shape project,animal --list project=words.txt
cargo run -p nwords-cli -- plan --shape color,adjective,animal
cargo run -p nwords-cli -- help encode
```

`*-spread` presets apply a deterministic reversible permutation before
positional encoding, so nearby assigned IDs usually produce less visually
similar phrases. They are not encryption and do not add entropy.

For custom `--shape`, `--words`, or legacy `--dict` encoding, omit `--range`
to use the exact full phrase capacity as the accepted ID range. Provide
`--range` to narrow the accepted domain and reject slack phrase states. Shapes
whose full capacity exceeds `u128` require an explicit `--range`.

`text` and `bytes` commands use `word-bytes-v1`: a 32-bit big-endian byte
length, payload bytes, and zero padding to an 11-bit word boundary. Text is
encoded as byte-exact UTF-8 with no default Unicode normalization.

The CLI includes BIP-39 English positional presets and named word-list shapes
such as `adjective,animal`, `descriptor,object`, and
`weather,descriptor,plant` and `mood,descriptor,food`. BIP-39 positional
presets do not produce BIP-39 wallet mnemonics:

```text
BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.
```

Named-list presets use ordered word maps derived from MIT-licensed source
lists: `unique-names-generator` for adjective, animal, and color, and
`glitchdotcom/friendly-words` for descriptor and object. Mood, material, shape,
weather, plant, and food are authored in-repo from permissive seed references.
They are deterministic ID encodings, not random names:

```sh
cargo run -p nwords-cli -- encode 0 --preset aa
# able aardvark
cargo run -p nwords-cli -- encode 249416 --preset aa
# zippy zebra
cargo run -p nwords-cli -- encode 0 --preset color-aa
# amaranth able aardvark
cargo run -p nwords-cli -- encode 0 --preset descriptor-object
# abalone aardvark
cargo run -p nwords-cli -- encode 0 --preset color-descriptor-object
# amaranth abalone aardvark
cargo run -p nwords-cli -- encode 0 --preset mood-descriptor-object
# alert abalone aardvark
cargo run -p nwords-cli -- encode 0 --preset material-shape-object
# acrylic angular aardvark
cargo run -p nwords-cli -- encode 0 --preset weather-descriptor-plant
# balmy abalone abelia
cargo run -p nwords-cli -- encode 0 --preset mood-descriptor-food
# alert abalone almond
```

User-defined lists can be mixed with built-ins by passing repeatable
`--list name=path` entries and referencing `name` inside `--shape`. Files use
one lowercase ASCII word per line; blank lines and full-line `#` comments are
ignored, with only outer ASCII whitespace trimmed. Pin and share the same list
files for reproducible decoding. The CLI reports an `fnv1a64:<hex>` drift
fingerprint for user lists; it is not a security hash.

### BIP-39 English

```rust
use nwords::bip39::English;

let codec = English::default();
let entropy = [0u8; 16];
let phrase = codec.encode_entropy(&entropy).expect("valid entropy");

assert_eq!(
    phrase,
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
);
assert_eq!(codec.decode_phrase(&phrase).expect("valid phrase"), entropy);
```

### BIP-39 Seed Derivation

Enable the `bip39-seed` feature:

```rust
use nwords::bip39::English;

let codec = English::default();
let phrase = codec.encode_entropy(&[0u8; 16]).expect("valid entropy");
let seed = nwords::bip39::seed::derive_seed(&phrase, "TREZOR");

assert_eq!(seed.len(), nwords::bip39::seed::SEED_BYTES);
```

### Positional IDs

```rust
use nwords::{core::Linear, positional::Positional};

const WORDS: &[&str] = &[
    "zero", "one", "two", "three", "four",
    "five", "six", "seven", "eight", "nine",
];

let codec = Positional::new(Linear::new(WORDS), 3, 1_000).expect("valid shape");

assert_eq!(codec.encode(42).expect("in range"), "zero four two");
assert_eq!(
    codec.decode_words(&["zero", "four", "two"]).expect("valid phrase"),
    42
);
```

### Adjective-Animal IDs

```rust
use nwords::{
    positional::MixedPositional,
    wordlists::adjective_animal::{AdjectiveAnimal, CAPACITY, WORD_COUNT},
};

let codec = MixedPositional::new(AdjectiveAnimal, WORD_COUNT, CAPACITY)
    .expect("valid adjective-animal shape");

assert_eq!(codec.encode(0).expect("in range"), "able aardvark");
assert_eq!(
    codec.decode_words(&["zippy", "zebra"]).expect("valid phrase"),
    CAPACITY - 1
);
```

### Named Word-List Shapes

```rust
use nwords::{
    positional::MixedPositional,
    wordlists::named::{NamedWordList, WordListSequence},
};

let lists = [
    NamedWordList::Color,
    NamedWordList::Adjective,
    NamedWordList::Animal,
];
let shape = WordListSequence::new(&lists);
let codec = MixedPositional::new(shape, shape.word_count(), 12_969_684)
    .expect("valid named shape");

assert_eq!(codec.encode(0).expect("in range"), "amaranth able aardvark");
assert_eq!(
    codec
        .decode_words(&["yellow", "zippy", "zebra"])
        .expect("valid phrase"),
    12_969_683
);
```

### Stats Planning

```rust
use nwords::stats::{self, PlanSolution, PlanTarget};

let solution = stats::required_words(PlanTarget::Range(1_000_000), 10)
    .expect("valid target");
assert_eq!(
    solution,
    PlanSolution::RequiredWords {
        word_count: 6,
        capacity: stats::CapacityClass::Exact(1_000_000),
    }
);

let bip39 = stats::bip39::minimum_word_count_for_entropy_bits(192)
    .expect("valid BIP-39 target");
assert_eq!(bip39.word_count, 18);
```

## Feature Matrix

| Feature | Default | Adds |
|---|---:|---|
| `std` | yes | Standard-library support; enables `alloc`. |
| `alloc` | yes | `String`, `Vec`, and phrase-facing APIs in `no_std` builds. |
| `stats` | yes | Exact capacity and planning helpers under `nwords::stats`. |
| `adjective-animal` | yes | Compatibility adjective-animal word map. |
| `named` | yes | Named word lists and ordered phrase shapes. |
| `bip39` | yes | BIP-39 English phrase codec. |
| `bip39-japanese` | no | Japanese wordlist, U+3000 display, and Unicode parsing. |
| `bip39-seed` | no | PBKDF2-HMAC-SHA512 seed derivation. |
| `positional` | yes | Positional N-word ID codec. |
| `word-bytes` | yes | `word-bytes-v1` arbitrary byte and UTF-8 text codec. |

## Static Web Demo

For reusable Node/browser output and the Murmur naming use case, see the
[JavaScript / WASM package plan](docs/wasm-package-plan.md).

The repository includes a small browser demo under `site/` backed by the Rust
implementation compiled to WebAssembly from `crates/nwords-web`.

Build the static demo:

```sh
wasm-pack build crates/nwords-web --target web --out-dir ../../site/pkg --no-pack
```

Then serve `site/` with any static file server:

```sh
python3 -m http.server 8787 --bind 127.0.0.1 --directory site
```

The demo exposes numeric ID encode/decode, preset planning reports, spread
comparison, and `word-bytes-v1` text encode/decode. GitHub Pages deployment is
handled by `.github/workflows/pages.yml`, which builds the WASM package and
uploads `site/` as the Pages artifact.

`wasm-bindgen` is the only web-demo runtime bridge dependency; it is used to
call the Rust library from the static browser app without reimplementing the
codec in JavaScript.

## `no_std + alloc`

Default builds use `std`. For `no_std + alloc`, disable default features and
select the capabilities you need:

```toml
nwords = { version = "0.1", default-features = false, features = ["alloc", "bip39", "stats"] }
```

V1 does not support phrase-facing APIs without `alloc`. BIP-39 seed derivation
also requires `alloc` for Unicode normalization.

## Compatibility Scope

V1 ships:

- A `nwords` CLI for positional ID phrase encoding and decoding.
- `word-bytes-v1` CLI/library support for arbitrary bytes and UTF-8 text.
- BIP-39 English and Japanese entropy, mnemonic, checksum, and seed-vector
  compatibility.
- Uniform and mixed-radix positional N-word codecs over user-provided and
  built-in dictionaries.
- Curated English named word lists, ordered phrase shapes, and CLI presets.
- Exact `u128` capacity/range math plus log-domain estimates beyond `u128`.
- `Linear` and `Sorted` word maps.
- Identity and affine spread permutations.

Built-in word-list names, contents, and order are compatibility surfaces.
Changing a shipped list requires a new list name or version rather than an
in-place edit.

V1 intentionally defers SLIP-39, Niceware, Proquint, PGP word lists,
non-identity permutations, BIP-32/xprv derivation, and BigInt-backed exact
capacity math.
