//! Build-time WGSL→GLSL ES 300 translation for the `webgpu` feature's WebGL
//! override pair. `renderer.rs` consumes canonical WGSL directly for the
//! WebGPU/native/Vulkan backends; WebGL needs hand-supplied GLSL
//! ( `Device::shader_module_create` ), so this script runs each shader's
//! WGSL through gpu_hal's own `webgl_build` kit
//! ( `gpu_hal::webgl_build::wgsl_to_webgl_glsl`, the `webgl-glsl-build`
//! build-dependency feature ) once, at build time, instead of hand-porting
//! the GLSL. Runs on the host regardless of target — naga never becomes a
//! wasm32 runtime dependency of the built artifact.

fn main()
{
  println!( "cargo:rerun-if-changed=build.rs" );
  println!( "cargo:rerun-if-changed=src/webgpu/shaders/main.wgsl" );
  println!( "cargo:rerun-if-changed=src/webgpu/shaders/tonemap.wgsl" );

  #[ cfg( feature = "webgpu" ) ]
  webgl_shaders_generate();
}

#[ cfg( feature = "webgpu" ) ]
fn webgl_shaders_generate()
{
  let out_dir = std::env::var( "OUT_DIR" ).expect( "OUT_DIR not set" );

  shader_generate( &out_dir, "main", include_str!( "src/webgpu/shaders/main.wgsl" ) );
  shader_generate( &out_dir, "tonemap", include_str!( "src/webgpu/shaders/tonemap.wgsl" ) );
}

/// Translates `wgsl` and writes `{name}.vert.glsl` / `{name}.frag.glsl` into
/// `out_dir`, matching the `include_str!` paths `renderer.rs` expects.
#[ cfg( feature = "webgpu" ) ]
fn shader_generate( out_dir : &str, name : &str, wgsl : &str )
{
  let source = gpu_hal::webgl_build::wgsl_to_webgl_glsl( wgsl, "vs_main", "fs_main" )
  .unwrap_or_else( | e | panic!( "renderer build.rs: {name}.wgsl :: {e}" ) );

  std::fs::write( format!( "{out_dir}/{name}.vert.glsl" ), source.vertex )
  .unwrap_or_else( | e | panic!( "renderer build.rs: failed writing {name}.vert.glsl :: {e}" ) );
  std::fs::write( format!( "{out_dir}/{name}.frag.glsl" ), source.fragment )
  .unwrap_or_else( | e | panic!( "renderer build.rs: failed writing {name}.frag.glsl :: {e}" ) );
}
