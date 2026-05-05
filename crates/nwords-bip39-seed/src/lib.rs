#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! BIP-39 seed derivation for `nwords`.

#[cfg(not(feature = "alloc"))]
compile_error!("nwords-bip39-seed requires the alloc feature");

extern crate alloc;

use alloc::string::String;

use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha512;
use unicode_normalization::UnicodeNormalization;

/// BIP-39 PBKDF2 iteration count.
pub const PBKDF2_ROUNDS: u32 = 2048;

/// Length in bytes of a derived BIP-39 seed.
pub const SEED_BYTES: usize = 64;

/// BIP-39 salt prefix.
pub const SALT_PREFIX: &str = "mnemonic";

/// A derived BIP-39 seed.
pub type Seed = [u8; SEED_BYTES];

/// Derives a BIP-39 seed from a mnemonic phrase and passphrase.
///
/// The mnemonic and passphrase are NFKD-normalized before PBKDF2. The salt is
/// `mnemonic` followed by the normalized passphrase, as required by BIP-39.
///
/// This function derives the seed for the text it is given. Validate mnemonic
/// words and checksums separately when accepting user input.
pub fn derive_seed(mnemonic: &str, passphrase: &str) -> Seed {
    let normalized_mnemonic = normalize_nfkd(mnemonic);
    let salt = salt(passphrase);

    pbkdf2_hmac_array::<Sha512, SEED_BYTES>(
        normalized_mnemonic.as_bytes(),
        salt.as_bytes(),
        PBKDF2_ROUNDS,
    )
}

/// Returns the NFKD-normalized form used by BIP-39 seed derivation.
pub fn normalize_nfkd(text: &str) -> String {
    text.nfkd().collect()
}

fn salt(passphrase: &str) -> String {
    let normalized_passphrase = normalize_nfkd(passphrase);
    let mut salt = String::with_capacity(SALT_PREFIX.len() + normalized_passphrase.len());
    salt.push_str(SALT_PREFIX);
    salt.push_str(&normalized_passphrase);
    salt
}

#[cfg(test)]
mod tests {
    use super::{derive_seed, Seed, SEED_BYTES};

    #[test]
    fn english_trezor_seed_vector_matches() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let expected = decode_seed("c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04");

        assert_eq!(derive_seed(mnemonic, "TREZOR"), expected);
    }

    #[test]
    fn japanese_bip32jp_seed_vector_matches() {
        let mnemonic = "あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あおぞら";
        let passphrase = "㍍ガバヴァぱばぐゞちぢ十人十色";
        let expected = decode_seed("a262d6fb6122ecf45be09c50492b31f92e9beb7d9a845987a02cefda57a15f9c467a17872029a9e92299b5cbdf306e3a0ee620245cbd508959b6cb7ca637bd55");

        assert_eq!(derive_seed(mnemonic, passphrase), expected);
    }

    fn decode_seed(hex: &str) -> Seed {
        assert_eq!(hex.len(), SEED_BYTES * 2);

        let mut seed = [0; SEED_BYTES];
        for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
            seed[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
        }
        seed
    }

    fn hex_value(byte: u8) -> u8 {
        match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => panic!("invalid hex byte"),
        }
    }
}
