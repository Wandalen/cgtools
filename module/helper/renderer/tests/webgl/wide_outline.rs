//! Structural browser test for `WideOutlinePass` ( BUG-179: `outline_thickness` never reached
//! the shader that decides whether a background pixel is close enough to draw the outline
//! color -- it was hardcoded as `const float outlineThickness = 30.0;` in `outline.frag`,
//! completely disconnected from the constructor/setter parameter of the same name ).
//!
//! Matches `pmrem_tests.rs`'s tier for this crate's multi-pass WebGL post-processing code:
//! exercises the real GPU pipeline in a headless WebGL2 context and catches signature
//! regressions, panics, and missing/renamed uniform wiring -- but does not assert pixel-level
//! outline thickness, which stays delegated to visual inspection per this crate's existing
//! convention for this code area. Concretely, this test *would* catch a regression back to a
//! hardcoded shader constant: `outline_pass` looks up `outlineThickness`'s uniform location via
//! `.unwrap()`, which panics if the shader stops declaring that uniform.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::post_processing::{ Pass, outline::wide_outline::WideOutlinePass };

  async fn gl_init() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// A minimal RGBA8 texture. Contents are irrelevant to this structural test.
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

  /// ## Root Cause
  /// `outline.frag` declared `const float outlineThickness = 30.0;` instead of a uniform, so
  /// `WideOutlinePass::outline_thickness` ( set via the constructor or `outline_thickness_set` )
  /// never reached the pass that decides whether a background pixel is within the outline band.
  /// ## Why Not Caught
  /// No test constructed or rendered a `WideOutlinePass` prior to this bug -- the only coverage
  /// was visual inspection of the `outline`/`narrow_outline` examples, and a hardcoded-but-
  /// plausible-looking `30.0` produces a visually reasonable outline regardless of what value
  /// the caller actually requested, so nothing looked obviously wrong on inspection.
  /// ## Fix Applied
  /// Changed `outline.frag`'s `outlineThickness` to a `uniform float`, added it to
  /// `WideOutlineShader`'s location list, and uploaded `self.outline_thickness` every
  /// `outline_pass` call, matching the existing `resolution` uniform's upload pattern.
  /// ## Prevention
  /// This test constructs two independent `WideOutlinePass` instances with different
  /// `outline_thickness` values and asserts both render without error -- a regression back to a
  /// hardcoded shader constant would panic here, since `outline_pass` looks up the
  /// `outlineThickness` uniform location via `.unwrap()`, which fails if the shader doesn't
  /// declare it.
  /// ## Pitfall
  /// A struct field that's clearly threaded through the constructor and a public setter reads as
  /// "already wired up" -- but a Rust-side field controls nothing on its own; the shader itself
  /// must independently declare and consume a matching uniform, and nothing at compile time
  /// checks that the two sides agree.
  // test_kind: bug_reproducer(BUG-179)
  #[ wasm_bindgen_test( async ) ]
  async fn render_succeeds_for_two_different_outline_thicknesses()
  {
    let gl = gl_init().await;
    let width = 16;
    let height = 16;

    // Thin outline.
    let object_color_1 = texture_make( &gl, width, height );
    let source_1 = texture_make( &gl, width, height );
    let output_1 = texture_make( &gl, width, height );
    let pass_1 = WideOutlinePass::new( &gl, object_color_1, 3.0, width as u32, height as u32 )
    .expect( "WideOutlinePass construction should succeed ( thickness = 3.0 )" );
    pass_1.render( &gl, Some( source_1 ), Some( output_1 ) )
    .expect( "render should succeed ( thickness = 3.0 )" );

    // Thick outline -- a distinct, independently-constructed pass/texture set, so this doesn't
    // depend on `WebGlTexture` being cheaply re-usable across two `render` calls.
    let object_color_2 = texture_make( &gl, width, height );
    let source_2 = texture_make( &gl, width, height );
    let output_2 = texture_make( &gl, width, height );
    let mut pass_2 = WideOutlinePass::new( &gl, object_color_2, 3.0, width as u32, height as u32 )
    .expect( "WideOutlinePass construction should succeed ( thickness = 30.0, pre-mutation )" );
    // Exercises the per-frame re-upload path ( `outline_thickness_set` then re-render ), not
    // only the value passed at construction time.
    pass_2.outline_thickness_set( 30.0 );
    pass_2.render( &gl, Some( source_2 ), Some( output_2 ) )
    .expect( "render should succeed ( thickness = 30.0, post-mutation )" );
  }
}
