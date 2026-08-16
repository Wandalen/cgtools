//! Build-time WGSL→GLSL ES 300 translation for the `webgl` feature only.
//! gpu_hal's WebGPU/native/Vulkan backends consume the shared scene WGSL
//! ( `orrery_webgpu::shader_source::assemble()` ) directly; WebGL needs
//! hand-supplied GLSL ( `Device::shader_module_create` ), so this script
//! runs that same WGSL through naga's own GLSL backend once, at build time,
//! instead of hand-porting the ~300-line procedural fragment shader. This
//! runs on the host regardless of target — naga never becomes a wasm32
//! runtime dependency of the built artifact.
//!
//! gpu_hal's WebGL uniform-block introspection ( `webgl_bindings_introspect`,
//! `module/helper/gpu_hal/src/device.rs` ) resolves a uniform block by the
//! literal name `ub_{group}_{binding}` and fails silently on a mismatch;
//! naga generates its own block name instead, so the fragment translation's
//! block is renamed to `ub_0_0` using naga's own `ReflectionInfo::uniforms`
//! map — the exact generated name, not a guessed pattern — before the GLSL
//! is written to `OUT_DIR`.

fn main()
{
  println!( "cargo:rerun-if-changed=build.rs" );

  #[ cfg( feature = "webgl" ) ]
  webgl_shaders_generate();
}

#[ cfg( feature = "webgl" ) ]
fn webgl_shaders_generate()
{
  use naga::back::glsl;

  let wgsl = orrery_webgpu::shader_source::assemble();

  let module = naga::front::wgsl::parse_str( &wgsl )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: WGSL parse failed :: {e}" ) );
  let info = naga::valid::Validator::new( naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all() )
  .validate( &module )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: WGSL validation failed :: {e}" ) );

  // WebGL2 = GLSL ES 300. `writer_flags` deliberately omits
  // `ADJUST_COORDINATE_SPACE` ( naga's `Options::default()` sets it ) :
  // gpu_hal's WebGL backend expects plain GLSL with no NDC Y-flip, the same
  // convention `triangle_browser`'s hand-written `GLSL_VERTEX` already uses.
  let options = glsl::Options
  {
    version : glsl::Version::Embedded { version : 300, is_webgl : true },
    writer_flags : glsl::WriterFlags::empty(),
    ..glsl::Options::default()
  };

  let ( vertex_glsl, _ ) = stage_translate( &module, &info, &options, naga::ShaderStage::Vertex, "vs_main" );
  let ( fragment_glsl, reflection ) = stage_translate( &module, &info, &options, naga::ShaderStage::Fragment, "fs_main" );

  let block_name = reflection.uniforms.values().next().unwrap_or_else( ||
  {
    panic!( "orrery_flexible build.rs: naga produced no uniform reflection for fs_main -- expected exactly one ( the scene Uniforms block )" )
  } );
  let fragment_glsl = fragment_glsl.replace( block_name.as_str(), "ub_0_0" );

  let out_dir = std::env::var( "OUT_DIR" ).expect( "OUT_DIR not set" );
  std::fs::write( format!( "{out_dir}/scene_vertex.glsl" ), vertex_glsl )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: failed writing scene_vertex.glsl :: {e}" ) );
  std::fs::write( format!( "{out_dir}/scene_fragment.glsl" ), fragment_glsl )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: failed writing scene_fragment.glsl :: {e}" ) );
}

#[ cfg( feature = "webgl" ) ]
fn stage_translate
(
  module : &naga::Module,
  info : &naga::valid::ModuleInfo,
  options : &naga::back::glsl::Options,
  shader_stage : naga::ShaderStage,
  entry_point : &str,
) -> ( String, naga::back::glsl::ReflectionInfo )
{
  use naga::back::glsl;
  let pipeline_options = glsl::PipelineOptions
  {
    shader_stage,
    entry_point : entry_point.to_string(),
    multiview : None,
  };
  let mut out = String::new();
  let reflection = glsl::Writer::new( &mut out, module, info, options, &pipeline_options, naga::proc::BoundsCheckPolicies::default() )
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: GLSL writer construction failed for {entry_point} :: {e}" ) )
  .write()
  .unwrap_or_else( | e | panic!( "orrery_flexible build.rs: GLSL translation failed for {entry_point} :: {e}" ) );
  ( out, reflection )
}
