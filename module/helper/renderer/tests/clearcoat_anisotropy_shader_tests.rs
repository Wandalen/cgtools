//! Structural shader-compilation tests for `KHR_materials_clearcoat` / `KHR_materials_anisotropy`.
//!
//! These compile the real `main.vert` / `main.frag` sources (via the same `ProgramFromSources`
//! path `renderer.rs` uses at draw time) in a headless WebGL2 context, for every `#define`
//! combination the new extension code introduces. They do not verify pixel-level correctness
//! (that still relies on visual inspection of the `gltf_viewer` example, matching
//! `pmrem_tests.rs`'s philosophy) — they catch GLSL syntax/type errors that only surface at
//! runtime shader-compile time.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::{ material::PbrMaterial, Material, Texture, TextureInfo };

  async fn init_gl() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// Texture contents are irrelevant to a shader-compile test — only its presence flips on the
  /// `USE_*_TEXTURE` defines and the corresponding `sampler2D` uniform.
  fn dummy_texture_info() -> TextureInfo
  {
    TextureInfo { texture : Rc::new( RefCell::new( Texture::new() ) ), uv_position : 0 }
  }

  /// Compiles `material`'s shaders exactly the way `renderer.rs` does (same `#version` header,
  /// same per-stage defines, same optional `USE_IBL` injection), and panics with the GL error on
  /// failure.
  fn assert_compiles( gl : &GL, material : &PbrMaterial, with_ibl : bool, label : &str )
  {
    let ibl_define = if with_ibl { "#define USE_IBL\n" } else { "" };
    let vs_src = format!( "#version 300 es\n{}\n{}", material.vertex_defines_str(), material.vertex_shader() );
    let fs_src = format!( "#version 300 es\n{}\n{}\n{}", material.fragment_defines_str(), ibl_define, material.fragment_shader() );

    gl::ProgramFromSources::new( &vs_src, &fs_src )
    .compile_and_link( gl )
    .unwrap_or_else( | e | panic!( "{label} failed to compile/link: {e:?}" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn clearcoat_factor_only_compiles()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_clearcoat_factor( Some( 1.0 ) );
    material.set_clearcoat_roughness_factor( Some( 0.2 ) );
    assert_compiles( &gl, &material, false, "clearcoat factor-only" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn clearcoat_with_all_textures_compiles()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_clearcoat_factor( Some( 1.0 ) );
    material.set_clearcoat_texture( Some( dummy_texture_info() ) );
    material.set_clearcoat_roughness_texture( Some( dummy_texture_info() ) );
    material.set_clearcoat_normal_texture( Some( dummy_texture_info() ) );
    assert_compiles( &gl, &material, false, "clearcoat with all textures (derivative TBN fallback)" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn anisotropy_strength_only_compiles()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_anisotropy_strength( Some( 0.8 ) );
    assert_compiles( &gl, &material, false, "anisotropy strength-only (derivative TBN fallback)" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn anisotropy_with_texture_compiles()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_anisotropy_strength( Some( 0.8 ) );
    material.set_anisotropy_texture( Some( dummy_texture_info() ) );
    assert_compiles( &gl, &material, false, "anisotropy with texture" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn anisotropy_with_real_tangents_and_normal_map_compiles()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_anisotropy_strength( Some( 0.5 ) );
    material.set_normal_texture( Some( dummy_texture_info() ) );
    // Mirrors what the gltf loader does when a TANGENT attribute is present, exercising the
    // real-tangent TBN branch (shared between normal mapping and anisotropy) instead of the
    // screen-space-derivative fallback.
    material.add_define( "USE_TANGENTS", "" );
    assert_compiles( &gl, &material, false, "anisotropy + base normal map sharing a real-tangent TBN" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn clearcoat_and_anisotropy_combined_with_ibl_compiles()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_clearcoat_factor( Some( 1.0 ) );
    material.set_clearcoat_normal_texture( Some( dummy_texture_info() ) );
    material.set_anisotropy_strength( Some( 0.8 ) );
    material.set_anisotropy_texture( Some( dummy_texture_info() ) );
    material.set_specular_factor( Some( 0.5 ) ); // also exercised alongside the existing KHR_materials_specular path
    assert_compiles( &gl, &material, true, "clearcoat + anisotropy + specular + IBL, worst-case combo" );
  }
}
