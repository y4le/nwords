# Named-shape JavaScript parity vectors

`named-shapes.tsv` was generated with the committed nwords CLI at
`c63efb502f9ece8e6e1517c933e6ddc6b67a5adb` (2026-09-19), before adding
JavaScript bindings. For each row, run `nwords encode ID --shape LISTS`, adding
`--range RANGE` when the range column is not `-`.

These project-authored vectors use the workspace MIT OR Apache-2.0 license.
The ordered underlying lists and their upstream provenance remain in
[adjective-animal](../adjective-animal/README.md). Rust bindings and installed
JavaScript consumers read this same fixture. It includes both three-word
orders, narrowed ranges, `2^53 + 1`, and `u128::MAX - 1` with an explicit
`u128::MAX` range. Do not regenerate it from a changing local executable.
