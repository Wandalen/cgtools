# tests

Native tests for `minwebgl`'s pure-logic layer (established by task 069), runnable
without a browser via `cargo test -p minwebgl --all-features`. The GL-context/DOM
layer has no runner yet — see the crate readme's Testing section for the full
runnability story.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| data_type_test.rs | DataType ↔ Const WebGL-constant pinning and roundtrip |
| clean_test.rs | Verifies attachment-id conversion rejects out-of-range ids |
| geometry_test.rs | Verifies natoms validation accepts 1-4, rejects the rest |
