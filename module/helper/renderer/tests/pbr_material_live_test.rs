//! Live WebGL2-context tests for `PbrMaterial` ( `renderer::webgl::material::pbr` ) -- construction
//! itself needs a real `&GL` reference ( `PbrMaterial::new`'s parameter, though internally unused,
//! is `&GL` not `Option< &GL >` ), so these pure-logic assertions still require a browser context,
//! unlike `tests/webgl/pbr_material.rs`'s enum-only native tests.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: `PbrMaterial::new` needs a real `&GL` reference to construct.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::{ material::PbrMaterial, AlphaMode, Material };

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap()
  }

  #[ wasm_bindgen_test ]
  fn new_constructs_with_expected_defaults()
  {
    let gl_context = gl_init();
    let mat = PbrMaterial::new( &gl_context );

    assert!( mat.need_use_ibl(), "PbrMaterial::new should default need_use_ibl to true" );
    assert!( !mat.has_emission(), "a freshly-constructed material has no emissive texture or factor" );
    assert_eq!( mat.alpha_mode(), AlphaMode::Opaque, "PbrMaterial::new should default alpha_mode to Opaque" );
    assert_eq!( mat.cull_mode, None, "PbrMaterial::new should default cull_mode to None" );
  }

  #[ wasm_bindgen_test ]
  fn need_use_ibl_set_flips_recompile_flag_only_on_actual_change()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    mat.recompile_flag_clear();
    assert!( !mat.needs_recompile(), "flag must start clear for this test to be meaningful" );

    // Same value as the current true default -- must NOT flip the flag.
    mat.need_use_ibl_set( true );
    assert!( !mat.needs_recompile(), "setting need_use_ibl to its current value must not request a recompile" );

    // An actual change -- must flip the flag.
    mat.need_use_ibl_set( false );
    assert!( !mat.need_use_ibl(), "need_use_ibl_set(false) should be reflected by need_use_ibl()" );
    assert!( mat.needs_recompile(), "an actual need_use_ibl change must request a recompile" );
  }

  #[ wasm_bindgen_test ]
  fn vertex_define_add_reflects_only_in_vertex_defines_str()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );

    mat.vertex_define_add( "MY_VERTEX_DEFINE", "1" );

    assert!( mat.vertex_defines_str().contains( "#define MY_VERTEX_DEFINE 1" ) );
    assert!( !mat.fragment_defines_str().contains( "MY_VERTEX_DEFINE" ) );
    assert!( mat.defines_str().contains( "#define MY_VERTEX_DEFINE 1" ), "defines_str is the vertex+fragment concatenation" );
  }

  #[ wasm_bindgen_test ]
  fn fragment_define_add_reflects_only_in_fragment_defines_str()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );

    mat.fragment_define_add( "MY_FRAGMENT_DEFINE", "2" );

    assert!( mat.fragment_defines_str().contains( "#define MY_FRAGMENT_DEFINE 2" ) );
    assert!( !mat.vertex_defines_str().contains( "MY_FRAGMENT_DEFINE" ) );
    assert!( mat.defines_str().contains( "#define MY_FRAGMENT_DEFINE 2" ) );
  }

  #[ wasm_bindgen_test ]
  fn define_add_reflects_in_both_vertex_and_fragment_defines_str()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );

    mat.define_add( "MY_SHARED_DEFINE", "3" );

    assert!( mat.vertex_defines_str().contains( "#define MY_SHARED_DEFINE 3" ) );
    assert!( mat.fragment_defines_str().contains( "#define MY_SHARED_DEFINE 3" ) );
  }

  #[ wasm_bindgen_test ]
  fn has_emission_defaults_false_and_flips_true_after_setting_emissive_factor()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    assert!( !mat.has_emission() );

    mat.emissive_factor = gl::F32x3::from( [ 1.0, 0.0, 0.0 ] );

    assert!( mat.has_emission(), "a non-zero emissive_factor must make has_emission true" );
  }

  #[ wasm_bindgen_test ]
  fn clone_generates_a_fresh_uuid_but_preserves_other_state()
  {
    let gl_context = gl_init();
    let mut original = PbrMaterial::new( &gl_context );
    original.vertex_define_add( "PRESERVED_DEFINE", "1" );

    let cloned = original.clone();

    assert_ne!( cloned.id, original.id, "Clone must generate a fresh uuid, not preserve the original's id" );
    assert_eq!( cloned.vertex_defines_str(), original.vertex_defines_str(), "Clone must preserve define state" );
    assert_eq!( cloned.base_color_factor, original.base_color_factor, "Clone must preserve scalar/vector state" );
  }
}
