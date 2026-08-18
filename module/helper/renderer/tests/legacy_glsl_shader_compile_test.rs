//! Compiles every shipped legacy GLSL ES 3.00 shader (`*.vert` / `*.frag`
//! under `src/webgl/shaders/`) through a real headless WebGL2 context's
//! `compileShader` / `COMPILE_STATUS`, using the exact same production
//! compile path `gl::ProgramFromSources`/`gl::ShaderSource` use elsewhere in
//! this crate (see `pbr.rs`, `renderer.rs`, `unreal_bloom.rs`).
//!
//! `shader_validation_tests.rs` documents why this half of the surface is
//! out of *its* scope: naga's `front::glsl` targets GLSL 440+/Vulkan
//! semantics and rejects these GLSL-ES-idiom sources outright. A live
//! WebGL2 context's own compiler is the correct validator for GLSL ES
//! 3.00, so that's what this file drives, one `#[ wasm_bindgen_test ]` per
//! shader stage so a failing file is identifiable from the test name alone.
//!
//! Structural only: a syntax/type/binding defect in a shipped shader
//! surfaces here as a `COMPILE_STATUS` failure with the driver's own info
//! log; this makes no claim about a shader's rendered output.
//!
//! 6 of the 28 files are templates, not complete standalone sources: they
//! have no `#version` line of their own and/or reference `#define`d
//! constants (`NUM_MIPS`, `KERNEL_RADIUS`) that their real call site
//! injects as a preamble before compiling. For those, this file builds the
//! same preamble the production call site builds (cited per test below)
//! instead of `include_str!`-ing the raw file alone — anything else would
//! be testing a source nothing in this crate ever actually compiles.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::post_processing::ALL;

  /// Headless WebGL2 context. No extensions needed: shader compilation
  /// alone never touches a framebuffer or texture.
  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas( &canvas ).unwrap()
  }

  /// Compiles `source` as `shader_type` via the crate's real shader-compile
  /// path and panics with the driver's own info log (surfaced through
  /// `ShaderSource::compile`'s `Err`) on failure.
  fn glsl_validate( gl : &GL, name : &str, shader_type : u32, source : &str )
  {
    let result = gl::ShaderSource::former()
    .shader_type( shader_type )
    .source( source )
    .shader_name( name )
    .compile( gl );

    assert!( result.is_ok(), "{name} failed to compile:\n{}", result.err().unwrap() );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn bake_frag_compiles()
  {
    glsl_validate( &gl_init(), "bake.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/bake.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn bake_vert_compiles()
  {
    glsl_validate( &gl_init(), "bake.vert", GL::VERTEX_SHADER, include_str!( "../src/webgl/shaders/bake.vert" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn big_triangle_vert_compiles()
  {
    glsl_validate( &gl_init(), "big_triangle.vert", GL::VERTEX_SHADER, include_str!( "../src/webgl/shaders/big_triangle.vert" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn composite_frag_compiles()
  {
    glsl_validate( &gl_init(), "composite.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/composite.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn copy_frag_compiles()
  {
    glsl_validate( &gl_init(), "copy.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/copy.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn depth_vert_compiles()
  {
    glsl_validate( &gl_init(), "depth.vert", GL::VERTEX_SHADER, include_str!( "../src/webgl/shaders/depth.vert" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn empty_frag_compiles()
  {
    glsl_validate( &gl_init(), "empty.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/empty.frag" ) );
  }

  /// `KERNEL_RADIUS` is a `#define` the shader expects as a preamble, not a
  /// constant it declares — production ( `post_processing/unreal_bloom.rs`,
  /// `UnrealBloomPass::new` ) compiles this exact file 5 times, once per
  /// entry in its own hardcoded `let kernel_radius = [ 3, 5, 7, 9, 11 ];`.
  /// Mirror all 5 rather than picking one, since that IS the real call
  /// pattern.
  #[ wasm_bindgen_test( async ) ]
  async fn filters_gaussian_frag_compiles()
  {
    let gl = gl_init();
    let source = include_str!( "../src/webgl/shaders/filters/gaussian.frag" );
    for radius in [ 3, 5, 7, 9, 11 ]
    {
      let prefixed = format!( "#version 300 es\n#define KERNEL_RADIUS {radius}\n{source}" );
      glsl_validate( &gl, &format!( "filters/gaussian.frag (KERNEL_RADIUS={radius})" ), GL::FRAGMENT_SHADER, &prefixed );
    }
  }

  /// Neither file has a `#version` line of its own — production
  /// ( `webgl/renderer.rs`, `primitive_register` ) always prepends
  /// `#version 300 es` plus `PbrMaterial::local_defines()`'s output, which
  /// varies per material/mesh ( textures, skinning, IBL, ... ). The 3 shown
  /// here are the one combination both files require unconditionally
  /// ( `material/pbr.rs`; `main.frag` uses all 3 unguarded ); every other
  /// `USE_*` branch has a working `#ifndef`/`#else` fallback in both files,
  /// so this is the minimal structurally-valid state — a fresh
  /// `PbrMaterial::new()` with no textures/skin/morph/IBL.
  const MAIN_SHADER_DEFINES : &str = "#define MAX_POINT_LIGHTS 8\n#define MAX_DIRECT_LIGHTS 8\n#define MAX_SPOT_LIGHTS 8\n";

  #[ wasm_bindgen_test( async ) ]
  async fn main_frag_compiles()
  {
    let source = include_str!( "../src/webgl/shaders/main.frag" );
    let prefixed = format!( "#version 300 es\n{MAIN_SHADER_DEFINES}\n{source}" );
    glsl_validate( &gl_init(), "main.frag", GL::FRAGMENT_SHADER, &prefixed );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn main_vert_compiles()
  {
    let source = include_str!( "../src/webgl/shaders/main.vert" );
    let prefixed = format!( "#version 300 es\n{MAIN_SHADER_DEFINES}\n{source}" );
    glsl_validate( &gl_init(), "main.vert", GL::VERTEX_SHADER, &prefixed );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn pmrem_brdf_integration_frag_compiles()
  {
    glsl_validate( &gl_init(), "pmrem/brdf_integration.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/pmrem/brdf_integration.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn pmrem_equirect_to_cube_frag_compiles()
  {
    glsl_validate( &gl_init(), "pmrem/equirect_to_cube.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/pmrem/equirect_to_cube.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn pmrem_irradiance_convolution_frag_compiles()
  {
    glsl_validate( &gl_init(), "pmrem/irradiance_convolution.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/pmrem/irradiance_convolution.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn pmrem_prefilter_specular_frag_compiles()
  {
    glsl_validate( &gl_init(), "pmrem/prefilter_specular.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/pmrem/prefilter_specular.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn post_processing_color_grading_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/color_grading.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/color_grading.frag" ) );
  }

  /// Both gbuffer shaders are `#ifdef`-gated per attachment with no
  /// `#version` line of their own — production ( `post_processing/gbuffer.rs`,
  /// `GBuffer::new` ) always prepends `#version 300 es` plus one `#define`
  /// per requested `GBufferAttachment`. Use `post_processing::ALL` ( all 7 )
  /// for maximal branch coverage, built via the same public
  /// `define_const()` `GBuffer::new`'s own private `into_defines` wraps.
  fn gbuffer_defines() -> String
  {
    let mut defines = String::new();
    for attachment in ALL
    {
      defines.push_str( "#define " );
      defines.push_str( &attachment.define_const() );
      defines.push( '\n' );
    }
    defines
  }

  #[ wasm_bindgen_test( async ) ]
  async fn post_processing_gbuffer_frag_compiles()
  {
    let source = include_str!( "../src/webgl/shaders/post_processing/gbuffer.frag" );
    let prefixed = format!( "#version 300 es\n{}\n{source}", gbuffer_defines() );
    glsl_validate( &gl_init(), "post_processing/gbuffer.frag", GL::FRAGMENT_SHADER, &prefixed );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn post_processing_gbuffer_vert_compiles()
  {
    let source = include_str!( "../src/webgl/shaders/post_processing/gbuffer.vert" );
    let prefixed = format!( "#version 300 es\n{}\n{source}", gbuffer_defines() );
    glsl_validate( &gl_init(), "post_processing/gbuffer.vert", GL::VERTEX_SHADER, &prefixed );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn outline_narrow_outline_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/outline/narrow_outline.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/outline/narrow_outline.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn outline_normal_depth_outline_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/outline/normal_depth_outline.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/outline/normal_depth_outline.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn wide_outline_jfa_init_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/outline/wide_outline/jfa_init.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/outline/wide_outline/jfa_init.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn wide_outline_jfa_step_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/outline/wide_outline/jfa_step.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/outline/wide_outline/jfa_step.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn wide_outline_outline_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/outline/wide_outline/outline.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/outline/wide_outline/outline.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn post_processing_shadow_to_color_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/shadow_to_color.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/shadow_to_color.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn post_processing_to_srgb_frag_compiles()
  {
    glsl_validate( &gl_init(), "post_processing/to_srgb.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/post_processing/to_srgb.frag" ) );
  }

  /// `NUM_MIPS` is a `#define` preamble, not a constant this file declares —
  /// production ( `post_processing/unreal_bloom.rs`, `UnrealBloomPass::new`
  /// ) always injects `#define NUM_MIPS 5` ( its own `const MIPS : usize =
  /// 5`, the only value used anywhere in this crate, since `main()` below
  /// hardcodes `bloomFactors[0..4]`/`blurTexture0..4` ).
  #[ wasm_bindgen_test( async ) ]
  async fn post_processing_unreal_bloom_frag_compiles()
  {
    let source = include_str!( "../src/webgl/shaders/post_processing/unreal_bloom.frag" );
    let prefixed = format!( "#version 300 es\n#define NUM_MIPS 5\n{source}" );
    glsl_validate( &gl_init(), "post_processing/unreal_bloom.frag", GL::FRAGMENT_SHADER, &prefixed );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn skybox_frag_compiles()
  {
    glsl_validate( &gl_init(), "skybox.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/skybox.frag" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn skybox_vert_compiles()
  {
    glsl_validate( &gl_init(), "skybox.vert", GL::VERTEX_SHADER, include_str!( "../src/webgl/shaders/skybox.vert" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn tonemapping_aces_frag_compiles()
  {
    glsl_validate( &gl_init(), "tonemapping/aces.frag", GL::FRAGMENT_SHADER, include_str!( "../src/webgl/shaders/tonemapping/aces.frag" ) );
  }
}
