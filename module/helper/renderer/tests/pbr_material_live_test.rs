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
  use renderer::webgl::{ material::PbrMaterial, AlphaMode, Material, Texture, TextureInfo };
  use renderer::webgl::loaders::gltf_extensions::material_layer_extensions_apply;
  use std::{ cell::RefCell, rc::Rc };

  fn texture_info() -> TextureInfo
  {
    TextureInfo { texture : Rc::new( RefCell::new( Texture::new() ) ), uv_position : 0 }
  }

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

  #[ wasm_bindgen_test ]
  fn clearcoat_normal_texture_alone_enables_clearcoat_and_tbn()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    mat.clearcoat_normal_texture_set( Some( texture_info() ) );
    let defines = mat.fragment_defines_str();

    assert!( defines.contains( "#define USE_KHR_materials_clearcoat" ), "{defines}" );
    assert!( defines.contains( "#define USE_CLEARCOAT_NORMAL_TEXTURE" ), "{defines}" );
    assert!( defines.contains( "#define USE_TBN" ), "a coat normal map needs the tangent frame: {defines}" );
    assert!( !defines.contains( "USE_KHR_materials_anisotropy" ), "{defines}" );
  }

  #[ wasm_bindgen_test ]
  fn anisotropy_strength_alone_enables_anisotropy_and_tbn()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    mat.anisotropy_strength_set( Some( 0.5 ) );
    let defines = mat.fragment_defines_str();

    assert!( defines.contains( "#define USE_KHR_materials_anisotropy" ), "{defines}" );
    assert!( defines.contains( "#define USE_TBN" ), "anisotropy needs the tangent frame: {defines}" );
    assert!( !defines.contains( "USE_ANISOTROPY_TEXTURE" ), "{defines}" );
    assert!( !defines.contains( "USE_KHR_materials_clearcoat" ), "{defines}" );
  }

  #[ wasm_bindgen_test ]
  fn plain_material_enables_no_layer_extension()
  {
    let gl_context = gl_init();
    let defines = PbrMaterial::new( &gl_context ).fragment_defines_str().to_owned();

    assert!( !defines.contains( "USE_KHR_materials_clearcoat" ), "{defines}" );
    assert!( !defines.contains( "USE_KHR_materials_anisotropy" ), "{defines}" );
    assert!( !defines.contains( "USE_TBN" ), "{defines}" );
  }

  #[ wasm_bindgen_test ]
  fn layer_extensions_apply_maps_json_onto_material()
  {
    // One texture in the asset: index 0 resolves (with its texCoord), index 5 is out of range
    // and must leave that texture unset rather than panic.
    let gltf = gltf::Gltf::from_slice_without_validation( br#"
    {
      "asset" : { "version" : "2.0" },
      "materials" : [ { "extensions" : {
        "KHR_materials_clearcoat" : {
          "clearcoatFactor" : 1.0,
          "clearcoatNormalTexture" : { "index" : 0, "texCoord" : 1, "scale" : 0.5 },
          "clearcoatTexture" : { "index" : 5 }
        },
        "KHR_materials_anisotropy" : { "anisotropyStrength" : 0.4, "anisotropyRotation" : 0.3 }
      } } ]
    }"# ).expect( "fixture parses" );
    let gltf_m = gltf.materials().next().expect( "one material" );
    let textures = [ Rc::new( RefCell::new( Texture::new() ) ) ];

    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    material_layer_extensions_apply( &gltf_m, &textures, &mut mat );

    assert_eq!( mat.clearcoat_factor(), Some( 1.0 ) );
    assert_eq!( mat.clearcoat_roughness_factor(), Some( 0.0 ), "absent factor gets the extension default" );
    assert!( mat.clearcoat_texture().is_none(), "out-of-range texture index must leave the texture unset" );
    assert_eq!( mat.clearcoat_normal_texture().map( | t | t.uv_position ), Some( 1 ) );
    assert!( ( mat.clearcoat_normal_scale - 0.5 ).abs() < 1e-6 );
    assert_eq!( mat.anisotropy_strength(), Some( 0.4 ) );
    assert!( ( mat.anisotropy_rotation - 0.3 ).abs() < 1e-6 );
  }
}
