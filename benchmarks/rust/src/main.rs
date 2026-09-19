//! Standalone measurement harness; dependencies do not enter the nwords workspace.
#![forbid(unsafe_code)]
use nwords::{
    core::{Formatter, IdentityPermutation},
    positional::{MixedPositional, Positional},
    wordlists::named::{NamedWordList, WordListSequence},
};
use rand::Rng;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

const ADJECTIVES: &str =
    include_str!("../../../tests/vectors/adjective-animal/nwords-adjectives.txt");
const ANIMALS: &str = include_str!("../../../tests/vectors/adjective-animal/nwords-animals.txt");
struct Hyphen;
impl Formatter for Hyphen {
    fn join(&self, words: &[&str]) -> String {
        words.join("-")
    }
}
fn split_four(phrase: &str) -> [&str; 4] {
    let mut words = phrase.split_whitespace();
    [
        words.next().expect("word"),
        words.next().expect("word"),
        words.next().expect("word"),
        words.next().expect("word"),
    ]
}
fn split_three(phrase: &str) -> [&str; 3] {
    let mut words = phrase.split_whitespace();
    [
        words.next().expect("word"),
        words.next().expect("word"),
        words.next().expect("word"),
    ]
}
const COUNT: usize = 4096;
const RANGE: u128 = 1u128 << 32;
struct Case<'a> {
    name: &'static str,
    run: Box<dyn FnMut(usize) -> u64 + 'a>,
}
fn consume(value: String) -> u64 {
    let value = black_box(value);
    value.len() as u64 + u64::from(value.as_bytes()[0])
}
fn batch(case: &mut Case<'_>, count: usize) -> (f64, u64) {
    let started = Instant::now();
    let mut sink = 0u64;
    for i in 0..count {
        sink = sink.wrapping_add((case.run)(black_box(i & (COUNT - 1))));
    }
    (started.elapsed().as_secs_f64() * 1e9, black_box(sink))
}
fn main() {
    let mode = std::env::args()
        .nth(1)
        .expect("case or list/vectors/cold-mnemonic");
    let round: usize = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "0".into())
        .parse()
        .expect("round");
    if mode == "cold-mnemonic" {
        let phrase = mnemonic::to_string(42u32.to_be_bytes());
        let started = Instant::now();
        let mut out = [0u8; 4];
        mnemonic::decode(&phrase, &mut out[..]).expect("decode");
        let ns = started.elapsed().as_secs_f64() * 1e9;
        assert_eq!(out, 42u32.to_be_bytes());
        println!(
            "{{\"kind\":\"cold-native\",\"case\":\"mnemonic/first-decode\",\"elapsedNs\":{ns}}}"
        );
        return;
    }
    let target_ms: f64 = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "100".into())
        .parse()
        .expect("milliseconds");
    assert!((10.0..=1000.0).contains(&target_ms));
    let pair_lists = [NamedWordList::Adjective, NamedWordList::Animal];
    let four_lists = [NamedWordList::Animal; 4];
    let pair = MixedPositional::new(WordListSequence::new(&pair_lists), 2, 249417).expect("pair");
    let four = MixedPositional::new(WordListSequence::new(&four_lists), 4, RANGE).expect("u32");
    let name_pair = MixedPositional::with_formatter_and_permutation(
        WordListSequence::new(&pair_lists),
        Hyphen,
        IdentityPermutation::new(249417),
        2,
        249417,
    )
    .expect("name shape");
    let three =
        Positional::new(nwords::wordlists::bip39::English, 3, RANGE).expect("native u32 preset");
    let fixture = std::env::args().nth(4).expect("fixture path");
    let ids: Vec<u32> = std::fs::read_to_string(fixture)
        .expect("fixture")
        .lines()
        .map(|line| line.parse().expect("u32"))
        .collect();
    assert_eq!(ids.len(), COUNT);
    let decimals: Vec<String> = ids.iter().map(u32::to_string).collect();
    let phrases: Vec<String> = ids
        .iter()
        .map(|id| four.encode(u128::from(*id)).expect("encode"))
        .collect();
    if mode == "vectors" {
        for (id, phrase) in ids.iter().zip(&phrases) {
            println!("{id}\t{phrase}");
        }
        return;
    }
    let three_phrases: Vec<String> = ids
        .iter()
        .map(|id| three.encode(u128::from(*id)).expect("encode"))
        .collect();
    let mnemonic_phrases: Vec<String> = ids
        .iter()
        .map(|id| mnemonic::to_string(id.to_be_bytes()))
        .collect();
    for (i, phrase) in phrases.iter().enumerate() {
        assert_eq!(
            four.decode_words(&phrase.split_whitespace().collect::<Vec<_>>())
                .expect("decode"),
            u128::from(ids[i])
        );
        assert_eq!(
            three
                .decode_words(&split_three(&three_phrases[i]))
                .expect("decode"),
            u128::from(ids[i])
        );
        let mut bytes = Vec::new();
        mnemonic::decode(&mnemonic_phrases[i], &mut bytes).expect("decode mnemonic");
        assert_eq!(bytes, ids[i].to_be_bytes());
    }
    assert_eq!(pair.encode(42).expect("vector"), "able cardinal");
    let adjectives: Vec<&str> = ADJECTIVES.lines().collect();
    let animals: Vec<&str> = ANIMALS.lines().collect();
    assert_eq!((adjectives.len(), animals.len()), (749, 333));
    let mut names_shared = names::Generator::new(&adjectives, &animals, names::Name::Plain);
    let mut names_default = names::Generator::default();
    let pets_shared = petname::Petnames::new(ADJECTIVES, "very", ANIMALS);
    let pets_default = petname::Petnames::default();
    let pet_cardinality = pets_default.cardinality(2);
    let shared_namer = pets_shared.namer(2, "-");
    let default_namer = pets_default.namer(2, "-");
    let mut rng = rand::thread_rng();
    let mut rng10 = rand10::rng();
    let mut baseline08_one = rand::thread_rng();
    let mut baseline08_two = rand::thread_rng();
    let mut baseline10_one = rand10::rng();
    let mut baseline10_two = rand10::rng();
    let mut pet_rng = rand10::rng();
    let mut pet_default_rng = rand10::rng();
    let mut shared_iter = shared_namer.iter(&mut pet_rng);
    let mut default_iter = default_namer.iter(&mut pet_default_rng);
    let mut cases = vec![
        Case {
            name: "nwords/name/shared-rand10",
            run: Box::new(|_| {
                consume(
                    name_pair
                        .encode(rand10::RngExt::random_range(&mut rng10, 0..249417u32).into())
                        .expect("encode"),
                )
            }),
        },
        Case {
            name: "rng/rand08/one-draw",
            run: Box::new(|_| baseline08_one.gen_range(0..249417u32) as u64),
        },
        Case {
            name: "rng/rand08/two-draws",
            run: Box::new(|_| {
                baseline08_two.gen_range(0..749u32) as u64
                    + baseline08_two.gen_range(0..333u32) as u64
            }),
        },
        Case {
            name: "rng/rand10/one-draw",
            run: Box::new(|_| {
                rand10::RngExt::random_range(&mut baseline10_one, 0..249417u32) as u64
            }),
        },
        Case {
            name: "rng/rand10/two-draws",
            run: Box::new(|_| {
                rand10::RngExt::random_range(&mut baseline10_two, 0..749u32) as u64
                    + rand10::RngExt::random_range(&mut baseline10_two, 0..333u32) as u64
            }),
        },
        Case {
            name: "nwords-abi/u32/encode",
            run: Box::new(|i| {
                consume(nwords_js::encode_id_json(
                    &decimals[i],
                    "animal,animal,animal,animal",
                    Some("4294967296".to_owned()),
                ))
            }),
        },
        Case {
            name: "nwords-abi/u32/decode",
            run: Box::new(|i| {
                consume(nwords_js::decode_phrase_json(
                    &phrases[i],
                    "animal,animal,animal,animal",
                    Some("4294967296".to_owned()),
                ))
            }),
        },
        Case {
            name: "nwords-bip39-list/u32/encode",
            run: Box::new(|i| consume(three.encode(u128::from(ids[i])).expect("encode"))),
        },
        Case {
            name: "nwords-bip39-list/u32/decode",
            run: Box::new(|i| {
                three
                    .decode_words(&split_three(&three_phrases[i]))
                    .expect("decode") as u64
            }),
        },
        Case {
            name: "nwords/name/shared-rand08",
            run: Box::new(|_| {
                consume(
                    name_pair
                        .encode(rng.gen_range(0..249417u32).into())
                        .expect("encode"),
                )
            }),
        },
        Case {
            name: "names/name/shared",
            run: Box::new(|_| consume(names_shared.next().expect("name"))),
        },
        Case {
            name: "petname/name/shared-rand10",
            run: Box::new(|_| consume(shared_iter.next().expect("name"))),
        },
        Case {
            name: "names/name/default",
            run: Box::new(|_| consume(names_default.next().expect("name"))),
        },
        Case {
            name: "petname/name/default-rand10",
            run: Box::new(|_| consume(default_iter.next().expect("name"))),
        },
        Case {
            name: "nwords/pair/encode",
            run: Box::new(|i| consume(pair.encode(u128::from(ids[i] % 249417)).expect("encode"))),
        },
        Case {
            name: "nwords/u32/encode",
            run: Box::new(|i| consume(four.encode(u128::from(ids[i])).expect("encode"))),
        },
        Case {
            name: "mnemonic/u32/encode",
            run: Box::new(|i| consume(mnemonic::to_string(ids[i].to_be_bytes()))),
        },
        Case {
            name: "nwords/u32/decode",
            run: Box::new(|i| four.decode_words(&split_four(&phrases[i])).expect("decode") as u64),
        },
        Case {
            name: "mnemonic/u32/decode",
            run: Box::new(|i| {
                let mut out = [0u8; 4];
                mnemonic::decode(&mnemonic_phrases[i], &mut out[..]).expect("decode");
                u32::from_be_bytes(out) as u64
            }),
        },
        Case {
            name: "nwords/setup/pair",
            run: Box::new(|_| {
                black_box(
                    MixedPositional::new(WordListSequence::new(&pair_lists), 2, black_box(249417))
                        .expect("codec"),
                );
                1
            }),
        },
        Case {
            name: "names/setup/default",
            run: Box::new(|_| {
                black_box(names::Generator::default());
                1
            }),
        },
        Case {
            name: "petname/setup/default",
            run: Box::new(|_| {
                black_box(petname::Petnames::default());
                1
            }),
        },
    ];
    if mode == "list" {
        for case in &cases {
            println!("{}", case.name);
        }
        return;
    }
    assert!(cases.iter().any(|case| case.name == mode), "unknown case");
    cases.retain(|case| case.name == mode);
    println!("{{\"kind\":\"metadata\",\"runtime\":\"rust\",\"round\":{round},\"idsChecksum\":{},\"namesDefaultCapacity\":{},\"petnameDefaultCapacity\":{pet_cardinality}}}", ids.iter().map(|id| u64::from(*id)).sum::<u64>(), names::ADJECTIVES.len() * names::NOUNS.len());
    println!("{{\"kind\":\"lengths\",\"runtime\":\"rust\",\"nwords4\":{},\"nwords3\":{},\"mnemonic\":{}}}", phrases.iter().map(String::len).sum::<usize>() as f64 / COUNT as f64, three_phrases.iter().map(String::len).sum::<usize>() as f64 / COUNT as f64, mnemonic_phrases.iter().map(String::len).sum::<usize>() as f64 / COUNT as f64);
    for case in &mut cases {
        let warmup = Instant::now();
        while warmup.elapsed() < Duration::from_millis(100) {
            black_box(batch(case, COUNT));
        }
        let mut count = COUNT;
        loop {
            let (ns, _) = batch(case, count);
            if ns >= target_ms * 1e6 || count >= (1 << 24) {
                break;
            }
            count *= 2;
        }
        for sample_index in 0..3 {
            let (ns, checksum) = batch(case, count);
            println!("{{\"kind\":\"sample\",\"runtime\":\"rust\",\"round\":{round},\"case\":\"{}\",\"sampleIndex\":{sample_index},\"iterations\":{count},\"elapsedNs\":{ns},\"nsPerOp\":{},\"checksum\":{checksum}}}", case.name, ns / count as f64);
        }
    }
}
