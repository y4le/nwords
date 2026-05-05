use nwords::schemes::bip39::English;

const ENGLISH_VECTORS: &[(&str, &str)] = &[
    ("00000000000000000000000000000000", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"),
    ("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f", "legal winner thank year wave sausage worth useful legal winner thank yellow"),
    ("80808080808080808080808080808080", "letter advice cage absurd amount doctor acoustic avoid letter advice cage above"),
    ("ffffffffffffffffffffffffffffffff", "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"),
    ("000000000000000000000000000000000000000000000000", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon agent"),
    ("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f", "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal will"),
    ("808080808080808080808080808080808080808080808080", "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter always"),
    ("ffffffffffffffffffffffffffffffffffffffffffffffff", "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo when"),
    ("0000000000000000000000000000000000000000000000000000000000000000", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"),
    ("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f", "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title"),
    ("8080808080808080808080808080808080808080808080808080808080808080", "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless"),
    ("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote"),
    ("9e885d952ad362caeb4efe34a8e91bd2", "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic"),
    ("6610b25967cdcca9d59875f5cb50b0ea75433311869e930b", "gravity machine north sort system female filter attitude volume fold club stay feature office ecology stable narrow fog"),
    ("68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c", "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length"),
    ("c0ba5a8e914111210f2bd131f3d5e08d", "scheme spot photo card baby mountain device kick cradle pact join borrow"),
    ("6d9be1ee6ebd27a258115aad99b7317b9c8d28b6d76431c3", "horn tenant knee talent sponsor spell gate clip pulse soap slush warm silver nephew swap uncle crack brave"),
    ("9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863", "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside"),
    ("23db8160a31d3e0dca3688ed941adbf3", "cat swing flag economy stadium alone churn speed unique patch report train"),
    ("8197a4a47f0425faeaa69deebc05ca29c0a5b5cc76ceacc0", "light rule cinnamon wrap drastic word pride squirrel upgrade then income fatal apart sustain crack supply proud access"),
    ("066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad", "all hour make first leader extend hole alien behind guard gospel lava path output census museum junior mass reopen famous sing advance salt reform"),
    ("f30f8c1da665478f49b001d94c5fc452", "vessel ladder alter error federal sibling chat ability sun glass valve picture"),
    ("c10ec20dc3cd9f652c7fac2f1230f7a3c828389a14392f05", "scissors invite lock maple supreme raw rapid void congress muscle digital elegant little brisk hair mango congress clump"),
    ("f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f", "void come effort suffer camp survey warrior heavy shoot primary clutch crush open amazing screen patrol group space point ten exist slush involve unfold"),
];

#[cfg(feature = "bip39-japanese")]
const JAPANESE_VECTORS: &[(&str, &str)] = &[
    ("00000000000000000000000000000000", "あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あおぞら"),
    ("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f", "そつう　れきだい　ほんやく　わかす　りくつ　ばいか　ろせん　やちん　そつう　れきだい　ほんやく　わかめ"),
    ("80808080808080808080808080808080", "そとづら　あまど　おおう　あこがれる　いくぶん　けいけん　あたえる　いよく　そとづら　あまど　おおう　あかちゃん"),
    ("ffffffffffffffffffffffffffffffff", "われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　ろんぶん"),
    ("000000000000000000000000000000000000000000000000", "あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あらいぐま"),
    ("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f", "そつう　れきだい　ほんやく　わかす　りくつ　ばいか　ろせん　やちん　そつう　れきだい　ほんやく　わかす　りくつ　ばいか　ろせん　やちん　そつう　れいぎ"),
    ("808080808080808080808080808080808080808080808080", "そとづら　あまど　おおう　あこがれる　いくぶん　けいけん　あたえる　いよく　そとづら　あまど　おおう　あこがれる　いくぶん　けいけん　あたえる　いよく　そとづら　いきなり"),
    ("ffffffffffffffffffffffffffffffffffffffffffffffff", "われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　りんご"),
    ("0000000000000000000000000000000000000000000000000000000000000000", "あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　いってい"),
    ("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f", "そつう　れきだい　ほんやく　わかす　りくつ　ばいか　ろせん　やちん　そつう　れきだい　ほんやく　わかす　りくつ　ばいか　ろせん　やちん　そつう　れきだい　ほんやく　わかす　りくつ　ばいか　ろせん　まんきつ"),
    ("8080808080808080808080808080808080808080808080808080808080808080", "そとづら　あまど　おおう　あこがれる　いくぶん　けいけん　あたえる　いよく　そとづら　あまど　おおう　あこがれる　いくぶん　けいけん　あたえる　いよく　そとづら　あまど　おおう　あこがれる　いくぶん　けいけん　あたえる　うめる"),
    ("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　われる　らいう"),
    ("9e885d952ad362caeb4efe34a8e91bd2", "ておくれ　げざん　しねま　こりる　きぼう　しねん　ななおし　ほんやく　きない　けむり　けまり　てんない"),
    ("6610b25967cdcca9d59875f5cb50b0ea75433311869e930b", "しはつ　たいちょう　ちめいど　ひりつ　ほくろ　こやく　こんかい　いひん　よろしい　さくら　がはく　ふっかつ　こまる　つごう　けぬき　ふすま　ちから　さくし"),
    ("68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c", "しやくしょ　くちこみ　どんぶり　けつじょ　おとしもの　くうぐん　どんぶり　たずさわる　ひたむき　みうち　にほん　うわさ　しゃけん　このよ　じどう　ほめる　たいよう　くふう　そんちょう　ろくが　はんこ　せあぶら　くうぐん　そっこう"),
    ("c0ba5a8e914111210f2bd131f3d5e08d", "はいち　ふかい　てんすう　おさない　いろえんぴつ　だんち　くださる　せんちょう　きさらぎ　てきとう　せもたれ　うんどう"),
    ("6d9be1ee6ebd27a258115aad99b7317b9c8d28b6d76431c3", "すいえい　ほとんど　せんやく　ほしい　ふうふ　ひんそう　ざんしょ　がちょう　なにわ　ひはん　ひつじゅひん　られつ　はんぼうき　ちそう　ほいく　めだつ　きさま　えがお"),
    ("9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863", "てそう　こつこつ　えんちょう　じてん　おおや　ぴっちり　だんねつ　ほそく　たなばた　くらべる　ひまん　ていき　あんい　ひんしゅ　ちきん　ざいげん　くたびれる　そなえる　しんか　にいがた　せきむ　けしょう　しあさって　せたい"),
    ("23db8160a31d3e0dca3688ed941adbf3", "おたく　ほうりつ　さいかい　げねつ　ふせい　いいだす　かいてん　ひんしゅ　もえる　てのひら　ねいき　むいか"),
    ("8197a4a47f0425faeaa69deebc05ca29c0a5b5cc76ceacc0", "そむく　のぞく　かいふく　ろてん　げきやく　ろくが　ともだち　ふじみ　やおや　まかせる　すらすら　こぼれる　いぜん　へんたい　きさま　へきが　なたでここ　あさひ"),
    ("066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad", "あんぜん　すうじつ　たいふう　こんぽん　そこそこ　こたつ　しんせいじ　あんこ　うしなう　しまる　じどう　そうり　てはい　ていし　おめでとう　たんまつ　せんげん　たおる　ぬめり　このまま　ひいき　あまい　のらねこ　にんそう"),
    ("f30f8c1da665478f49b001d94c5fc452", "ようきゅう　そあく　いきおい　こうつう　こもじ　はんだん　おんしゃ　あいさつ　へいたく　しすう　ゆうびんきょく　てんぷら"),
    ("c10ec20dc3cd9f652c7fac2f1230f7a3c828389a14392f05", "はえる　せっさたくま　そんみん　たいよう　へこむ　になう　にっさん　よゆう　きあつ　だんぼう　くねくね　けらい　そんけい　えほうまき　しゃうん　たいむ　きあつ　かぶか"),
    ("f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f", "よゆう　かんけい　けぶかい　へいこう　おかず　べんごし　りえき　じゆう　はんい　ともる　かほご　きぬごし　つみき　いきる　はかる　てふだ　しほう　ひろう　とくてん　ほったん　こさめ　ひつじゅひん　せつぞく　めんどう"),
];

#[test]
fn english_trezor_vectors_round_trip() {
    let codec = English::default();
    for &(entropy_hex, mnemonic) in ENGLISH_VECTORS {
        let entropy = decode_hex(entropy_hex);
        assert_eq!(codec.encode_entropy(&entropy).as_deref(), Ok(mnemonic));
        assert_eq!(codec.decode_phrase(mnemonic), Ok(entropy));
    }
}

#[cfg(feature = "bip39-japanese")]
#[test]
fn japanese_trezor_vectors_round_trip() {
    let codec = nwords::schemes::bip39::Japanese::default();
    for &(entropy_hex, mnemonic) in JAPANESE_VECTORS {
        let entropy = decode_hex(entropy_hex);
        assert_eq!(codec.encode_entropy(&entropy).as_deref(), Ok(mnemonic));
        assert_eq!(codec.decode_phrase(mnemonic), Ok(entropy));
    }
}

fn decode_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let raw = hex.as_bytes();
    for pair in raw.chunks_exact(2) {
        bytes.push((hex_value(pair[0]) << 4) | hex_value(pair[1]));
    }
    bytes
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("invalid hex byte"),
    }
}
