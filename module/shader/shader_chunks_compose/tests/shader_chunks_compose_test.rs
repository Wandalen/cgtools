//! Direct-call tests for `shader_chunks_compose`'s command logic — no
//! subprocess; see `tests/cli_subprocess_test.rs` in the aggregator for
//! end-to-end argv and exit-code coverage.

use shader_chunks_compose::{ ComposeCliError, chunks_compose, wgsl_try_compose };

#[ test ]
fn compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order()
{
  let output = chunks_compose( &[ "value_noise".to_string(), "hash21".to_string() ], false ).expect( "chunks_compose should succeed" );
  let hash21_pos = output.find( "fn hash21" ).expect( "hash21 present" );
  let value_noise_pos = output.find( "fn value_noise" ).expect( "value_noise present" );
  assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise:\n{output}" );
}

#[ test ]
fn compose_chunks_reports_unknown_chunk_error_for_bogus_name()
{
  let err = chunks_compose( &[ "bogus_chunk".to_string() ], false ).expect_err( "chunks_compose should fail for an unknown name" );
  assert!
  (
    matches!( &err, ComposeCliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
}

#[ test ]
fn compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted()
{
  let err = chunks_compose( &[ "value_noise".to_string() ], false ).expect_err( "chunks_compose should fail on a missing dependency" );
  assert!
  (
    matches!( &err, ComposeCliError::Compose( shader_chunks_core::ComposeError::MissingDependency { .. } ) ),
    "expected Compose(MissingDependency), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn compose_chunks_transitive_closure_equals_the_explicit_full_set()
{
  let closure = chunks_compose( &[ "fbm3".to_string() ], true )
  .expect( "transitive compose of a single root should pull its whole chain" );
  let explicit = chunks_compose
  (
    &[ "hash21".to_string(), "value_noise".to_string(), "fbm3".to_string() ],
    false,
  ).expect( "explicit full set should compose" );
  assert_eq!( closure, explicit, "closure and explicit full set must compose identically" );

  let hash21_pos = closure.find( "fn hash21" ).expect( "hash21 pulled in transitively" );
  let value_noise_pos = closure.find( "fn value_noise" ).expect( "value_noise pulled in transitively" );
  let fbm3_pos = closure.find( "fn fbm3" ).expect( "fbm3 present" );
  assert!
  (
    hash21_pos < value_noise_pos && value_noise_pos < fbm3_pos,
    "closure must compose in dependency order:\n{closure}"
  );
}

#[ test ]
fn compose_chunks_transitive_reports_unknown_chunk_error_for_bogus_name()
{
  // The closure walk resolves every reachable dependency through the same
  // loud lookup as directly-named chunks — a bogus root fails identically
  // under both modes rather than the transitive path masking it.
  let err = chunks_compose( &[ "bogus_chunk".to_string() ], true )
  .expect_err( "transitive compose should fail for an unknown name" );
  assert!
  (
    matches!( &err, ComposeCliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
}

#[ test ]
fn try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture()
{
  const A : &str = "//@ name: a\n//@ depends_on: b\nfn a() {}";
  const B : &str = "//@ name: b\n//@ depends_on: a\nfn b() {}";
  let err = wgsl_try_compose( &[ A, B ] ).expect_err( "wgsl_try_compose should fail on a cyclic dependency" );
  assert!
  (
    matches!( &err, ComposeCliError::Compose( shader_chunks_core::ComposeError::CyclicDependency( _ ) ) ),
    "expected Compose(CyclicDependency), got {err:?}"
  );
}
