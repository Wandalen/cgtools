//! Structural browser tests for `UnrealBloomPass` -- the only post-processing pass in this
//! layer with zero prior test coverage ( no test citation anywhere in the codebase, no
//! tracking task ).
//!
//! Matches `pmrem_tests.rs`/`fbo_pass_cycle_test.rs`'s tier for this crate's multi-pass WebGL
//! post-processing code: exercises the real GPU pipeline ( 5 mip levels of ping-pong
//! horizontal/vertical Gaussian blur, then a composite pass ) in a headless WebGL2 context and
//! catches signature regressions, panics, and incomplete-framebuffer failures -- but does not
//! assert pixel-level bloom correctness, which stays delegated to visual inspection per this
//! crate's existing convention for post-processing passes ( see `wide_outline.rs`,
//! `pmrem_tests.rs` ).
//!
//! Unlike `WideOutlinePass`, `UnrealBloomPass` owns no framebuffer of its own -- it has no
//! `bind()` method and no framebuffer field, so `render` relies entirely on the caller having
//! already bound one ( see `renderer.rs`'s `composite`: `swap.bind( gl )` immediately precedes
//! `bloom.render( gl, swap.input_get(), swap.output_get() )` ). These tests reproduce that exact
//! real-usage sequence via `SwapFramebuffer` rather than a synthetic standalone binding, so a
//! regression back to an unbound/default-framebuffer attach ( which WebGL rejects with
//! `INVALID_OPERATION` on `framebufferTexture2D`, not a panic ) would be exercised the same way
//! it would be in production, instead of the test silently rendering into nothing.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::post_processing::{ Pass, SwapFramebuffer, UnrealBloomPass };

  /// Creates a headless WebGL2 context with the float-render-target extension both
  /// `SwapFramebuffer` ( hardcoded `RGBA16F` output texture, see `pass.rs` ) and
  /// `UnrealBloomPass`'s mip blur targets need to be color-renderable -- production
  /// ( `renderer.rs` ) always constructs `UnrealBloomPass` with `gl::RGBA16F`, so this mirrors
  /// the real configuration rather than a synthetic `RGBA8` one. Synchronous ( matching
  /// `fbo_pass_cycle_test.rs`'s `gl_init`, unlike `pmrem_tests.rs`'s ) -- nothing here awaits.
  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default();
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

    gl.get_extension( "EXT_color_buffer_float" )
      .expect( "get_extension call should not throw" )
      .expect( "EXT_color_buffer_float must be available in the test environment" );

    gl
  }

  /// A minimal RGBA8 source texture. Contents are irrelevant to this structural test; it is only
  /// ever sampled by the first horizontal blur pass, never attached as a framebuffer target, so
  /// it doesn't need to be a color-renderable format itself.
  fn texture_make( gl : &GL, width : i32, height : i32 ) -> gl::web_sys::WebGlTexture
  {
    let texture = gl.create_texture().unwrap();
    gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
    gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width, height );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
    texture
  }

  /// `UnrealBloomPass::new` + `Pass::render` must complete without panicking or returning an
  /// error against a real headless WebGL2 context, using the exact bind sequence `renderer.rs`'s
  /// `composite` uses ( `SwapFramebuffer::bind` immediately before `render` ) -- this exercises
  /// allocation and use of all 5 mip levels' horizontal/vertical Gaussian blur ping-pong plus the
  /// final composite pass. `UnrealBloomPass` had zero test coverage prior to this.
  #[ wasm_bindgen_test( async ) ]
  async fn render_completes_on_a_solid_color_source()
  {
    let gl = gl_init();
    let width = 64;
    let height = 64;

    let pass = UnrealBloomPass::new( &gl, width, height, gl::RGBA16F )
    .expect( "UnrealBloomPass construction should succeed on a valid context" );

    let mut swap = SwapFramebuffer::new( &gl, width, height );
    swap.bind( &gl );
    swap.input_set( Some( texture_make( &gl, width as i32, height as i32 ) ) );

    let result = pass.render( &gl, swap.input_get(), swap.output_get() );

    assert!
    (
      result.is_ok(),
      "UnrealBloomPass::render should succeed on a solid-color source -- got {:?}", result.err()
    );
  }

  /// Rendering once with the default bloom parameters, then again after mutating both public
  /// setters ( `bloom_radius_set`, `bloom_strength_set` ), must succeed both times -- exercises
  /// the per-frame uniform re-upload path, not only the values set at construction, mirroring
  /// `wide_outline.rs`'s pre-/post-mutation coverage for `outline_thickness_set`. Also asserts
  /// the getters round-trip correctly, including `bloom_radius_set`'s documented `[ 0.0, 1.0 ]`
  /// clamp.
  #[ wasm_bindgen_test( async ) ]
  async fn render_completes_after_bloom_parameters_are_changed()
  {
    let gl = gl_init();
    let width = 64;
    let height = 64;

    let mut pass = UnrealBloomPass::new( &gl, width, height, gl::RGBA16F )
    .expect( "UnrealBloomPass construction should succeed on a valid context" );

    let mut swap = SwapFramebuffer::new( &gl, width, height );
    swap.bind( &gl );
    swap.input_set( Some( texture_make( &gl, width as i32, height as i32 ) ) );

    let result = pass.render( &gl, swap.input_get(), swap.output_get() );
    assert!
    (
      result.is_ok(),
      "UnrealBloomPass::render should succeed with default bloom parameters -- got {:?}", result.err()
    );

    // Out-of-range radius must clamp to 1.0, matching `bloom_radius_set`'s documented clamp.
    pass.bloom_radius_set( 5.0 );
    pass.bloom_strength_set( 3.0 );
    assert!
    (
      ( pass.bloom_radius() - 1.0 ).abs() < f32::EPSILON,
      "bloom_radius_set( 5.0 ) should clamp to 1.0, got {}", pass.bloom_radius()
    );
    assert!
    (
      ( pass.bloom_strength() - 3.0 ).abs() < f32::EPSILON,
      "bloom_strength_set( 3.0 ) should round-trip through bloom_strength(), got {}", pass.bloom_strength()
    );

    // Re-bind before the second render, mirroring `renderer.rs`'s `composite`, which calls
    // `swap.bind( gl )` immediately before every `bloom.render(...)` call, once per frame.
    swap.bind( &gl );
    let result = pass.render( &gl, swap.input_get(), swap.output_get() );
    assert!
    (
      result.is_ok(),
      "UnrealBloomPass::render should succeed after bloom parameters are changed -- got {:?}", result.err()
    );
  }
}
