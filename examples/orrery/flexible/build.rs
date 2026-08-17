//! Build-time WGSL→GLSL ES 300 translation for the `webgl` feature only.
//! gpu_hal's WebGPU/native/Vulkan backends consume the shared scene WGSL
//! ( `orrery_webgpu::shader_source::assemble()` ) directly; WebGL needs
//! hand-supplied GLSL ( `Device::shader_module_create` ), so this script
//! runs that same WGSL through gpu_hal's own `webgl_build` kit
//! ( `gpu_hal::webgl_build::wgsl_to_webgl_glsl`, the `webgl-glsl-build`
//! build-dependency feature ) once, at build time, instead of hand-porting
//! the ~300-line procedural fragment shader. This runs on the host
//! regardless of target — naga never becomes a wasm32 runtime dependency of
//! the built artifact.

fn main()
{
  println!( "cargo:rerun-if-changed=build.rs" );

  #[ cfg( feature = "webgl" ) ]
  webgl_shaders_generate();
}

#[ cfg( feature = "webgl" ) ]
fn webgl_shaders_generate()
{
  let wgsl = orrery_webgpu::shader_source::assemble();

  let source = gpu_hal::webgl_build::wgsl_to_webgl_glsl( &wgsl, "vs_main", "fs_main" )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: {e}" ) );

  let out_dir = std::env::var( "OUT_DIR" ).expect( "OUT_DIR not set" );
  std::fs::write( format!( "{out_dir}/scene_vertex.glsl" ), source.vertex )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: failed writing scene_vertex.glsl :: {e}" ) );
  std::fs::write( format!( "{out_dir}/scene_fragment.glsl" ), source.fragment )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: failed writing scene_fragment.glsl :: {e}" ) );
}
