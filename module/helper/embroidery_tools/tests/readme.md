# tests

Integration tests for `embroidery_tools`, exercising the crate's public API only
(relocated from inline `#[ cfg( test ) ]` modules by task 066). Fixtures live in
`../test_files/` — reference PES/PEC files the format tests pin against; test
binaries run with the crate root as working directory, so fixture paths are
relative (`test_files/...`).

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| embroidery_file_test.rs | EmbroideryFile stitch accumulation, bounds, block splitting |
| pes_test.rs | PES writer fixture pinning and v6 metadata roundtrip |
| pec_test.rs | PEC sample decoding and encoding roundtrip |
