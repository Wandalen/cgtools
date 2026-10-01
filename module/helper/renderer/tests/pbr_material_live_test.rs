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
  use renderer::webgl::{ material::{ PbrMaterial, PBR_TEXTURE_UNITS }, AlphaMode, Material, Texture, TextureInfo };
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
  fn clearcoat_normal_texture_needs_a_positive_factor()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    mat.clearcoat_normal_texture_set( Some( texture_info() ) );
    let defines = mat.fragment_defines_str().to_owned();
    assert!( !defines.contains( "USE_KHR_materials_clearcoat" ), "no clearcoatFactor, no coat: {defines}" );
    assert!( !defines.contains( "USE_TBN" ), "{defines}" );

    mat.clearcoat_factor_set( Some( 0.5 ) );
    let defines = mat.fragment_defines_str();
    assert!( defines.contains( "#define USE_KHR_materials_clearcoat" ), "{defines}" );
    assert!( defines.contains( "#define USE_CLEARCOAT_NORMAL_TEXTURE" ), "{defines}" );
    assert!( defines.contains( "#define USE_TBN" ), "a coat normal map needs the tangent frame: {defines}" );
    assert!( !defines.contains( "USE_KHR_materials_anisotropy" ), "{defines}" );
  }

  #[ wasm_bindgen_test ]
  fn texture_unit_table_pairs_each_sampler_with_its_own_texture()
  {
    let gl_context = gl_init();
    let setters : [ ( &str, fn( &mut PbrMaterial, Option< TextureInfo > ) ); 12 ] =
    [
      ( "metallicRoughnessTexture", PbrMaterial::metallic_roughness_texture_set ),
      ( "baseColorTexture", PbrMaterial::base_color_texture_set ),
      ( "normalTexture", PbrMaterial::normal_texture_set ),
      ( "occlusionTexture", PbrMaterial::occlusion_texture_set ),
      ( "emissiveTexture", PbrMaterial::emissive_texture_set ),
      ( "specularTexture", PbrMaterial::specular_texture_set ),
      ( "specularColorTexture", PbrMaterial::specular_color_texture_set ),
      ( "lightMap", PbrMaterial::light_map_set ),
      ( "clearcoatTexture", PbrMaterial::clearcoat_texture_set ),
      ( "clearcoatRoughnessTexture", PbrMaterial::clearcoat_roughness_texture_set ),
      ( "clearcoatNormalTexture", PbrMaterial::clearcoat_normal_texture_set ),
      ( "anisotropyTexture", PbrMaterial::anisotropy_texture_set ),
    ];

    for ( sampler, set ) in setters
    {
      let mut mat = PbrMaterial::new( &gl_context );
      set( &mut mat, Some( texture_info() ) );
      let bound : Vec< &str > = PBR_TEXTURE_UNITS.iter()
      .filter( | ( _, _, texture ) | texture( &mat ).is_some() )
      .map( | ( name, _, _ ) | *name )
      .collect();
      assert_eq!( bound, [ sampler ], "the texture set for {sampler} must be bound to {sampler}'s unit, and only there" );
    }
  }

  #[ wasm_bindgen_test ]
  fn uv_set_beyond_the_shader_falls_back_to_set_zero()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    mat.clearcoat_factor_set( Some( 1.0 ) );
    let mut info = texture_info();
    info.uv_position = 5;
    mat.clearcoat_texture_set( Some( info.clone() ) );
    mat.base_color_texture_set( Some( info ) );
    let defines = mat.fragment_defines_str();

    assert!( !defines.contains( "vUv_5" ), "main.frag declares vUv_0 to vUv_4 only: {defines}" );
    assert!( defines.contains( "#define vClearcoatUv vUv_0" ), "{defines}" );
    assert!( defines.contains( "#define vBaseColorUv vUv_0" ), "{defines}" );
  }

  #[ wasm_bindgen_test ]
  fn zero_layer_factors_select_no_layer_variant()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    mat.clearcoat_factor_set( Some( 0.0 ) );
    mat.clearcoat_roughness_factor_set( Some( 0.5 ) );
    mat.clearcoat_texture_set( Some( texture_info() ) );
    mat.anisotropy_strength_set( Some( 0.0 ) );
    mat.anisotropy_texture_set( Some( texture_info() ) );
    let defines = mat.fragment_defines_str();

    assert!( !defines.contains( "USE_KHR_materials_clearcoat" ), "{defines}" );
    assert!( !defines.contains( "USE_CLEARCOAT_TEXTURE" ), "{defines}" );
    assert!( !defines.contains( "USE_KHR_materials_anisotropy" ), "{defines}" );
    assert!( !defines.contains( "USE_TBN" ), "{defines}" );
  }

  #[ wasm_bindgen_test ]
  fn factor_change_within_a_variant_skips_the_recompile()
  {
    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    mat.clearcoat_factor_set( Some( 1.0 ) );
    mat.anisotropy_strength_set( Some( 0.5 ) );
    mat.specular_factor_set( Some( 0.5 ) );
    mat.recompile_flag_clear();
    mat.needs_update_set( false );

    mat.clearcoat_factor_set( Some( 0.5 ) );
    mat.clearcoat_roughness_factor_set( Some( 0.2 ) );
    mat.anisotropy_strength_set( Some( 0.8 ) );
    mat.specular_factor_set( Some( 0.7 ) );
    assert!( !mat.needs_recompile(), "the define set is unchanged" );
    assert!( mat.needs_update(), "the new factors still have to be uploaded" );

    mat.clearcoat_factor_set( Some( 0.0 ) );
    assert!( mat.needs_recompile(), "a zero coat factor drops the coat variant" );
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

  #[ wasm_bindgen_test ]
  fn empty_layer_extension_objects_select_no_layer_variant()
  {
    let gltf = gltf::Gltf::from_slice_without_validation( br#"
    {
      "asset" : { "version" : "2.0" },
      "materials" : [ { "extensions" : { "KHR_materials_clearcoat" : {}, "KHR_materials_anisotropy" : {} } } ]
    }"# ).expect( "fixture parses" );
    let gltf_m = gltf.materials().next().expect( "one material" );

    let gl_context = gl_init();
    let mut mat = PbrMaterial::new( &gl_context );
    material_layer_extensions_apply( &gltf_m, &[], &mut mat );
    let defines = mat.fragment_defines_str();

    assert!( !defines.contains( "USE_KHR_materials_clearcoat" ), "clearcoatFactor defaults to 0: {defines}" );
    assert!( !defines.contains( "USE_KHR_materials_anisotropy" ), "anisotropyStrength defaults to 0: {defines}" );
  }
}
