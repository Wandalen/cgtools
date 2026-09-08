//! Headless-browser tests for the OpenPBR textured lobe carriers ( adoption
//! plan §3.1 subset : `KHR_materials_sheen` color/roughness textures and
//! `KHR_materials_iridescence` weight texture on `PbrMaterial` ). Covers the
//! define gating ( a texture alone must raise `USE_OPENPBR` /
//! `USE_OPENPBR_IRIDESCENCE`, glTF factor-defaults-to-one semantics ) and the
//! shader compile of every new `USE_*_TEXTURE` combination, following the
//! pattern of `clearcoat_anisotropy_shader_tests.rs`.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::{ material::{ OpenPbrParams, PbrMaterial }, Material, Texture, TextureInfo };

  async fn init_gl() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// Texture contents are irrelevant to a shader-compile test - only its
  /// presence flips on the `USE_*_TEXTURE` defines and the `sampler2D` uniform.
  fn dummy_texture_info() -> TextureInfo
  {
    TextureInfo { texture : Rc::new( RefCell::new( Texture::new() ) ), uv_position : 0 }
  }

  /// Compiles `material`'s shaders exactly the way `renderer.rs` does ( same
  /// `#version` header, same per-stage defines, same optional `USE_IBL` /
  /// `USE_KULLA_CONTY` injections ), and panics with the GL error on failure.
  fn assert_compiles( gl : &GL, material : &PbrMaterial, with_ibl : bool, with_kulla : bool, label : &str )
  {
    let ibl_define = if with_ibl { "#define USE_IBL\n" } else { "" };
    let kulla_define = if with_kulla { "#define USE_KULLA_CONTY\n" } else { "" };
    let vs_src = format!( "#version 300 es\n{}\n{}", material.vertex_defines_str(), material.vertex_shader() );
    let fs_src = format!( "#version 300 es\n{}\n{}{}\n{}", material.fragment_defines_str(), ibl_define, kulla_define, material.fragment_shader() );

    gl::ProgramFromSources::new( &vs_src, &fs_src )
    .compile_and_link( gl )
    .unwrap_or_else( | e | panic!( "{label} failed to compile/link: {e:?}" ) );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn sheen_textures_alone_raise_use_openpbr()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    // No scalar carriers at all : texture presence must be the sole gate.
    assert_eq!( material.openpbr_params, OpenPbrParams::default() );
    material.set_sheen_color_texture( Some( dummy_texture_info() ) );
    material.set_sheen_roughness_texture( Some( dummy_texture_info() ) );

    let defines = material.defines_str().to_owned();
    assert!( defines.contains( "#define USE_OPENPBR\n" ), "sheen textures must raise USE_OPENPBR ( exact line ) : {defines}" );
    assert!( defines.contains( "#define USE_SHEEN_COLOR_TEXTURE" ), "missing USE_SHEEN_COLOR_TEXTURE : {defines}" );
    assert!( defines.contains( "#define USE_SHEEN_ROUGHNESS_TEXTURE" ), "missing USE_SHEEN_ROUGHNESS_TEXTURE : {defines}" );
    assert!( defines.contains( "#define vSheenColorUv vUv_0" ), "missing the vSheenColorUv uv alias : {defines}" );
    assert!( !defines.contains( "USE_OPENPBR_IRIDESCENCE" ), "no iridescence carrier - define must stay off" );

    assert_compiles( &gl, &material, false, false, "sheen textures, no scalars" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn iridescence_texture_alone_raises_the_layer_defines()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_iridescence_texture( Some( dummy_texture_info() ) );

    let defines = material.defines_str().to_owned();
    assert!( defines.contains( "#define USE_OPENPBR\n" ), "iridescence texture must raise USE_OPENPBR ( exact line ) : {defines}" );
    assert!( defines.contains( "#define USE_OPENPBR_IRIDESCENCE\n" ), "missing USE_OPENPBR_IRIDESCENCE : {defines}" );
    assert!( defines.contains( "#define USE_IRIDESCENCE_TEXTURE" ), "missing USE_IRIDESCENCE_TEXTURE : {defines}" );

    assert_compiles( &gl, &material, false, false, "iridescence texture, no scalars" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn lobe_textures_on_uv_channel_1_compile()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    let mut info = dummy_texture_info();
    info.uv_position = 1;
    material.set_sheen_color_texture( Some( info.clone() ) );
    material.set_iridescence_texture( Some( info ) );

    let defines = material.defines_str().to_owned();
    assert!( defines.contains( "#define vSheenColorUv vUv_1" ), "uv alias must follow TextureInfo::uv_position : {defines}" );
    assert!( defines.contains( "#define vIridescenceUv vUv_1" ), "uv alias must follow TextureInfo::uv_position : {defines}" );

    assert_compiles( &gl, &material, false, false, "lobe textures on uv set 1" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn lobe_textures_with_scalars_ibl_and_kulla_compile()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.openpbr_params_set
    (
      OpenPbrParams
      {
        sheen_color_factor : Some( [ 1.0, 0.5, 0.25 ] ),
        sheen_roughness_factor : Some( 0.4 ),
        iridescence_factor : Some( 0.7 ),
        iridescence_ior : Some( 1.4 ),
        iridescence_thickness_minimum : Some( 300.0 ),
        iridescence_thickness_maximum : Some( 400.0 ),
        ..OpenPbrParams::default()
      }
    );
    material.set_sheen_color_texture( Some( dummy_texture_info() ) );
    material.set_sheen_roughness_texture( Some( dummy_texture_info() ) );
    material.set_iridescence_texture( Some( dummy_texture_info() ) );
    material.specular_factor_set( Some( 0.5 ) );

    assert_compiles( &gl, &material, true, true, "all three lobe textures + scalars + IBL + Kulla-Conty, worst-case combo" );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn clearing_lobe_textures_drops_their_defines()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.set_sheen_color_texture( Some( dummy_texture_info() ) );
    material.set_iridescence_texture( Some( dummy_texture_info() ) );
    assert!( material.defines_str().contains( "USE_SHEEN_COLOR_TEXTURE" ));

    material.set_sheen_color_texture( None );
    material.set_iridescence_texture( None );

    let defines = material.defines_str().to_owned();
    assert!( !defines.contains( "USE_SHEEN_COLOR_TEXTURE" ), "define must disappear with the texture" );
    assert!( !defines.contains( "USE_IRIDESCENCE_TEXTURE" ), "define must disappear with the texture" );
    assert!( !defines.contains( "#define USE_OPENPBR\n" ), "no carriers left - USE_OPENPBR must be off" );
    assert!( !defines.contains( "USE_OPENPBR_IRIDESCENCE" ), "no iridescence carrier - define must be off" );

    assert_compiles( &gl, &material, false, false, "plain material after clearing lobe textures" );
  }
}
