//! Structural tests for the PMREM IBL generator ( `renderer::webgl::loaders::pmrem` ).
//!
//! These exercise the real GPU pipeline in a headless WebGL2 context: they do not verify
//! pixel-level correctness ( that still relies on visual inspection of the `gltf_viewer`
//! example ), but they catch signature regressions, panics, incomplete-framebuffer failures
//! and missing output textures without a human in the loop.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::loaders::pmrem;

  /// Creates a headless WebGL2 context with the float-render-target extension PMREM needs.
  async fn init_gl() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

    // PMREM renders into RGBA16F attachments, which are only color-renderable with this
    // extension; without it the off-screen FBO is incomplete and `generate` returns an error.
    gl.get_extension( "EXT_color_buffer_float" )
      .expect( "get_extension call should not throw" )
      .expect( "EXT_color_buffer_float must be available in the test environment" );

    gl
  }

  /// Minimal equirectangular source. Contents are irrelevant to a structural test, so the
  /// 4x2 RGBA8 storage is left uninitialized; `RGBA8` + `LINEAR` is filterable everywhere and
  /// avoids depending on `OES_texture_float_linear`.
  fn make_equirect( gl : &GL ) -> gl::web_sys::WebGlTexture
  {
    let texture = gl.create_texture().unwrap();
    gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA8, 4, 2 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
    texture
  }

  /// `generate` succeeds on a valid context and returns all three IBL textures.
  #[ wasm_bindgen_test( async ) ]
  async fn generate_returns_all_textures()
  {
    let gl = init_gl().await;
    let equirect = make_equirect( &gl );

    let ibl = pmrem::generate( &gl, &equirect, 64 ).expect( "PMREM generate should succeed" );

    assert!( ibl.diffuse_texture.is_some(), "diffuse irradiance texture missing" );
    assert!( ibl.specular_1_texture.is_some(), "prefiltered specular texture missing" );
    assert!( ibl.specular_2_texture.is_some(), "BRDF LUT texture missing" );
  }

  /// A single-mip cubemap ( `resolution = 1`, so `num_mips = 1` ) must not divide by zero in
  /// the prefilter roughness computation and must still produce a full IBL set.
  #[ wasm_bindgen_test( async ) ]
  async fn generate_single_mip_resolution()
  {
    let gl = init_gl().await;
    let equirect = make_equirect( &gl );

    let ibl = pmrem::generate( &gl, &equirect, 1 ).expect( "PMREM generate should succeed for a 1x1 cubemap" );

    assert!( ibl.diffuse_texture.is_some() );
    assert!( ibl.specular_1_texture.is_some() );
    assert!( ibl.specular_2_texture.is_some() );
  }

  /// Non-power-of-two resolution is valid in WebGL2 and must produce a full IBL set.
  #[ wasm_bindgen_test( async ) ]
  async fn generate_non_power_of_two_resolution()
  {
    let gl = init_gl().await;
    let equirect = make_equirect( &gl );

    let ibl = pmrem::generate( &gl, &equirect, 96 ).expect( "PMREM generate should succeed for an NPOT cubemap" );

    assert!( ibl.diffuse_texture.is_some() );
    assert!( ibl.specular_1_texture.is_some() );
    assert!( ibl.specular_2_texture.is_some() );
  }
}
