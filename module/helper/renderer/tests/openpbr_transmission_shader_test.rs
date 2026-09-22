//! Headless-browser tests for the transmission shader variants ( adoption plan
//! §3.3 / §3.3b / §3.4 ): the define gating that selects them and the GLSL
//! compile of every combination.
//!
//! `main.frag`'s `USE_TRANSMISSION` block now compiles into three shapes — the
//! volumetric slab, its Beer-Lambert absorption variant
//! ( `USE_TRANSMISSION_ABSORPTION` ) and the thin-walled shell
//! ( `USE_TRANSMISSION_THIN_WALLED` ) — each reached by a different carrier
//! combination, and each with its own uniform set. GLSL ES 3.00 only compiles
//! inside a browser ( see `shader_validation_tests.rs` for why naga cannot
//! stand in ), so this suite is the gate on all three; the arithmetic they
//! evaluate is native-tested in `openpbr_transmission_test.rs`.
//!
//! Follows the pattern of `openpbr_lobe_texture_test.rs` /
//! `clearcoat_anisotropy_shader_tests.rs`.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::{ material::{ OpenPbrParams, PbrMaterial }, Material };

  async fn init_gl() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// Compiles `material`'s shaders exactly the way `renderer.rs` does, and
  /// panics with the GL error on failure.
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

  /// A transmission weight alone selects the volumetric slab variant: the
  /// refraction block is in, absorption and the thin-walled shell are not.
  #[ wasm_bindgen_test( async ) ]
  async fn transmission_weight_alone_raises_the_slab_variant()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.openpbr_params_set
    (
      OpenPbrParams { transmission_factor : Some( 1.0 ), ..OpenPbrParams::default() }
    );

    let defines = material.defines_str().to_owned();
    assert!( defines.contains( "#define USE_TRANSMISSION\n" ), "missing USE_TRANSMISSION : {defines}" );
    assert!( !defines.contains( "USE_TRANSMISSION_ABSORPTION" ), "no attenuation distance - absorption must stay off : {defines}" );
    assert!( !defines.contains( "USE_TRANSMISSION_THIN_WALLED" ), "no zero thickness - thin-walled must stay off : {defines}" );

    assert_compiles( &gl, &material, false, false, "transmission weight only" );
  }

  /// A zero weight must not compile the refraction block at all — the material
  /// stays in the opaque/transparent passes, where the transmission samplers
  /// are never bound.
  #[ wasm_bindgen_test( async ) ]
  async fn zero_transmission_weight_keeps_the_block_out()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.openpbr_params_set
    (
      OpenPbrParams { transmission_factor : Some( 0.0 ), ..OpenPbrParams::default() }
    );

    let defines = material.defines_str().to_owned();
    assert!( !defines.contains( "USE_TRANSMISSION" ), "zero weight must not raise USE_TRANSMISSION : {defines}" );
    assert!( !material.transmission_active(), "zero weight must not route into the transmission pass" );

    assert_compiles( &gl, &material, false, false, "zero transmission weight" );
  }

  /// A finite attenuation distance ( the glTF carrier of OpenPBR
  /// `transmission_depth` ) adds the Beer-Lambert variant and its own uniform.
  #[ wasm_bindgen_test( async ) ]
  async fn attenuation_distance_raises_the_absorption_variant()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.openpbr_params_set
    (
      OpenPbrParams
      {
        transmission_factor : Some( 1.0 ),
        volume_thickness_factor : Some( 0.4 ),
        volume_attenuation_distance : Some( 0.25 ),
        volume_attenuation_color : Some( [ 0.8, 0.9, 1.0 ] ),
        ..OpenPbrParams::default()
      }
    );

    let defines = material.defines_str().to_owned();
    assert!( defines.contains( "#define USE_TRANSMISSION_ABSORPTION\n" ), "missing USE_TRANSMISSION_ABSORPTION : {defines}" );
    assert!( !defines.contains( "USE_TRANSMISSION_THIN_WALLED" ), "a positive thickness is not thin-walled : {defines}" );

    assert_compiles( &gl, &material, false, false, "transmission + Beer-Lambert absorption" );
  }

  /// Zero thickness is the thin-walled shell; it wins over absorption, which
  /// has no interior volume to act in.
  #[ wasm_bindgen_test( async ) ]
  async fn zero_thickness_raises_the_thin_walled_variant()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.openpbr_params_set
    (
      OpenPbrParams
      {
        transmission_factor : Some( 1.0 ),
        volume_thickness_factor : Some( 0.0 ),
        volume_attenuation_distance : Some( 0.25 ),
        ..OpenPbrParams::default()
      }
    );

    let defines = material.defines_str().to_owned();
    assert!( defines.contains( "#define USE_TRANSMISSION_THIN_WALLED\n" ), "missing USE_TRANSMISSION_THIN_WALLED : {defines}" );
    assert!( !defines.contains( "USE_TRANSMISSION_ABSORPTION" ), "thin-walled has no interior medium to absorb in : {defines}" );

    assert_compiles( &gl, &material, false, false, "thin-walled transmission" );
  }

  /// Worst case: the slab with absorption on top of every other OpenPBR lobe,
  /// IBL and the Kulla-Conty LUT — the combination with the most live uniforms
  /// and the deepest `main()`.
  #[ wasm_bindgen_test( async ) ]
  async fn transmission_with_every_other_lobe_compiles()
  {
    let gl = init_gl().await;
    let mut material = PbrMaterial::new( &gl );
    material.openpbr_params_set
    (
      OpenPbrParams
      {
        ior : Some( 1.5 ),
        transmission_factor : Some( 0.9 ),
        volume_thickness_factor : Some( 0.3 ),
        volume_attenuation_distance : Some( 0.6 ),
        volume_attenuation_color : Some( [ 0.7, 0.85, 1.0 ] ),
        sheen_color_factor : Some( [ 1.0, 0.5, 0.25 ] ),
        sheen_roughness_factor : Some( 0.4 ),
        iridescence_factor : Some( 0.7 ),
        iridescence_ior : Some( 1.4 ),
        iridescence_thickness_minimum : Some( 300.0 ),
        iridescence_thickness_maximum : Some( 400.0 ),
        emissive_strength : Some( 2.0 ),
        ..OpenPbrParams::default()
      }
    );

    assert_compiles( &gl, &material, true, true, "transmission + absorption + fuzz + thin film + IBL + Kulla-Conty" );
  }
}
