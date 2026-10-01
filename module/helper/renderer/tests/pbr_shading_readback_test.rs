//! Pixel readback of the real PBR program ( `main.vert` / `main.frag` ) shading a full-screen quad.
//!
//! A `PbrMaterial`'s program is compiled from its defines, and the material uploads its own
//! uniforms and binds its own textures, as the renderer does; the test sets only the matrices
//! ( identity, so the quad's world position is its clip position, facing +Z ), the camera and
//! one directional light. No IBL, so a black base color leaves only direct light. The checks
//! compare pixels between two setups instead of pinning exact values, so they only depend on
//! the property under test.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::
  {
    material::{ PbrMaterial, PBRShader },
    Material,
    MaterialUploadContext,
    Node,
    ShaderProgram,
    Texture,
    TextureInfo,
  };

  /// UVs of the quad's corners in strip order ( -1, -1 ), ( 1, -1 ), ( -1, 1 ), ( 1, 1 ):
  /// u = ( 1 + x ) / 2 and v = ( 1 - y ) / 2, the image upright on the quad.
  const UPRIGHT : [ f32; 8 ] = [ 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0 ];

  const IDENTITY_4 : [ f32; 16 ] = [ 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0 ];
  const IDENTITY_3 : [ f32; 9 ] = [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ];

  /// A directional light: direction towards the light, and strength.
  type Light = ( [ f32; 3 ], f32 );

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// A 1x1 texture holding `rgba`, sampled from UV set `uv_position`.
  fn solid_texture( gl : &GL, rgba : [ u8; 4 ], uv_position : u32 ) -> TextureInfo
  {
    let source = gl.create_texture().expect( "create_texture succeeds" );
    gl.bind_texture( gl::TEXTURE_2D, Some( &source ) );
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    (
      gl::TEXTURE_2D, 0, gl::RGBA8 as i32, 1, 1, 0, gl::RGBA, gl::UNSIGNED_BYTE, Some( &rgba )
    ).expect( "tex_image_2d succeeds" );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32 );
    let mut texture = Texture::new();
    texture.source = Some( source );
    TextureInfo { texture : Rc::new( RefCell::new( texture ) ), uv_position }
  }

  /// A tangent-space normal-map texel: `n` mapped from [ -1, 1 ] to bytes.
  fn normal_texel( n : [ f32; 3 ] ) -> [ u8; 4 ]
  {
    let byte = | c : f32 | ( ( c * 0.5 + 0.5 ) * 255.0 ).round() as u8;
    [ byte( n[ 0 ] ), byte( n[ 1 ] ), byte( n[ 2 ] ), 255 ]
  }

  fn array_buffer( gl : &GL, location : u32, size : i32, data : &[ f32 ] )
  {
    let buffer = gl.create_buffer().expect( "create_buffer succeeds" );
    gl.bind_buffer( gl::ARRAY_BUFFER, Some( &buffer ) );
    gl.buffer_data_with_array_buffer_view( gl::ARRAY_BUFFER, &gl::js_sys::Float32Array::from( data ), gl::STATIC_DRAW );
    gl.enable_vertex_attrib_array( location );
    gl.vertex_attrib_pointer_with_i32( location, size, gl::FLOAT, false, 0, 0 );
  }

  /// Draws the quad with `material`'s program and UV sets 0 and 1, lit by `light` ( or by no
  /// light ), and returns the center pixel.
  fn shade( gl : &GL, material : &PbrMaterial, uv_0 : [ f32; 8 ], uv_1 : [ f32; 8 ], light : Option< Light > ) -> [ u8; 4 ]
  {
    let vs = format!( "#version 300 es\n{}\n{}", material.vertex_defines_str(), material.vertex_shader() );
    let fs = format!( "#version 300 es\n{}\n\n{}", material.fragment_defines_str(), material.fragment_shader() );
    let program = gl::ProgramFromSources::new( &vs, &fs ).compile_and_link( gl ).expect( "PBR program compiles" );
    let shader = PBRShader::new( gl, &program );
    gl.use_program( Some( &program ) );

    let vao = gl.create_vertex_array().expect( "create_vertex_array succeeds" );
    gl.bind_vertex_array( Some( &vao ) );
    array_buffer( gl, 0, 3, &[ -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0 ] );
    array_buffer( gl, 1, 3, &[ 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0 ] );
    array_buffer( gl, 2, 2, &uv_0 );
    array_buffer( gl, 3, 2, &uv_1 );

    let uniform = | name : &str | gl.get_uniform_location( &program, name );
    gl.uniform_matrix4fv_with_f32_array( uniform( "worldMatrix" ).as_ref(), false, &IDENTITY_4 );
    gl.uniform_matrix4fv_with_f32_array( uniform( "viewMatrix" ).as_ref(), false, &IDENTITY_4 );
    gl.uniform_matrix4fv_with_f32_array( uniform( "projectionMatrix" ).as_ref(), false, &IDENTITY_4 );
    gl.uniform_matrix3fv_with_f32_array( uniform( "normalMatrix" ).as_ref(), false, &IDENTITY_3 );
    gl.uniform3f( uniform( "cameraPosition" ).as_ref(), 0.0, 0.0, 5.0 );
    gl.uniform1i( uniform( "directLightsCount" ).as_ref(), i32::from( light.is_some() ) );
    if let Some( ( [ x, y, z ], strength ) ) = light
    {
      gl.uniform3f( uniform( "directLights[0].direction" ).as_ref(), x, y, z );
      gl.uniform3f( uniform( "directLights[0].color" ).as_ref(), 1.0, 1.0, 1.0 );
      gl.uniform1f( uniform( "directLights[0].strength" ).as_ref(), strength );
    }

    let node = Node::default();
    let ctx = MaterialUploadContext { node : &node, primitive_id : None, locations : shader.locations() };
    material.configure( gl, &ctx );
    material.upload_on_state_change( gl, &ctx ).expect( "material uniforms upload" );
    material.bind( gl );

    let ( width, height ) = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.viewport( 0, 0, width, height );
    gl.clear_color( 0.0, 0.0, 0.0, 0.0 );
    gl.clear( gl::COLOR_BUFFER_BIT );
    gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );
    assert_eq!( gl.get_error(), gl::NO_ERROR, "the draw raises no GL error" );

    let mut pixel = [ 0_u8; 4 ];
    gl.read_pixels_with_opt_u8_array( width / 2, height / 2, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) )
    .expect( "read_pixels succeeds" );
    gl.bind_vertex_array( None );
    gl.delete_vertex_array( Some( &vao ) );
    gl.delete_program( Some( &program ) );
    pixel
  }

  /// A black dielectric whose only visible light is the coat: base color 0, metallic 0.
  fn black_dielectric( gl : &GL ) -> PbrMaterial
  {
    let mut material = PbrMaterial::new( gl );
    material.base_color_factor = gl::F32x4::from( [ 0.0, 0.0, 0.0, 1.0 ] );
    material.metallic_factor = 0.0;
    material
  }

  /// The coat has its own normal, so its highlight must not depend on the base normal facing the
  /// light. Here a base normal map tilts the base normal to -X, away from a light the smooth
  /// coat ( normal +Z ) faces; the coat's highlight must still be drawn.
  #[ wasm_bindgen_test ]
  fn clearcoat_highlight_survives_a_base_normal_facing_away()
  {
    let gl = gl_init();
    let mut material = black_dielectric( &gl );
    material.normal_texture_set( Some( solid_texture( &gl, normal_texel( [ -1.0, 0.0, 0.05 ] ), 0 ) ) );
    material.clearcoat_factor_set( Some( 1.0 ) );
    material.clearcoat_roughness_factor_set( Some( 0.3 ) );
    let light = ( [ 0.447_214, 0.0, 0.894_427 ], 50.0 );

    let lit = shade( &gl, &material, UPRIGHT, UPRIGHT, Some( light ) );
    let unlit = shade( &gl, &material, UPRIGHT, UPRIGHT, None );

    assert!( lit[ 0 ] > unlit[ 0 ].saturating_add( 10 ), "coat highlight missing: lit {lit:?}, unlit {unlit:?}" );
  }

  /// `anisotropyStrength` is defined on [ 0, 1 ]. Out-of-range values, from an asset or from
  /// `anisotropy_strength_set`, must shade like the nearest bound: above 1 the tangent roughness
  /// leaves the GGX model, and a negative strength widens the lobe as if it were positive.
  #[ wasm_bindgen_test ]
  fn anisotropy_strength_is_clamped_to_its_range()
  {
    let gl = gl_init();
    let light = ( [ 0.4, 0.3, 0.866_025 ], 20.0 );
    let shade_with = | strength : f32 |
    {
      let mut material = black_dielectric( &gl );
      material.roughness_factor = 0.5;
      material.anisotropy_strength_set( Some( strength ) );
      shade( &gl, &material, UPRIGHT, UPRIGHT, Some( light ) )
    };
    let ( none, full ) = ( shade_with( 0.0 ), shade_with( 1.0 ) );
    assert_ne!( none, full, "precondition: anisotropy changes this pixel" );

    assert_eq!( shade_with( 2.0 ), full, "strength 2 must shade as 1" );
    assert_eq!( shade_with( -1.0 ), none, "strength -1 must shade as 0" );
  }
}
