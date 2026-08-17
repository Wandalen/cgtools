# tests

Native tests for `minwebgl`'s pure-logic layer (established by task 069), runnable
without a browser via `cargo test -p minwebgl --all-features`. The GL-context/DOM
layer now has a scripted browser-side runner too (`manual/`, via `browsee`) — see
the crate readme's Testing section for the full runnability story.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| data_type_test.rs | DataType ↔ Const WebGL-constant pinning and roundtrip |
| clean_test.rs | Verifies attachment-id conversion rejects out-of-range ids |
| geometry_test.rs | Verifies natoms validation accepts 1-4, rejects the rest |
| drawbuffers_test.rs | Verifies color-attachment index validation rejects out-of-range indices (BUG-159) |
| sprite_upload_test.rs | Verifies sprite mip-level count and row/col position helpers (BUG-160, BUG-161) |
| diagnostics_test.rs | Verifies `diagnostics`'s `obj` re-export resolves standalone (BUG-274) |
| uniform_test.rs | Verifies the f32 matrix-upload length-error message content (BUG-277) |
| enabled_feature_web_gate_test.rs | Verifies `exec_loop`/`log` resolve under `enabled` alone (BUG-279) |
| manual/ | Scripted `browsee` browser-side pixel-verification procedure for the GL-context/DOM layer |
