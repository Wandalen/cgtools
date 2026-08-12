//! Tests for the manifest-driven shader-chunk composer — manifest/WGSL-body
//! cross-checks over the real bundled chunks plus `compose`'s ordering and
//! panic contracts.

use shader_chunks_core::
{
  compose, try_compose, parse_name, parse_depends_on, parse_description, parse_stage, parse_exports, parse_tags,
  ComposeError, ALL_CHUNKS,
  HASH21, VALUE_NOISE, FBM3, FULLSCREEN_TRIANGLE,
};

/// Test-only: pulls the declared symbol name out of an `export` line's
/// WGSL signature ( `"fn hash21(p: vec2f) -> f32"` -> `"hash21"`,
/// `"struct VertexOutput { .. }"` -> `"VertexOutput"` ).
fn exported_name( signature : &str ) -> &str
{
  signature.split_whitespace().nth( 1 ).unwrap_or( signature )
  .split( '(' ).next().unwrap_or( signature )
}

#[ test ]
fn depends_on_covers_every_actual_wgsl_call_to_another_chunk()
{
  for &chunk in ALL_CHUNKS
  {
    let name = parse_name( chunk );
    let declared = parse_depends_on( chunk );
    for &other in ALL_CHUNKS
    {
      let other_name = parse_name( other );
      if other_name == name
      {
        continue;
      }
      let calls_it = chunk.contains( &format!( "{other_name}(" ) );
      let declares_it = declared.contains( &other_name );
      assert_eq!
      (
        calls_it, declares_it,
        "chunk `{name}`: actual wgsl call to `{other_name}` = {calls_it}, but depends_on lists it = {declares_it}"
      );
    }
  }
}

#[ test ]
fn export_names_match_a_real_declaration_in_the_wgsl_body()
{
  for &chunk in ALL_CHUNKS
  {
    for signature in parse_exports( chunk )
    {
      let name = exported_name( signature );
      let declared = chunk.contains( &format!( "fn {name}(" ) ) || chunk.contains( &format!( "struct {name}" ) );
      assert!( declared, "chunk declares export `{signature}` but no `fn {name}(` or `struct {name}` found in its body" );
    }
  }
}

#[ test ]
fn compose_orders_dependencies_before_dependents_regardless_of_input_order()
{
  let composed = compose( &[ FBM3, FULLSCREEN_TRIANGLE, VALUE_NOISE, HASH21 ] );
  let hash21_pos = composed.find( "fn hash21" ).expect( "hash21 present" );
  let value_noise_pos = composed.find( "fn value_noise" ).expect( "value_noise present" );
  let fbm3_pos = composed.find( "fn fbm3" ).expect( "fbm3 present" );
  assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise" );
  assert!( value_noise_pos < fbm3_pos, "value_noise must precede fbm3" );
}

#[ test ]
#[ should_panic( expected = "was not passed to compose" ) ]
fn compose_panics_on_missing_dependency()
{
  let _ = compose( &[ VALUE_NOISE, FBM3 ] );
}

#[ test ]
#[ should_panic( expected = "cyclic shader-chunk dependency" ) ]
fn compose_panics_on_cyclic_dependency()
{
  const A : &str = "//@ name: a\n//@ depends_on: b\nfn a() {}";
  const B : &str = "//@ name: b\n//@ depends_on: a\nfn b() {}";
  let _ = compose( &[ A, B ] );
}

#[ test ]
fn parse_depends_on_handles_empty_value()
{
  assert_eq!( parse_depends_on( "//@ name: x\n//@ depends_on:\n" ), Vec::< &str >::new() );
}

#[ test ]
fn parse_depends_on_handles_multiple_entries()
{
  assert_eq!( parse_depends_on( "//@ depends_on: a, b\n" ), vec![ "a", "b" ] );
}

#[ test ]
fn all_chunks_lists_every_bundled_chunk()
{
  assert_eq!( ALL_CHUNKS.len(), 4 );
}

#[ test ]
fn parse_description_reads_every_bundled_chunk()
{
  assert_eq!( parse_description( HASH21 ), "Single-value hash of a 2D point into [0, 1)." );
  assert_eq!( parse_description( VALUE_NOISE ), "Bilinear-interpolated value noise sampled at a 2D point, in [0, 1)." );
  assert_eq!( parse_description( FBM3 ), "Fixed 3-octave fractal Brownian motion built on value_noise, in [0, 0.875]." );
  assert_eq!
  (
    parse_description( FULLSCREEN_TRIANGLE ),
    "Fullscreen-triangle vertex stage: 3 vertices, no vertex buffer, vertex_index alone picks the corner."
  );
}

#[ test ]
fn parse_stage_is_some_only_for_the_vertex_chunk()
{
  assert_eq!( parse_stage( HASH21 ), None );
  assert_eq!( parse_stage( VALUE_NOISE ), None );
  assert_eq!( parse_stage( FBM3 ), None );
  assert_eq!( parse_stage( FULLSCREEN_TRIANGLE ), Some( "vertex" ) );
}

#[ test ]
fn parse_exports_counts_match_each_chunk()
{
  assert_eq!( parse_exports( HASH21 ).len(), 1 );
  assert_eq!( parse_exports( VALUE_NOISE ).len(), 1 );
  assert_eq!( parse_exports( FBM3 ).len(), 1 );
  assert_eq!( parse_exports( FULLSCREEN_TRIANGLE ).len(), 2 );
}

#[ test ]
fn parse_tags_reads_every_bundled_chunk()
{
  assert_eq!( parse_tags( HASH21 ), vec![ ( "category", "hash" ) ] );
  assert_eq!( parse_tags( VALUE_NOISE ), vec![ ( "category", "noise" ) ] );
  assert_eq!( parse_tags( FBM3 ), vec![ ( "category", "noise" ), ( "technique", "fractal" ) ] );
  assert_eq!( parse_tags( FULLSCREEN_TRIANGLE ), vec![ ( "category", "vertex" ) ] );
}

#[ test ]
#[ should_panic( expected = "malformed `//@ tags:` entry" ) ]
fn parse_tags_panics_on_malformed_entry()
{
  let _ = parse_tags( "//@ name: x\n//@ tags: not_a_pair\n" );
}

#[ test ]
fn try_compose_matches_compose_output_on_success()
{
  let expected = compose( &[ FBM3, FULLSCREEN_TRIANGLE, VALUE_NOISE, HASH21 ] );
  let actual = try_compose( &[ FBM3, FULLSCREEN_TRIANGLE, VALUE_NOISE, HASH21 ] ).expect( "should succeed" );
  assert_eq!( actual, expected );
}

#[ test ]
fn try_compose_returns_err_on_missing_dependency()
{
  let err = try_compose( &[ VALUE_NOISE, FBM3 ] ).expect_err( "should fail" );
  assert!( matches!( err, ComposeError::MissingDependency { .. } ), "expected MissingDependency, got {err:?}" );
}

#[ test ]
fn try_compose_returns_err_on_cyclic_dependency()
{
  const A : &str = "//@ name: a\n//@ depends_on: b\nfn a() {}";
  const B : &str = "//@ name: b\n//@ depends_on: a\nfn b() {}";
  let err = try_compose( &[ A, B ] ).expect_err( "should fail" );
  assert!( matches!( err, ComposeError::CyclicDependency( _ ) ), "expected CyclicDependency, got {err:?}" );
}
