# EFF long wordlist

Source: Electronic Frontier Foundation, Joseph Bonneau.
https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt
Retrieved 2026-09-20. The dated source URL and SHA256SUMS pin this snapshot.
The source snapshot is packaged at crates/nwords-wordlists/fixtures/eff_large_wordlist.txt.

The EFF website copyright policy (https://www.eff.org/copyright), retrieved
2026-09-20, licenses original material under CC BY 4.0 unless otherwise noted.
The license is in ../licenses/eff-CC-BY-4.0.LICENSE. Attribution: Electronic
Frontier Foundation, “EFF's New Wordlists for Random Passphrases”, Joseph
Bonneau, https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases.
No endorsement is implied.

Transformation: remove the five-digit dice-roll column and retain all 7,776
words in original order, including drop-down, felt-tip, t-shirt, and yo-yo.
No filtering, additions, or spelling changes. words.txt records the transformed
list; crates/nwords-wordlists/src/eff_long.rs contains the same Rust array.
`eff-long` is immutable; future changes require a new name. Its role is `either`.

Reproduce: split each nonempty source line on whitespace and take column two.
The sequence has all rolls from 11111 through 66666 in base-six order.
