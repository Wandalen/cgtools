//! Tests for the manifest-driven shader-chunk composer — manifest/WGSL-body
//! cross-checks over the real bundled chunks plus `compose`'s ordering and
//! panic contracts.

use shader_chunks_core::
{
  compose, try_compose, set_compose, set_try_compose, set_resolve, depends_on_parse, description_parse,
  stage_parse, exports_parse, tags_parse, ComposeError, ResolveError, ChunkDescriptor, CHUNKS, chunk_get,
  chunk, chunk_get_from, dependency_closed, manifest_mismatches,
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

// A mixed chunk set — two bundled chunks imported by name plus one chunk
// defined locally right here, its manifest and body inline — selected and
// dependency-validated entirely in `const` position. These items compiling
// at all IS the compile-time-contract test ( a typo'd `chunk` name or a
// missing dependency fails the build ); the `#[ test ]` fns below assert
// on their values.

const LOCAL_GLOW_WGSL : &str = "\
//@ name: glow
//@ description: Doubled value noise, a test-local chunk.
//@ tags: category:test
//@ depends_on: value_noise
//@ export: fn glow(p: vec2f) -> f32

fn glow( p : vec2f ) -> f32
{
  return value_noise( p, 0.0 ) * 2.0;
}
";

const LOCAL_GLOW : ChunkDescriptor = ChunkDescriptor
{
  name : "glow",
  description : "Doubled value noise, a test-local chunk.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "value_noise" ],
  exports : &[ "fn glow(p: vec2f) -> f32" ],
  wgsl : LOCAL_GLOW_WGSL,
};

const MIXED_SET : &[ ChunkDescriptor ] =
&[
  chunk( "hash21" ),
  chunk( "value_noise" ),
  LOCAL_GLOW,
];

const _ : () = assert!( dependency_closed( MIXED_SET ) );

#[ test ]
fn depends_on_covers_every_actual_wgsl_call_to_another_chunk()
{
  for chunk in CHUNKS
  {
    let name = chunk.name;
    let declared = depends_on_parse( chunk.wgsl );
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
    for signature in exports_parse( chunk.wgsl )
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
  assert_eq!( depends_on_parse( "//@ name: x\n//@ depends_on:\n" ), Vec::< &str >::new() );
}

#[ test ]
fn parse_depends_on_handles_multiple_entries()
{
  assert_eq!( depends_on_parse( "//@ depends_on: a, b\n" ), vec![ "a", "b" ] );
}

#[ test ]
fn chunks_table_lists_every_bundled_chunk()
{
  assert_eq!( CHUNKS.len(), 50 );
}

#[ test ]
fn chunks_table_matches_each_manifest()
{
  for chunk in CHUNKS
  {
    let mismatches = manifest_mismatches( chunk );
    assert!
    (
      mismatches.is_empty(),
      "descriptor for `{}` must mirror every field of its `//@` manifest: {mismatches:#?}",
      chunk.name
    );
  }
}

#[ test ]
fn manifest_mismatches_reports_every_drifted_field()
{
  let drifted = ChunkDescriptor
  {
    name : "wrong",
    description : "Wrong.",
    tags : &[ ( "category", "wrong" ) ],
    stage : Some( "vertex" ),
    depends_on : &[ "hash21" ],
    exports : &[ "fn wrong() -> f32" ],
    wgsl : LOCAL_GLOW_WGSL,
  };
  let mismatches = manifest_mismatches( &drifted );
  assert_eq!( mismatches.len(), 6, "one mismatch per drifted field, got: {mismatches:#?}" );
  for field in [ "name", "description", "tags", "stage", "depends_on", "export" ]
  {
    assert!
    (
      mismatches.iter().any( | mismatch | mismatch.contains( field ) ),
      "no mismatch message mentions `{field}`: {mismatches:#?}"
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
fn chunk_imports_a_bundled_descriptor_by_value_in_const_position()
{
  const IMPORTED : ChunkDescriptor = chunk( "value_noise" );
  assert_eq!( Some( &IMPORTED ), chunk_get( "value_noise" ) );
}

#[ test ]
#[ should_panic( expected = "unknown chunk name" ) ]
fn chunk_panics_for_unknown_name_at_runtime()
{
  let _ = chunk( "no_such_chunk" );
}

#[ test ]
fn chunk_get_from_resolves_imported_and_local_rows_of_a_mixed_set()
{
  let local = chunk_get_from( MIXED_SET, "glow" ).expect( "local row must resolve" );
  assert_eq!( local, &LOCAL_GLOW );
  let imported = chunk_get_from( MIXED_SET, "hash21" ).expect( "imported row must resolve" );
  assert_eq!( Some( imported ), chunk_get( "hash21" ) );
  assert!( chunk_get_from( MIXED_SET, "fbm3" ).is_none(), "unselected bundled chunk must not resolve from the set" );
}

#[ test ]
fn dependency_closed_is_false_when_a_dependency_is_missing_from_the_set()
{
  assert!( dependency_closed( MIXED_SET ) );
  assert!( !dependency_closed( &[ chunk( "fbm3" ) ] ), "fbm3 without value_noise must not count as closed" );
}

#[ test ]
fn compose_set_orders_a_mixed_set_dependency_before_dependent()
{
  let composed = set_compose( MIXED_SET );
  let hash21_pos = composed.find( "fn hash21" ).expect( "hash21 present" );
  let value_noise_pos = composed.find( "fn value_noise" ).expect( "value_noise present" );
  let glow_pos = composed.find( "fn glow" ).expect( "local glow chunk present" );
  assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise" );
  assert!( value_noise_pos < glow_pos, "value_noise must precede the local chunk depending on it" );
}

#[ test ]
fn try_compose_set_reports_missing_dependency()
{
  let err = set_try_compose( &[ chunk( "fbm3" ) ] ).expect_err( "should fail" );
  assert!( matches!( err, ComposeError::MissingDependency { .. } ), "expected MissingDependency, got {err:?}" );
}

#[ test ]
fn local_chunk_descriptor_matches_its_manifest()
{
  let mismatches = manifest_mismatches( &LOCAL_GLOW );
  assert!( mismatches.is_empty(), "{mismatches:#?}" );
}

#[ test ]
fn parse_description_reads_every_bundled_chunk()
{
  assert_eq!( description_parse( wgsl( "hash21" ) ), "Single-value hash of a 2D point into [0, 1)." );
  assert_eq!( description_parse( wgsl( "value_noise" ) ), "Bilinear-interpolated value noise sampled at a 2D point, in [0, 1)." );
  assert_eq!( description_parse( wgsl( "fbm3" ) ), "Fixed 3-octave fractal Brownian motion built on value_noise, in [0, 0.5*(1+gain+gain^2)]." );
  assert_eq!
  (
    description_parse( wgsl( "fullscreen_triangle" ) ),
    "Fullscreen-triangle vertex stage: 3 vertices, no vertex buffer, vertex_index alone picks the corner."
  );
}

#[ test ]
fn parse_stage_is_some_only_for_the_vertex_chunk()
{
  assert_eq!( stage_parse( wgsl( "hash21" ) ), None );
  assert_eq!( stage_parse( wgsl( "value_noise" ) ), None );
  assert_eq!( stage_parse( wgsl( "fbm3" ) ), None );
  assert_eq!( stage_parse( wgsl( "fullscreen_triangle" ) ), Some( "vertex" ) );
}

#[ test ]
fn parse_exports_counts_match_each_chunk()
{
  assert_eq!( exports_parse( wgsl( "hash21" ) ).len(), 1 );
  assert_eq!( exports_parse( wgsl( "value_noise" ) ).len(), 1 );
  assert_eq!( exports_parse( wgsl( "fbm3" ) ).len(), 1 );
  assert_eq!( exports_parse( wgsl( "fullscreen_triangle" ) ).len(), 2 );
}

#[ test ]
fn parse_tags_reads_every_bundled_chunk()
{
  assert_eq!( tags_parse( wgsl( "hash21" ) ), vec![ ( "category", "hash" ) ] );
  assert_eq!( tags_parse( wgsl( "value_noise" ) ), vec![ ( "category", "noise" ) ] );
  assert_eq!( tags_parse( wgsl( "fbm3" ) ), vec![ ( "category", "noise" ), ( "technique", "fractal" ) ] );
  assert_eq!( tags_parse( wgsl( "fullscreen_triangle" ) ), vec![ ( "category", "vertex" ) ] );
}

#[ test ]
#[ should_panic( expected = "malformed `//@ tags:` entry" ) ]
fn parse_tags_panics_on_malformed_entry()
{
  let _ = tags_parse( "//@ name: x\n//@ tags: not_a_pair\n" );
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

#[ test ]
fn set_resolve_returns_named_chunks_in_given_order()
{
  let resolved = set_resolve( &[ "fbm3", "hash21" ], false ).expect( "both names are bundled" );
  let names : Vec< &str > = resolved.iter().map( | chunk | chunk.name ).collect();
  assert_eq!( names, vec![ "fbm3", "hash21" ] );
}

#[ test ]
fn set_resolve_transitive_widens_to_full_dependency_closure()
{
  let resolved = set_resolve( &[ "fbm3" ], true ).expect( "fbm3 and its closure are bundled" );
  let names : Vec< &str > = resolved.iter().map( | chunk | chunk.name ).collect();
  assert_eq!( names[ 0 ], "fbm3", "named chunks come first, in given order" );
  assert!( names.contains( &"value_noise" ) && names.contains( &"hash21" ), "closure must pull in fbm3's whole chain, got {names:?}" );
  assert_eq!( names.len(), 3, "each closure member appears exactly once" );
}

#[ test ]
fn set_resolve_rejects_unknown_name()
{
  let err = set_resolve( &[ "bogus_chunk" ], false ).expect_err( "should fail" );
  assert_eq!( err, ResolveError::UnknownChunk( "bogus_chunk".to_string() ) );
}

#[ test ]
fn set_resolve_feeds_set_try_compose_identically_to_explicit_selection()
{
  let closure = set_resolve( &[ "fbm3" ], true ).expect( "resolves" );
  let closure : Vec< ChunkDescriptor > = closure.into_iter().copied().collect();
  let explicit = [ chunk( "hash21" ), chunk( "value_noise" ), chunk( "fbm3" ) ];
  assert_eq!
  (
    set_try_compose( &closure ).expect( "composes" ),
    set_try_compose( &explicit ).expect( "composes" ),
    "topological sort makes closure-selected and explicitly-selected sets compose to identical text"
  );
}
