//! Tests for the manifest-driven shader-chunk composer — manifest/WGSL-body
//! cross-checks over the real bundled chunks plus `compose`'s ordering and
//! panic contracts.

use shader_chunks_core::
{
  compose, try_compose, parse_name, parse_depends_on, parse_description, parse_stage, parse_exports, parse_tags,
  ComposeError, CHUNKS, chunk_get,
};

/// Test-only: pulls the declared symbol name out of an `export` line's
/// WGSL signature ( `"fn hash21(p: vec2f) -> f32"` -> `"hash21"`,
/// `"struct VertexOutput { .. }"` -> `"VertexOutput"` ).
fn exported_name( signature : &str ) -> &str
{
  signature.split_whitespace().nth( 1 ).unwrap_or( signature )
  .split( '(' ).next().unwrap_or( signature )
}

/// Test-only: the bundled chunk `name`'s full WGSL source, via [`chunk_get`].
fn wgsl( name : &str ) -> &'static str
{
  chunk_get( name ).unwrap_or_else( || panic!( "chunk `{name}` should be bundled" ) ).wgsl
}

#[ test ]
fn depends_on_covers_every_actual_wgsl_call_to_another_chunk()
{
  for chunk in CHUNKS
  {
    let name = chunk.name;
    let declared = parse_depends_on( chunk.wgsl );
    for other in CHUNKS
    {
      let other_name = other.name;
      if other_name == name
      {
        continue;
      }
      let calls_it = chunk.wgsl.contains( &format!( "{other_name}(" ) );
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
  for chunk in CHUNKS
  {
    for signature in parse_exports( chunk.wgsl )
    {
      let name = exported_name( signature );
      let declared = chunk.wgsl.contains( &format!( "fn {name}(" ) ) || chunk.wgsl.contains( &format!( "struct {name}" ) );
      assert!( declared, "chunk declares export `{signature}` but no `fn {name}(` or `struct {name}` found in its body" );
    }
  }
}

#[ test ]
fn compose_orders_dependencies_before_dependents_regardless_of_input_order()
{
  let composed = compose( &[ wgsl( "fbm3" ), wgsl( "fullscreen_triangle" ), wgsl( "value_noise" ), wgsl( "hash21" ) ] );
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
  let _ = compose( &[ wgsl( "value_noise" ), wgsl( "fbm3" ) ] );
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
fn chunks_table_lists_every_bundled_chunk()
{
  assert_eq!( CHUNKS.len(), 4 );
}

#[ test ]
fn chunks_table_names_match_each_manifest()
{
  for chunk in CHUNKS
  {
    assert_eq!
    (
      chunk.name,
      parse_name( chunk.wgsl ),
      "descriptor name `{}` must mirror its chunk's `//@ name:` manifest line",
      chunk.name
    );
  }
}

#[ test ]
fn chunks_table_descriptions_match_each_manifest()
{
  for chunk in CHUNKS
  {
    assert_eq!
    (
      chunk.description,
      parse_description( chunk.wgsl ),
      "descriptor description for `{}` must mirror its `//@ description:` manifest line",
      chunk.name
    );
  }
}

#[ test ]
fn chunks_table_tags_match_each_manifest()
{
  for chunk in CHUNKS
  {
    assert_eq!
    (
      parse_tags( chunk.wgsl ),
      chunk.tags,
      "descriptor tags for `{}` must mirror its `//@ tags:` manifest line",
      chunk.name
    );
  }
}

#[ test ]
fn chunks_table_stages_match_each_manifest()
{
  for chunk in CHUNKS
  {
    assert_eq!
    (
      chunk.stage,
      parse_stage( chunk.wgsl ),
      "descriptor stage for `{}` must mirror its `//@ stage:` manifest line ( or absence )",
      chunk.name
    );
  }
}

#[ test ]
fn chunks_table_depends_on_match_each_manifest()
{
  for chunk in CHUNKS
  {
    assert_eq!
    (
      parse_depends_on( chunk.wgsl ),
      chunk.depends_on,
      "descriptor depends_on for `{}` must mirror its `//@ depends_on:` manifest line",
      chunk.name
    );
  }
}

#[ test ]
fn chunks_table_exports_match_each_manifest()
{
  for chunk in CHUNKS
  {
    assert_eq!
    (
      parse_exports( chunk.wgsl ),
      chunk.exports,
      "descriptor exports for `{}` must mirror its `//@ export:` manifest lines, in file order",
      chunk.name
    );
  }
}

#[ test ]
fn chunk_get_resolves_every_bundled_name_to_its_row()
{
  for chunk in CHUNKS
  {
    assert_eq!( chunk_get( chunk.name ), Some( chunk ) );
  }
}

#[ test ]
fn chunk_get_returns_none_for_unknown_name()
{
  assert_eq!( chunk_get( "no_such_chunk" ), None );
}

#[ test ]
fn parse_description_reads_every_bundled_chunk()
{
  assert_eq!( parse_description( wgsl( "hash21" ) ), "Single-value hash of a 2D point into [0, 1)." );
  assert_eq!( parse_description( wgsl( "value_noise" ) ), "Bilinear-interpolated value noise sampled at a 2D point, in [0, 1)." );
  assert_eq!( parse_description( wgsl( "fbm3" ) ), "Fixed 3-octave fractal Brownian motion built on value_noise, in [0, 0.875]." );
  assert_eq!
  (
    parse_description( wgsl( "fullscreen_triangle" ) ),
    "Fullscreen-triangle vertex stage: 3 vertices, no vertex buffer, vertex_index alone picks the corner."
  );
}

#[ test ]
fn parse_stage_is_some_only_for_the_vertex_chunk()
{
  assert_eq!( parse_stage( wgsl( "hash21" ) ), None );
  assert_eq!( parse_stage( wgsl( "value_noise" ) ), None );
  assert_eq!( parse_stage( wgsl( "fbm3" ) ), None );
  assert_eq!( parse_stage( wgsl( "fullscreen_triangle" ) ), Some( "vertex" ) );
}

#[ test ]
fn parse_exports_counts_match_each_chunk()
{
  assert_eq!( parse_exports( wgsl( "hash21" ) ).len(), 1 );
  assert_eq!( parse_exports( wgsl( "value_noise" ) ).len(), 1 );
  assert_eq!( parse_exports( wgsl( "fbm3" ) ).len(), 1 );
  assert_eq!( parse_exports( wgsl( "fullscreen_triangle" ) ).len(), 2 );
}

#[ test ]
fn parse_tags_reads_every_bundled_chunk()
{
  assert_eq!( parse_tags( wgsl( "hash21" ) ), vec![ ( "category", "hash" ) ] );
  assert_eq!( parse_tags( wgsl( "value_noise" ) ), vec![ ( "category", "noise" ) ] );
  assert_eq!( parse_tags( wgsl( "fbm3" ) ), vec![ ( "category", "noise" ), ( "technique", "fractal" ) ] );
  assert_eq!( parse_tags( wgsl( "fullscreen_triangle" ) ), vec![ ( "category", "vertex" ) ] );
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
  let expected = compose( &[ wgsl( "fbm3" ), wgsl( "fullscreen_triangle" ), wgsl( "value_noise" ), wgsl( "hash21" ) ] );
  let actual = try_compose( &[ wgsl( "fbm3" ), wgsl( "fullscreen_triangle" ), wgsl( "value_noise" ), wgsl( "hash21" ) ] ).expect( "should succeed" );
  assert_eq!( actual, expected );
}

#[ test ]
fn try_compose_returns_err_on_missing_dependency()
{
  let err = try_compose( &[ wgsl( "value_noise" ), wgsl( "fbm3" ) ] ).expect_err( "should fail" );
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
