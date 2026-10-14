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
//!
//! The snapshots assume the toolchain's `rust-src` component is installed:
//! `unknown_chunk_name`'s panic is raised inside `core`'s `panic!` macro,
//! and rustc quotes that line of `core/src/panic.rs` only when it has the
//! standard library's source. Without it the diagnostic loses that line and
//! no longer matches, so the test checks for the component first.

use std::{ path::PathBuf, process::Command };

/// Where the active toolchain's `rust-src` component puts `core`'s
/// `panic.rs`, or `None` when `rustc` can't report its sysroot. `rustc` is
/// the toolchain cargo runs this test under: rustup passes the selection on
/// to child processes through `RUSTUP_TOOLCHAIN`.
fn core_panic_source() -> Option< PathBuf >
{
  let output = Command::new( "rustc" ).args( [ "--print", "sysroot" ] ).output().ok()?;
  let sysroot = String::from_utf8( output.stdout ).ok()?;
  Some( PathBuf::from( sysroot.trim() ).join( "lib/rustlib/src/rust/library/core/src/panic.rs" ) )
}

#[ test ]
fn const_position_misuse_fails_the_build()
{
  let panic_source = core_panic_source().expect( "`rustc --print sysroot` failed" );
  assert!
  (
    panic_source.is_file(),
    "the compile-fail snapshots need the `rust-src` component ( `rustup component add rust-src` ): \
     without it rustc can't quote {}, so `unknown_chunk_name`'s diagnostic differs from its snapshot",
    panic_source.display(),
  );

  let cases = trybuild::TestCases::new();
  cases.compile_fail( "tests/compile_fail/*.rs" );
}
