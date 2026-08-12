//! Tests for the manifest-driven shader-chunk composer — manifest/WGSL-body
//! cross-checks over the real bundled chunks plus `compose`'s ordering and
//! panic contracts.

use shader_chunks::
{
  compose, parse_name, parse_depends_on,
  HASH21, VALUE_NOISE, FBM3, FULLSCREEN_TRIANGLE,
};

const ALL_CHUNKS : &[ &str ] = &[ HASH21, VALUE_NOISE, FBM3, FULLSCREEN_TRIANGLE ];

/// Test-only: collects every `//@ key: value` line in `wgsl`, in file
/// order. Unlike the crate's own private `manifest_field`, this is never
/// needed by `compose()` — only `export` repeats per chunk, and only the
/// test below cares about it — so it lives here rather than the library,
/// where a fn unused outside tests would be dead code in a non-test build.
fn manifest_fields<'a>( wgsl : &'a str, key : &str ) -> Vec< &'a str >
{
  let prefix = format!( "//@ {key}:" );
  wgsl.lines()
  .filter_map( | line | line.strip_prefix( prefix.as_str() ) )
  .map( str::trim )
  .collect()
}

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
    for signature in manifest_fields( chunk, "export" )
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
