//! Structural shader-compilation tests for `KHR_materials_clearcoat` / `KHR_materials_anisotropy`.
//!
//! These compile the real `main.vert` / `main.frag` sources, assembled by the same
//! `material::shader_sources` `renderer.rs` compiles from ( reachable under `test_internals` ),
//! in a headless WebGL2 context, for every `#define` combination the new extension code
//! introduces. In the cases with up to five textures each texture samples its own UV set, so
//! every `v<Name>Uv` macro resolves to a distinct varying; `main.frag` declares five UV sets, so
//! the twelve-texture case shares them out. They do not verify pixel-level correctness
//! (that still relies on visual inspection of the `gltf_viewer` example, matching
//! `pmrem_tests.rs`'s philosophy) — they catch GLSL syntax/type errors that only surface at
//! runtime shader-compile time.

#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  // Browser, not Node: without it the binary runs under Node, where `web_sys::window()` is
  // `None` and every test fails at runtime with `CanvasRetrievingError("Failed to get window")`.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::{ material::{ internal::shader_sources, PbrMaterial }, Texture, TextureInfo };

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// Texture contents are irrelevant to a shader-compile test — only its presence flips on the
  /// `USE_*_TEXTURE` defines and the corresponding `sampler2D` uniform. `uv_position` is the UV
  /// set ( 0 to 4 ) the texture samples.
  fn dummy_texture_info( uv_position : u32 ) -> TextureInfo
  {
    TextureInfo { texture : Rc::new( RefCell::new( Texture::new() ) ), uv_position }
  }

  /// Compiles `material`'s shaders from the sources `renderer.rs` compiles ( `shader_sources` ),
  /// and panics with the GL error on failure.
  fn assert_compiles( gl : &GL, material : &PbrMaterial, with_ibl : bool, label : &str )
  {
    let ( vs_src, fs_src ) = shader_sources( material, with_ibl );

    gl::ProgramFromSources::new( &vs_src, &fs_src )
    .compile_and_link( gl )
    .unwrap_or_else( | e | panic!( "{label} failed to compile/link: {e:?}" ) );
  }

  /// The coat variant from factors alone: no coat texture and no tangent frame.
  #[ wasm_bindgen_test( async ) ]
  async fn clearcoat_factor_only_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.clearcoat_factor_set( Some( 1.0 ) );
    material.clearcoat_roughness_factor_set( Some( 0.2 ) );
    assert_compiles( &gl, &material, false, "clearcoat factor-only" );
  }

  /// The coat with its three textures; the coat normal map builds the derivative tangent frame
  /// from its own UV set.
  #[ wasm_bindgen_test( async ) ]
  async fn clearcoat_with_all_textures_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.clearcoat_factor_set( Some( 1.0 ) );
    material.clearcoat_texture_set( Some( dummy_texture_info( 1 ) ) );
    material.clearcoat_roughness_texture_set( Some( dummy_texture_info( 2 ) ) );
    material.clearcoat_normal_texture_set( Some( dummy_texture_info( 3 ) ) );
    assert_compiles( &gl, &material, false, "clearcoat with all textures (derivative TBN fallback)" );
  }

  /// Anisotropy without a texture: the default direction in the derivative frame from UV set 0.
  #[ wasm_bindgen_test( async ) ]
  async fn anisotropy_strength_only_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.anisotropy_strength_set( Some( 0.8 ) );
    assert_compiles( &gl, &material, false, "anisotropy strength-only (derivative TBN fallback)" );
  }

  /// Anisotropy with a direction / strength texture, whose UV set the derivative frame follows.
  #[ wasm_bindgen_test( async ) ]
  async fn anisotropy_with_texture_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.anisotropy_strength_set( Some( 0.8 ) );
    material.anisotropy_texture_set( Some( dummy_texture_info( 4 ) ) );
    assert_compiles( &gl, &material, false, "anisotropy with texture" );
  }

  /// Base normal map without vertex tangents: compiles the branch that builds the derivative
  /// frame from the normal texture's UV set ( `vNormalUv` ), which the clearcoat / anisotropy
  /// cases above never reach. Which UV set the frame follows is checked by pixel readback in
  /// `pbr_shading_readback_test.rs`.
  #[ wasm_bindgen_test( async ) ]
  async fn normal_map_and_clearcoat_normal_without_tangents_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.normal_texture_set( Some( dummy_texture_info( 1 ) ) );
    material.clearcoat_factor_set( Some( 1.0 ) );
    material.clearcoat_normal_texture_set( Some( dummy_texture_info( 2 ) ) );
    assert_compiles( &gl, &material, false, "base + clearcoat normal maps, derivative TBN from vNormalUv" );
  }

  /// With vertex tangents ( `USE_TANGENTS` ), anisotropy and the base normal map share the
  /// vertex-tangent frame instead of the derivative one.
  #[ wasm_bindgen_test( async ) ]
  async fn anisotropy_with_real_tangents_and_normal_map_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.anisotropy_strength_set( Some( 0.5 ) );
    material.normal_texture_set( Some( dummy_texture_info( 1 ) ) );
    // Mirrors what the gltf loader does when a TANGENT attribute is present, exercising the
    // real-tangent TBN branch (shared between normal mapping and anisotropy) instead of the
    // screen-space-derivative fallback.
    material.define_add( "USE_TANGENTS", "" );
    assert_compiles( &gl, &material, false, "anisotropy + base normal map sharing a real-tangent TBN" );
  }

  /// Both layers with specular, occlusion and IBL, which adds the coat's environment sample and
  /// its specular occlusion.
  #[ wasm_bindgen_test( async ) ]
  async fn clearcoat_and_anisotropy_combined_with_ibl_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.clearcoat_factor_set( Some( 1.0 ) );
    material.clearcoat_normal_texture_set( Some( dummy_texture_info( 1 ) ) );
    material.anisotropy_strength_set( Some( 0.8 ) );
    material.anisotropy_texture_set( Some( dummy_texture_info( 2 ) ) );
    material.specular_factor_set( Some( 0.5 ) ); // also exercised alongside the existing KHR_materials_specular path
    material.occlusion_texture_set( Some( dummy_texture_info( 3 ) ) ); // coat IBL specular occlusion branch
    assert_compiles( &gl, &material, true, "clearcoat + anisotropy + specular + occlusion + IBL" );
  }

  /// Every one of the twelve material textures, spread over the five UV sets `main.frag` declares
  /// ( `vUv_0` to `vUv_4` ), with both layers on and IBL: the largest program the material can
  /// produce, 15 fragment samplers, must compile and link within WebGL2's guaranteed 16.
  #[ wasm_bindgen_test( async ) ]
  async fn every_texture_with_both_layers_and_ibl_compiles()
  {
    let gl = gl_init();
    let mut material = PbrMaterial::new( &gl );
    material.clearcoat_factor_set( Some( 1.0 ) );
    material.anisotropy_strength_set( Some( 0.8 ) );
    let setters : [ fn( &mut PbrMaterial, Option< TextureInfo > ); 12 ] =
    [
      PbrMaterial::metallic_roughness_texture_set,
      PbrMaterial::base_color_texture_set,
      PbrMaterial::normal_texture_set,
      PbrMaterial::occlusion_texture_set,
      PbrMaterial::emissive_texture_set,
      PbrMaterial::specular_texture_set,
      PbrMaterial::specular_color_texture_set,
      PbrMaterial::light_map_set,
      PbrMaterial::clearcoat_texture_set,
      PbrMaterial::clearcoat_roughness_texture_set,
      PbrMaterial::clearcoat_normal_texture_set,
      PbrMaterial::anisotropy_texture_set,
    ];
    for ( i, set ) in ( 0_u32.. ).zip( setters )
    {
      set( &mut material, Some( dummy_texture_info( i % 5 ) ) );
    }
    assert_compiles( &gl, &material, true, "all twelve material textures + clearcoat + anisotropy + IBL" );
  }
}
