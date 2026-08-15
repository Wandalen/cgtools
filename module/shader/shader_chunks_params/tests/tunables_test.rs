//! Direct-call tests for `shader_chunks_params`'s command logic — no
//! subprocess; see `tests/cli_subprocess_test.rs` in the aggregator for
//! end-to-end argv and exit-code coverage.

use shader_chunks_params::{ ParamsCliError, tunables, tunables_of_chunk };

/// Mirrors `shader_chunks_params_core/tests/discovery_test.rs`'s own
/// `LOCAL_GLOW` fixture — a test-local chunk carrying `//@ param:` lines,
/// since no bundled chunk declares any (out of scope for this task to
/// change). It shares the name of the bundled `glow` chunk but never
/// touches the registry — it is only ever passed to `tunables_of_chunk`
/// directly.
const LOCAL_GLOW_WGSL : &str = "\
//@ name: glow
//@ description: Doubled value noise, a test-local chunk.
//@ tags: category:test
//@ depends_on: value_noise
//@ export: fn glow(p: vec2f) -> f32
//@ param: octaves argument u32 range(1, 8)
//@ param: seed define u32

fn glow( p : vec2f, octaves : u32, seed : u32 ) -> f32
{
  return value_noise( p ) * 2.0;
}
";

const LOCAL_GLOW : shader_chunks_core::ChunkDescriptor = shader_chunks_core::ChunkDescriptor
{
  name : "glow",
  description : "Doubled value noise, a test-local chunk.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "value_noise" ],
  exports : &[ "fn glow(p: vec2f) -> f32" ],
  wgsl : LOCAL_GLOW_WGSL,
};

#[ test ]
fn tunables_of_chunk_lists_declared_and_inferred_parameters()
{
  let output = tunables_of_chunk( &LOCAL_GLOW ).expect( "tunables_of_chunk should succeed" );

  assert!( output.contains( "octaves" ), "{output}" );
  assert!( output.contains( "Argument" ), "{output}" );
  assert!( output.contains( "U32" ), "{output}" );
  assert!( output.contains( "1..8" ), "declared range should render verbatim:\n{output}" );
  assert!( output.contains( "Declared" ), "{output}" );

  assert!( output.contains( "seed" ), "{output}" );
  assert!( output.contains( "Define" ), "{output}" );
  assert!( output.contains( "0..65535" ), "inferred range for `seed` should be [0, 65535]:\n{output}" );
  assert!( output.contains( "Inferred" ), "{output}" );
}

#[ test ]
fn tunables_zero_declared_params_reports_explicit_message_not_blank_or_error()
{
  let output = tunables( "hash21" ).expect( "tunables should succeed for a bundled chunk with no declared params" );
  assert!( output.contains( "hash21" ), "{output}" );
  assert!( output.contains( "no tunable parameters" ), "empty case must be an explicit message, not blank:\n{output}" );
}

#[ test ]
fn tunables_unknown_chunk_reports_unknown_chunk_error()
{
  let err = tunables( "bogus_chunk" ).expect_err( "tunables should fail for an unknown chunk name" );
  assert!
  (
    matches!( &err, ParamsCliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}
