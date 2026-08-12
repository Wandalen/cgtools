//! Tests for `shader_source`: the assembled WGSL's structural honesty
//! ( every declaration exactly once, dependencies ahead of dependents ), the
//! `shader/scene_fragment.wgsl`-vs-`scene.rs` fixed-size-array sync that
//! used to be kept by hand, and a full native naga parse + validation — the
//! one check that catches actual WGSL errors before a browser ever sees
//! them.

use sun_grid_lines::scene;
use sun_grid_lines::shader_source::{ assemble, FRAGMENT_WGSL };

/// Returns `source` with every whole-line `//` comment dropped, so chunk
/// manifest headers ( `//@ export: fn hash21(...)` ) and prose comments
/// can't shadow or masquerade as real declarations.
fn code_only( source : &str ) -> String
{
  source.lines()
  .filter( | line | !line.trim_start().starts_with( "//" ) )
  .collect::< Vec< _ > >()
  .join( "\n" )
}

/// Counts code lines of `source` containing `pattern` — comment lines
/// excluded via [`code_only`].
fn code_occurrences( source : &str, pattern : &str ) -> usize
{
  code_only( source ).lines().filter( | line | line.contains( pattern ) ).count()
}

#[ test ]
fn assembled_shader_declares_every_symbol_exactly_once()
{
  let shader = assemble();
  for declaration in
  [
    "fn hash21(",
    "fn value_noise(",
    "fn fbm3(",
    "fn vs_main(",
    "struct VertexOutput",
    "fn fs_main(",
  ]
  {
    assert_eq!
    (
      code_occurrences( &shader, declaration ), 1,
      "assembled shader must declare `{declaration}` exactly once"
    );
  }
}

#[ test ]
fn fragment_body_redeclares_no_chunk_symbol_and_consumes_them()
{
  let fragment_wgsl = FRAGMENT_WGSL;
  for chunk_declaration in [ "fn hash21(", "fn value_noise(", "fn fbm3(", "fn vs_main(", "struct VertexOutput" ]
  {
    assert_eq!
    (
      code_occurrences( &fragment_wgsl, chunk_declaration ), 0,
      "shader/scene_fragment.wgsl must not carry its own copy of `{chunk_declaration}` — it comes from shader_chunks"
    );
  }

  // What makes the chunks live code rather than dead weight: `fs_main`
  // takes the vertex stage's output type and calls into the noise stack
  // ( `value_noise` only indirectly, through `fbm3` ).
  for consumed in [ "VertexOutput", "hash21(", "fbm3(" ]
  {
    assert!
    (
      code_occurrences( &fragment_wgsl, consumed ) > 0,
      "shader/scene_fragment.wgsl must consume `{consumed}`"
    );
  }
}

// The structural tests above check composition honesty, not language
// validity — a typo inside a function body would sail through all of them
// and only explode at ShaderModule creation in the browser. Parsing and
// validating with naga ( the same front end wgpu itself uses ) fails such
// errors natively, at test time.
#[ test ]
fn assembled_wgsl_parses_and_validates()
{
  let shader = assemble();
  let module = naga::front::wgsl::parse_str( &shader )
  .unwrap_or_else( | error | panic!( "assembled WGSL does not parse :\n{}", error.emit_to_string( &shader ) ) );
  naga::valid::Validator::new( naga::valid::ValidationFlags::all(), naga::valid::Capabilities::default() )
  .validate( &module )
  .unwrap_or_else( | error | panic!( "assembled WGSL does not validate : {error:?}" ) );
}

#[ test ]
fn assembled_shader_orders_dependencies_before_dependents()
{
  let shader = code_only( &assemble() );
  let hash21 = shader.find( "fn hash21(" ).expect( "hash21 must be declared" );
  let value_noise = shader.find( "fn value_noise(" ).expect( "value_noise must be declared" );
  let fbm3 = shader.find( "fn fbm3(" ).expect( "fbm3 must be declared" );
  let fs_main = shader.find( "fn fs_main(" ).expect( "fs_main must be declared" );
  assert!( hash21 < value_noise, "hash21 must precede value_noise, its dependent" );
  assert!( value_noise < fbm3, "value_noise must precede fbm3, its dependent" );
  assert!( fbm3 < fs_main, "every chunk must precede the fragment body that consumes the stack" );
}

// Mechanizes what shader/scene_fragment.wgsl's header used to call "kept
// in sync by hand": the WGSL scene constants and fixed-size uniform-array
// lengths must equal `scene.rs`'s canonical `*_COUNT` constants, which
// `SceneConfig::load()` in turn asserts against `scene.rhai`'s list
// lengths — closing the script → Rust → WGSL chain end to end.
#[ test ]
fn wgsl_scene_constants_match_scene_rs()
{
  let fragment_wgsl = FRAGMENT_WGSL;
  let counts =
  [
    ( "NEBULA_BAND_COUNT", scene::NEBULA_BAND_COUNT ),
    ( "STAR_LAYER_COUNT", scene::STAR_LAYER_COUNT ),
    ( "ORBIT_RING_COUNT", scene::ORBIT_RING_COUNT ),
    ( "NODE_COUNT", scene::NODE_COUNT ),
  ];
  for ( name, count ) in counts
  {
    let declaration = format!( "const {name} : u32 = {count}u;" );
    assert!
    (
      fragment_wgsl.contains( &declaration ),
      "shader/scene_fragment.wgsl must declare `{declaration}`, matching scene::{name}"
    );
  }

  let array_fields =
  [
    ( "nebula_colors", scene::NEBULA_BAND_COUNT ),
    ( "nebula_params", scene::NEBULA_BAND_COUNT ),
    ( "star_colors", scene::STAR_LAYER_COUNT ),
    ( "star_params", scene::STAR_LAYER_COUNT ),
    ( "ring_colors", scene::ORBIT_RING_COUNT ),
    ( "ring_params", scene::ORBIT_RING_COUNT ),
    ( "node_colors", scene::NODE_COUNT ),
    ( "node_params", scene::NODE_COUNT ),
  ];
  for ( field, count ) in array_fields
  {
    let declaration = format!( "{field} : array< vec4f, {count} >" );
    assert!
    (
      fragment_wgsl.contains( &declaration ),
      "shader/scene_fragment.wgsl's Uniforms must declare `{declaration}`, matching its scene.rs count"
    );
  }
}
