//! Offline validation of the WGSL shaders behind the browser-only
//! `renderer::webgpu` path.
//!
//! WGSL compiles only inside a browser at runtime, so without this test a
//! syntax or type defect in these sources is invisible to `cargo` — it would
//! surface first as a black canvas (or a console panic) in a live session.
//! Parsing and validating with `naga` — the same front end wgpu itself uses —
//! catches that whole defect class on the native target, in an ordinary test
//! run, with no browser involved.
//!
//! Scope: the WGSL sources only. Their GLSL ES 3.00 twins
//! (`*.vert.glsl` / `*.frag.glsl`) are outside naga's reach — its `glsl-in`
//! front end parses desktop GLSL, not ES profiles — and need the Khronos
//! reference validator (`glslangValidator`) instead.
#![ cfg( not( target_arch = "wasm32" ) ) ]

/// Parses `source` as WGSL and runs naga's full IR validation over it,
/// panicking with a span-annotated report on any defect. `name` labels the
/// failure so the offending file is identifiable from the assertion alone.
fn validate_wgsl( name : &str, source : &str )
{
  let module = match naga::front::wgsl::parse_str( source )
  {
    Ok( module ) => module,
    Err( error ) => panic!( "{name} failed to parse:\n{}", error.emit_to_string( source ) ),
  };
  // Default capabilities approximate the base WebGPU feature set — exactly
  // what a browser guarantees without any extension negotiation.
  let mut validator = naga::valid::Validator::new
  (
    naga::valid::ValidationFlags::all(),
    naga::valid::Capabilities::default(),
  );
  if let Err( error ) = validator.validate( &module )
  {
    panic!( "{name} failed validation:\n{}", error.emit_to_string( source ) );
  }
}

#[ test ]
fn main_wgsl_parses_and_validates()
{
  validate_wgsl( "main.wgsl", include_str!( "../src/webgpu/shaders/main.wgsl" ) );
}

#[ test ]
fn tonemap_wgsl_parses_and_validates()
{
  validate_wgsl( "tonemap.wgsl", include_str!( "../src/webgpu/shaders/tonemap.wgsl" ) );
}
