# tests

Native tests for `minwebgl`'s pure-logic layer (established by task 069), runnable
without a browser via `cargo test -p minwebgl --all-features`. The GL-context/DOM
layer has no runner yet — see the crate readme's Testing section for the full
runnability story, including the two documented-exception inline test modules in
`src/` covering private validation helpers.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| data_type_test.rs | DataType ↔ Const WebGL-constant pinning and roundtrip |
