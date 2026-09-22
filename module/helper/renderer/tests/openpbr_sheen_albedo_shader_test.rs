//! Headless-browser tests for the fuzz ( sheen ) albedo-scaling path
//! ( `USE_SHEEN_ALBEDO` ) : the define is injected by the `Renderer` rather than
//! by the material, so what needs covering is that the shader compiles with it
//! on, that it composes with the other renderer-injected defines
//! ( `USE_IBL` / `USE_KULLA_CONTY` ), and that leaving it off still compiles -
//! the LUT is optional and a renderer without one keeps the older additive
//! behaviour. Follows the pattern of `openpbr_lobe_texture_test.rs`.
//!
//! Not runnable in this environment ( GLSL ES 3.00 only compiles inside a
//! browser ) ; CI with `cargo test --target wasm32-unknown-unknown` + a browser
//! driver is the gate. The albedo table's arithmetic is native-tested in
//! `sheen_albedo_test.rs`.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::{ material::PbrMaterial, Material };

  async fn init_gl() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// A material on the OpenPBR path with a live fuzz layer - the only case in
  /// which the renderer injects `USE_SHEEN_ALBEDO` at all.
  fn fuzzy_material( gl : &GL ) -> PbrMaterial
  {
    let mut material = PbrMaterial::new( gl );
    let mut params = material.openpbr_params.clone();
    params.sheen_color_factor = Some( [ 0.9, 0.8, 0.7 ] );
    params.sheen_roughness_factor = Some( 0.4 );
    material.openpbr_params_set( params );
    assert!
    (
      material.defines_str().contains( "#define USE_OPENPBR\n" ),
      "a fuzz carrier must raise USE_OPENPBR - otherwise the renderer never injects USE_SHEEN_ALBEDO"
    );
    material
  }

  /// Compiles `material`'s shaders exactly the way `renderer.rs` does, with the
  /// same renderer-injected defines in the same order.
  fn assert_compiles
  (
    gl : &GL,
    material : &PbrMaterial,
    with_ibl : bool,
    with_kulla : bool,
    with_sheen_albedo : bool,
    label : &str
  )
  {
    let ibl_define = if with_ibl { "#define USE_IBL\n" } else { "" };
    let kulla_define = if with_kulla { "#define USE_KULLA_CONTY\n" } else { "" };
    let sheen_define = if with_sheen_albedo { "#define USE_SHEEN_ALBEDO\n" } else { "" };
    let vs_src = format!( "#version 300 es\n{}\n{}", material.vertex_defines_str(), material.vertex_shader() );
    let fs_src = format!
    (
      "#version 300 es\n{}\n{}\n{}\n{}\n{}",
      material.fragment_defines_str(),
      ibl_define,
      kulla_define,
      sheen_define,
      material.fragment_shader()
    );

    gl::ProgramFromSources::new( &vs_src, &fs_src )
    .compile_and_link( gl )
    .unwrap_or_else( | e | panic!( "{label} failed to compile/link: {e:?}" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn fuzz_compiles_with_the_albedo_lut()
  {
    let gl = init_gl().await;
    let material = fuzzy_material( &gl );
    assert_compiles( &gl, &material, false, false, true, "fuzz + USE_SHEEN_ALBEDO" );
  }

  /// The LUT is optional : a renderer that was never handed one must still
  /// produce a compiling shader, just without the scaling.
  #[ wasm_bindgen_test( async ) ]
  async fn fuzz_compiles_without_the_albedo_lut()
  {
    let gl = init_gl().await;
    let material = fuzzy_material( &gl );
    assert_compiles( &gl, &material, false, false, false, "fuzz, no USE_SHEEN_ALBEDO" );
  }

  /// The environment fuzz term reads the LUT from inside the `USE_IBL` block, so
  /// that combination is the one that actually exercises `sheen_e` twice.
  #[ wasm_bindgen_test( async ) ]
  async fn fuzz_albedo_composes_with_ibl_and_kulla_conty()
  {
    let gl = init_gl().await;
    let material = fuzzy_material( &gl );
    assert_compiles( &gl, &material, true, true, true, "fuzz + IBL + Kulla-Conty + USE_SHEEN_ALBEDO" );
  }
}
