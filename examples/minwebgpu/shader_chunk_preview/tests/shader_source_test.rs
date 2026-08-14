//! Tests for `shader_source`: the assembled WGSL's structural honesty
//! ( every declaration exactly once, dependencies ahead of dependents ), a
//! full native naga parse + validation, the `preview_fragment.wgsl`-vs-
//! `PREVIEW_FRAGMENT` manifest sync, and -- this crate's own addition over
//! `examples/orrery/webgpu`'s template -- that the tunable parameters
//! `shader_chunks_params::chunk_discover` finds in the local chunk's
//! manifest match the uniform fields the fragment body actually reads.

use minwebgpu_shader_chunk_preview::shader_source::{ assemble, PREVIEW_FRAGMENT };

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
  let fragment_wgsl = PREVIEW_FRAGMENT.wgsl;
  for chunk_declaration in [ "fn hash21(", "fn value_noise(", "fn fbm3(", "fn vs_main(", "struct VertexOutput" ]
  {
    assert_eq!
    (
      code_occurrences( fragment_wgsl, chunk_declaration ), 0,
      "shader/preview_fragment.wgsl must not carry its own copy of `{chunk_declaration}` — it comes from shader_chunks_core"
    );
  }

  // What makes the chunks live code rather than dead weight: `fs_main`
  // takes the vertex stage's output type and calls into the noise stack
  // twice to build a domain warp, then a third time at the warped point.
  for consumed in [ "VertexOutput", "fbm3(" ]
  {
    assert!
    (
      code_occurrences( fragment_wgsl, consumed ) > 0,
      "shader/preview_fragment.wgsl must consume `{consumed}`"
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

// The local chunk's descriptor restates its `//@` manifest as Rust data;
// this is the same no-silent-drift guarantee shader_chunks_core's own
// tests give the bundled table, applied to this crate's one local chunk.
#[ test ]
fn preview_fragment_descriptor_matches_its_manifest()
{
  let mismatches = shader_chunks_core::manifest_mismatches( &PREVIEW_FRAGMENT );
  assert!( mismatches.is_empty(), "{mismatches:#?}" );
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

// Closes the loop this example exists to demonstrate: every `//@ param:`
// line `shader_chunks_params::chunk_discover` finds in the manifest must
// name a real field of the WGSL `Params` uniform struct `fs_main` actually
// reads -- a tunable declared in the manifest but absent from the struct
// would move a UI slider that changes nothing.
#[ test ]
fn discovered_tunable_parameters_match_params_uniform_fields()
{
  let parameters = shader_chunks_params::chunk_discover( &PREVIEW_FRAGMENT );
  let names : Vec< &str > = parameters.iter().map( | p | p.name.as_str() ).collect();
  assert_eq!
  (
    names, [ "noise_scale", "warp_strength", "brightness" ],
    "preview_fragment.wgsl's //@ param: lines must declare exactly these 3 tunables, in this order"
  );

  for parameter in &parameters
  {
    assert_eq!
    (
      parameter.kind, shader_chunks_params::ParameterKind::Uniform,
      "`{}` must be declared `uniform` — main.rs writes it into the same ParamsRaw buffer every frame", parameter.name
    );
    assert_eq!
    (
      parameter.value_type, shader_chunks_params::ValueType::F32,
      "`{}` must be declared `f32` — matches ParamsRaw's field type", parameter.name
    );
    assert!
    (
      parameter.range.is_some(),
      "`{}` must carry a range — main.rs's slider needs min/max to size its control", parameter.name
    );

    let field_declaration = format!( "{} : f32", parameter.name );
    assert!
    (
      PREVIEW_FRAGMENT.wgsl.contains( &field_declaration ),
      "Params struct must declare `{field_declaration}`, matching discovered parameter `{}`", parameter.name
    );
  }
}
