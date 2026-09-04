//! Build-time WGSL→GLSL ES 300 translation for `src/triangle.wgsl`'s WebGL
//! override pair, via gpu_hal's own `webgl_build` kit
//! ( `gpu_hal::webgl_build::wgsl_to_webgl_glsl`, the `webgl-glsl-build`
//! build-dependency feature ) instead of hand-porting the GLSL. Runs on the
//! host unconditionally — naga never becomes a wasm32 runtime dependency of
//! the built artifact, and generating the pair for the native stub build
//! ( which never reads it ) costs nothing worth gating.

fn main()
{
  println!( "cargo:rerun-if-changed=build.rs" );
  println!( "cargo:rerun-if-changed=src/triangle.wgsl" );

  let out_dir = std::env::var( "OUT_DIR" ).expect( "OUT_DIR not set" );
  let wgsl = include_str!( "src/triangle.wgsl" );

  let source = gpu_hal::webgl_build::wgsl_to_webgl_glsl( wgsl, "vs_main", "fs_main" )
  .unwrap_or_else( | e | panic!( "triangle_browser build.rs: triangle.wgsl :: {e}" ) );

  std::fs::write( format!( "{out_dir}/triangle.vert.glsl" ), source.vertex )
  .unwrap_or_else( | e | panic!( "triangle_browser build.rs: failed writing triangle.vert.glsl :: {e}" ) );
  std::fs::write( format!( "{out_dir}/triangle.frag.glsl" ), source.fragment )
  .unwrap_or_else( | e | panic!( "triangle_browser build.rs: failed writing triangle.frag.glsl :: {e}" ) );
}
