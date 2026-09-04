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
//! Scope: the WGSL sources only. Their GLSL ES 3.00 twins (`*.vert` /
//! `*.frag` under `src/webgl/shaders/`) are outside naga's reach. naga does
//! ship a GLSL front end (`naga::front::glsl`, feature `glsl-in`, available
//! in the `naga = "30.0"` pin used here), but its own support table scopes
//! it to "GLSL 440+ and Vulkan semantics only" — confirmed by feeding these
//! exact shader sources through `front::glsl::Frontend::parse`: it rejects
//! the `#version 300 es` pragma outright (`InvalidVersion(300)`,
//! `InvalidProfile("es")`), doesn't recognize GLSL-ES-only builtins such as
//! `gl_VertexID` (Vulkan GLSL spells it `gl_VertexIndex`), and requires
//! explicit `layout(binding = N)` on every uniform block — none of which
//! this OpenGL-ES-idiom codebase declares. Papering over that would mean
//! validating a rewritten shader that no longer matches what actually
//! ships, so naga is not the right tool for this half of the surface.
//! `legacy_glsl_shader_compile_test.rs` covers it instead, compiling all 28
//! shipped `.vert`/`.frag` files through a real headless WebGL2 context —
//! the actual GLSL ES 3.00 compiler these sources target, not a native
//! offline stand-in.
#![ cfg( not( target_arch = "wasm32" ) ) ]

/// Parses `source` as WGSL and runs naga's full IR validation over it,
/// panicking with a span-annotated report on any defect. `name` labels the
/// failure so the offending file is identifiable from the assertion alone.
fn wgsl_validate( name : &str, source : &str )
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
  wgsl_validate( "main.wgsl", include_str!( "../src/webgpu/shaders/main.wgsl" ) );
}

#[ test ]
fn tonemap_wgsl_parses_and_validates()
{
  wgsl_validate( "tonemap.wgsl", include_str!( "../src/webgpu/shaders/tonemap.wgsl" ) );
}
