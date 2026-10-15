# Compile-Fail Fixtures

Each `.rs` file here is a program that must FAIL to compile, proving one of
the crate's compile-time guarantees actually fires; its matching `.stderr`
file snapshots the expected rustc diagnostic. `compile_fail_test.rs` runs
them all through `trybuild`; after a toolchain bump shifts diagnostic
wording, regenerate the snapshots with `TRYBUILD=overwrite` and review the
diff before accepting it.

The snapshots are taken with the toolchain's `rust-src` component
installed (`rustup component add rust-src`): rustc quotes the line of
`core`'s `panic!` macro that `unknown_chunk_name` fails in only when it has
the standard library's source, so without the component the diagnostic
differs. `compile_fail_test.rs` checks for the component first and says so.

| File | Responsibility |
|------|----------------|
| unknown_chunk_name.rs | Typo'd `chunk( name )` import must fail the build |
| unclosed_set.rs | `dependency_closed` assert over an incomplete set must fail |
