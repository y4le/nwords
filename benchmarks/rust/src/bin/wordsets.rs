use nwords::{
    positional::MixedPositional,
    radix_bytes::RadixBytes,
    variable::VariablePositional,
    wordlists::named::{NamedWordList as L, WordListSequence},
};
use std::{hint::black_box, time::Instant};
fn measure(name: &str, words: usize, mut operation: impl FnMut(usize) -> usize) {
    let mut sink = 0;
    for i in 0..5000 {
        sink ^= black_box(operation(i));
    }
    let mut samples = Vec::new();
    for _ in 0..9 {
        let start = Instant::now();
        for i in 0..20000 {
            sink ^= black_box(operation(i));
        }
        samples.push(start.elapsed().as_nanos() as f64 / 20000.0);
    }
    let mut sorted = samples.clone();
    sorted.sort_by(f64::total_cmp);
    println!("{{\"name\":\"{name}\",\"words\":{words},\"medianNs\":{},\"q1Ns\":{},\"q3Ns\":{},\"samplesNs\":{:?},\"sink\":{sink}}}",sorted[4],sorted[2],sorted[6],samples);
}
fn main() {
    let map = WordListSequence::new(&[L::EffLong, L::EffLong, L::EffLong]);
    let fixed = MixedPositional::new(map, 3, 1 << 32).unwrap();
    let ids = (0..256).map(|i| i * 1234567u128).collect::<Vec<_>>();
    let phrases = ids
        .iter()
        .map(|id| fixed.encode(*id).unwrap())
        .collect::<Vec<_>>();
    measure("eff-u32/encode", 3, |i| {
        fixed.encode(black_box(ids[i & 255])).unwrap().len()
    });
    measure("eff-u32/decode", 3, |i| {
        fixed
            .decode_words(
                &black_box(phrases[i & 255].as_str())
                    .split_whitespace()
                    .collect::<Vec<_>>(),
            )
            .unwrap() as usize
    });
    let variable = VariablePositional::new(
        WordListSequence::new(&[L::Adjective, L::Animal]),
        1,
        0,
        Some(1 << 32),
        None,
    )
    .unwrap();
    let phrases = ids
        .iter()
        .map(|id| variable.encode(*id).unwrap())
        .collect::<Vec<_>>();
    measure("variable-u32/encode", variable.required_words(), |i| {
        variable.encode(black_box(ids[i & 255])).unwrap().len()
    });
    measure("variable-u32/decode", variable.required_words(), |i| {
        variable
            .decode_words(
                &black_box(phrases[i & 255].as_str())
                    .split_whitespace()
                    .collect::<Vec<_>>(),
            )
            .unwrap() as usize
    });
    let codec = RadixBytes::new(WordListSequence::new(&[L::EffLong]), 1, 4096).unwrap();
    let payload = (0..16).map(|i| i * 13).collect::<Vec<u8>>();
    let phrase = codec.encode_bytes(black_box(&payload)).unwrap();
    assert_eq!(
        codec
            .decode_words(
                &black_box(phrase.as_str())
                    .split_whitespace()
                    .collect::<Vec<_>>()
            )
            .unwrap(),
        payload
    );
    measure(
        "eff-bytes-16/encode",
        phrase.split_whitespace().count(),
        |_| codec.encode_bytes(black_box(&payload)).unwrap().len(),
    );
    measure(
        "eff-bytes-16/decode",
        phrase.split_whitespace().count(),
        |_| {
            codec
                .decode_words(
                    &black_box(phrase.as_str())
                        .split_whitespace()
                        .collect::<Vec<_>>(),
                )
                .unwrap()
                .len()
        },
    );
    let mnemonic = mnemonic::to_string(black_box(&payload));
    let mut out = [0u8; 16];
    mnemonic::decode(black_box(mnemonic.as_str()), &mut out[..]).unwrap();
    assert_eq!(&out[..], payload);
    measure(
        "mnemonic-bytes-16/encode",
        mnemonic.split('-').filter(|word| !word.is_empty()).count(),
        |_| mnemonic::to_string(black_box(&payload)).len(),
    );
    measure(
        "mnemonic-bytes-16/decode",
        mnemonic.split('-').filter(|word| !word.is_empty()).count(),
        |_| {
            let mut out = [0u8; 16];
            mnemonic::decode(black_box(mnemonic.as_str()), &mut out[..]).unwrap();
            black_box(out);
            out.len()
        },
    );
}
