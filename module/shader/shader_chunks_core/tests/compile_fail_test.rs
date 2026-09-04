//! Pins the crate's two compile-time failure guarantees as real failing
//! builds, via `trybuild`: an unknown name passed to `chunk` in `const`
//! position ( `compile_fail/unknown_chunk_name.rs` ), and a
//! `dependency_closed` assert over a set missing a transitive dependency
//! ( `compile_fail/unclosed_set.rs` ). The success halves of both
//! guarantees live in `shader_chunks_core_test.rs`
//! ( `chunk_imports_a_bundled_descriptor_by_value_in_const_position`,
//! `MIXED_SET`'s const assert ); these fixtures prove the failing halves
//! actually fail, with the exact diagnostics snapshotted in the fixtures'
//! `*.stderr` files ( regenerate after a toolchain bump with
//! `TRYBUILD=overwrite` ).

#[ test ]
fn const_position_misuse_fails_the_build()
{
  let cases = trybuild::TestCases::new();
  cases.compile_fail( "tests/compile_fail/*.rs" );
}
