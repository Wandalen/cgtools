//! Structural browser test for `renderer::webgl::loaders::ibl::ibl_texture_parameters_apply`
//! ( BUG-260 ). Matches `pmrem_tests.rs`'s / `wide_outline.rs`'s tier for this crate's real-GPU
//! WebGL code: exercises actual `bind_texture`/`tex_parameteri`/`get_tex_parameter` calls in a
//! headless WebGL2 context rather than mocking them, since texture-parameter state cannot be
//! observed any other way.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::loaders::ibl;

  async fn gl_init() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// Reads back an integer-valued texture parameter for whichever texture is currently bound to
  /// `GL::TEXTURE_CUBE_MAP`.
  fn cube_map_tex_parameter( gl : &GL, pname : u32 ) -> i32
  {
    gl.get_tex_parameter( GL::TEXTURE_CUBE_MAP, pname )
    .as_f64()
    .expect( "get_tex_parameter should return a numeric value for TEXTURE_BASE_LEVEL/TEXTURE_MAX_LEVEL" )
    as i32
  }

  /// ## Root Cause
  /// `ibl::load`'s texture-parameter setup block bound 3 different textures to the single global
  /// `TEXTURE_CUBE_MAP` binding point in sequence ( `specular_1_texture`, then -- via a
  /// `TEXTURE_2D` bind for `specular_2_texture` in between -- `diffuse_texture` ), and applied
  /// the caller-supplied `mip_range` ( `TEXTURE_BASE_LEVEL`/`TEXTURE_MAX_LEVEL` ) only after the
  /// *last* `TEXTURE_CUBE_MAP` rebind, to `diffuse_texture`. `diffuse_texture` has exactly one
  /// mip level, so clamping its range is meaningless; `specular_1_texture` -- the texture
  /// actually carrying the 10-level chain `IBL::num_mips` documents -- never received the clamp
  /// at all.
  ///
  /// ## Why Not Caught
  /// No test exercised `ibl::load`'s texture-parameter wiring prior to this bug. Of the 10 real
  /// call sites across `examples/`, 9 pass `mip_range: None` ( the `if let Some(..)` block never
  /// runs, so the bug is unreachable ), and the 1 real non-`None` caller
  /// ( `examples/minwebgl/pbr_lighting/src/main.rs` ) happens to pass `Some( 0..0 )` -- which
  /// coincidentally matches `TEXTURE_BASE_LEVEL`'s own spec default of `0` closely enough
  /// ( `TEXTURE_MAX_LEVEL` written as `0` instead of the spec default `1000` still visually
  /// clamps to mip 0, the base level, indistinguishable from "no clamp" for a call that also
  /// always samples mip 0 ) that the misapplication produced no visibly wrong output there
  /// either.
  ///
  /// ## Fix Applied
  /// Extracted the filter/mip-range block into its own `pub fn ibl_texture_parameters_apply`,
  /// moving the `mip_range` application to sit immediately after `specular_1_texture`'s own
  /// filter `tex_parameteri` calls -- while `specular_1_texture` is still the texture bound to
  /// `TEXTURE_CUBE_MAP` -- instead of after the later rebind to `diffuse_texture` ( BUG-260,
  /// `loaders/ibl.rs` ).
  ///
  /// ## Prevention
  /// This test calls `ibl_texture_parameters_apply` directly against 3 freshly-created real
  /// textures with a non-degenerate `mip_range` ( `Some( 2..5 )`, deliberately distinct from both
  /// endpoints' spec defaults so a misapplication cannot hide behind a coincidental match like
  /// the one production code's only real caller had ), then reads back
  /// `TEXTURE_BASE_LEVEL`/`TEXTURE_MAX_LEVEL` via `get_tex_parameter` for each of
  /// `specular_1_texture` and `diffuse_texture` and asserts the range landed on `specular_1`
  /// while `diffuse` stayed at the WebGL2/ES3.0 spec defaults ( `0`/`1000` ).
  ///
  /// ## Pitfall
  /// WebGL's `bind_texture`/`tex_parameteri` pair operates on whichever texture is *currently*
  /// bound to the target -- any `tex_parameteri` call must stay textually adjacent to the
  /// `bind_texture` call for the texture it is meant to configure, especially once more than one
  /// texture shares the same binding point within one function. A test asserting only "the call
  /// didn't panic" or "the function returned" would never catch this class of bug -- only reading
  /// back actual GL state per-texture does.
  // test_kind: bug_reproducer(BUG-260)
  #[ wasm_bindgen_test( async ) ]
  async fn ibl_texture_parameters_apply_targets_mip_range_at_specular_1_not_diffuse()
  {
    let gl = gl_init().await;

    let specular_1 = gl.create_texture();
    let specular_2 = gl.create_texture();
    let diffuse = gl.create_texture();

    ibl::ibl_texture_parameters_apply
    (
      &gl,
      specular_1.as_ref(),
      specular_2.as_ref(),
      diffuse.as_ref(),
      Some( 2..5 )
    );

    gl.bind_texture( GL::TEXTURE_CUBE_MAP, specular_1.as_ref() );
    let specular_1_base = cube_map_tex_parameter( &gl, GL::TEXTURE_BASE_LEVEL );
    let specular_1_max = cube_map_tex_parameter( &gl, GL::TEXTURE_MAX_LEVEL );

    gl.bind_texture( GL::TEXTURE_CUBE_MAP, diffuse.as_ref() );
    let diffuse_base = cube_map_tex_parameter( &gl, GL::TEXTURE_BASE_LEVEL );
    let diffuse_max = cube_map_tex_parameter( &gl, GL::TEXTURE_MAX_LEVEL );

    gl.bind_texture( GL::TEXTURE_CUBE_MAP, None );

    assert_eq!
    (
      specular_1_base, 2,
      "mip_range.start must land on specular_1_texture ( the texture with a real multi-level chain ), got TEXTURE_BASE_LEVEL={specular_1_base}"
    );
    assert_eq!
    (
      specular_1_max, 5,
      "mip_range.end must land on specular_1_texture, got TEXTURE_MAX_LEVEL={specular_1_max}"
    );

    // WebGL2 / OpenGL ES 3.0 spec defaults: TEXTURE_BASE_LEVEL = 0, TEXTURE_MAX_LEVEL = 1000.
    assert_eq!
    (
      diffuse_base, 0,
      "diffuse_texture must be left at the spec default TEXTURE_BASE_LEVEL, not have the mip range misapplied to it, got {diffuse_base}"
    );
    assert_eq!
    (
      diffuse_max, 1000,
      "diffuse_texture must be left at the spec default TEXTURE_MAX_LEVEL, not have the mip range misapplied to it, got {diffuse_max}"
    );
  }
}
