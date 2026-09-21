# Variable-v1 compatibility vectors

`variable-v1-wide.tsv` freezes canonical `adjective+,animal` output from the
published pure JavaScript codec in commit
`37fec8491eea5babd9a332b8c38a8f24551c4448`. It uses the checked-in
749-adjective and 333-animal wordsets, `min: 1`, and `maxWords: 32`.
Rows cover the first and next tier and IDs around powers of two from 64
through 300 bits. Decimal IDs are exact; the tab-separated phrase is in
encoding order. These fixtures preserve the browser's released phrase mapping
while Rust and its bindings gain wide support.

`variable-v1.tsv` is the older small toy-list fixture for both minimum-zero
and minimum-one variants.
