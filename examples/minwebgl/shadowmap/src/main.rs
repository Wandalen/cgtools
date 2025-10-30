//! Simple skull rendering with basic lighting and shadowmapping

mod shadowmap;

use minwebgl as gl;
use gl::{ JsCast as _, math::mat3x3h, AsBytes as _, QuatF32 };
use web_sys::HtmlCanvasElement;
use renderer::webgl::loaders::gltf;

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { gl::info!( "{:?}", run().await ) } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let width = ( fwidth * dpr ) as i32;
  let height = ( fheight * dpr ) as i32;
  let aspect = width as f32 / height as f32;

  let gl = gl::context::retrieve_or_make()
  .expect( "Failed to retrieve WebGl context" );

  // Enable extensions for float texture rendering
  let ext_color_buffer_float = gl.get_extension( "EXT_color_buffer_float" );
  let ext_float_linear = gl.get_extension( "OES_texture_float_linear" );
  gl::info!( "{ext_color_buffer_float:?}" );
  gl::info!( "{ext_float_linear:?}" );

  gl.enable( gl::DEPTH_TEST );

  let canvas = gl.canvas()
  .unwrap()
  .dyn_into::< HtmlCanvasElement >()
  .unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  // Setup camera with orbit controls
  let mut camera = renderer::webgl::Camera::new
  (
    [ 0.0, 1.5, 5.0 ].into(),
    [ 0.0, 1.0, 0.0 ].into(),
    [ 0.0, 0.5, 0.0 ].into(),
    aspect,
    45.0_f32.to_radians(),
    0.1,
    100.0
  );
  camera.set_window_size( [ width as f32, height as f32 ].into() );
  camera.bind_controls( &canvas );

  // Create shader program with basic lighting
  let vertex_src = include_str!( "shaders/geom.vert" );
  let fragment_src = include_str!( "shaders/geom.frag" );
  let shader = gl::Program::new( gl.clone(), vertex_src, fragment_src )?;

  // Create debug shader for visualizing shadow map
  let debug_vert_src = include_str!( "shaders/debug_shadowmap.vert" );
  let debug_frag_src = include_str!( "shaders/debug_shadowmap.frag" );
  let debug_shader = gl::Program::new( gl.clone(), debug_vert_src, debug_frag_src )?;

  // Load skull model
  let skull = gltf::load( &document, "skull_salazar_downloadable.glb", &gl ).await?;
  let skull_model = mat3x3h::translation( [ 0.0, 0.5, 0.0 ] );

  // Create plane geometry
  let plane_vao = create_plane_vao( &gl )?;
  let plane_model = mat3x3h::translation( [ 0.0, -0.5, 0.0 ] ) * mat3x3h::scale( [ 8.0, 1.0, 8.0 ] );

  // Setup shadowmap
  let shadowmap_resolution = 4096;
  let shadowmap = shadowmap::Shadowmap::new( &gl, shadowmap_resolution )?;

  // Setup lightmap for baked shadows
  let lightmap_resolution = 2048;
  let plane_lightmap = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );

  // Calculate number of mipmap levels: floor(log2(resolution)) + 1
  let mip_levels = ( lightmap_resolution as f32 ).log2().floor() as i32 + 1;

  // Allocate texture storage with mipmaps
  gl.tex_storage_2d( gl::TEXTURE_2D, mip_levels, gl::RGBA16F, lightmap_resolution, lightmap_resolution );
  gl::texture::d2::wrap_clamp( &gl );

  // Use trilinear filtering for smooth mipmap transitions
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

  // Setup shadow baker
  let shadow_baker = shadowmap::ShadowBaker::new( &gl )?;
  shadow_baker.set_target( plane_lightmap.as_ref() );

  // Lighting parameters
  let light_pos = [ 0.0, 2.0, 3.0 ];
  let light_color = [ 1.0, 1.0, 1.0 ];
  let skull_color = [ 0.9, 0.85, 0.8 ];
  let plane_color = [ 0.3, 0.3, 0.3 ];

  let near = 0.5;
  let far = 30.0;

  // Setup light source for shadows
  let light_orientation = QuatF32::from_euler_xyz( [ -30.0_f32.to_radians(), 0.0, 0.0 ] );
  let mut light_source = shadowmap::LightSource::new
  (
    light_pos.into(),
    light_orientation,
    // mat3x3h::orthographic_rh_gl( -10.0, 10.0, -10.0, 10.0, near, far )
    mat3x3h::perspective_rh_gl( 45.0_f32.to_radians(), 1.0, near, far )
  );

  // === Shadow Pass: Render depth from light's perspective ===
  shadowmap.bind();
  // Render back faces to shadow map to reduce shadow acne
  // This provides natural geometric offset instead of relying only on depth bias
  gl.enable( gl::CULL_FACE );
  gl.cull_face( gl::FRONT );

  let light_view_projection = light_source.view_projection();

  // Render skull to shadowmap (closed mesh, safe for back-face rendering)
  shadowmap.upload_mvp( light_view_projection * skull_model );
  for mesh in &skull.meshes
  {
    for primitive in &mesh.borrow().primitives
    {
      let primitive = primitive.borrow();
      primitive.geometry.borrow().bind( &gl );
      primitive.draw( &gl );
    }
  }
  // Plane is single-sided, render it with front faces (or disable culling)
  gl.disable( gl::CULL_FACE );

  shadowmap.upload_mvp( light_view_projection * plane_model );
  gl.bind_vertex_array( Some( &plane_vao ) );
  gl.draw_elements_with_i32( gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, 0 );

  // Reset to normal culling
  gl.cull_face( gl::BACK );

  // === Lightmap Baking Pass: Bake PCSS shadows into lightmap ===
  shadow_baker.bind( lightmap_resolution as u32 );
  shadow_baker.upload_model( plane_model );
  shadow_baker.upload_light_source( &mut light_source );
  shadow_baker.set_shadow_map_unit( 2 );

  // Bind shadow map texture for sampling
  gl.active_texture( gl::TEXTURE2 );
  gl.bind_texture( gl::TEXTURE_2D, shadowmap.depth_texture() );

  // Render plane to bake shadows
  gl.bind_vertex_array( Some( &plane_vao ) );
  gl.draw_elements_with_i32( gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, 0 );

  // Unbind framebuffer
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  // Generate mipmaps for the lightmap texture
  gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );
  gl.generate_mipmap( gl::TEXTURE_2D );

  // Restore viewport and clear color
  gl.viewport( 0, 0, width, height );
  gl.clear_color( 0.1, 0.1, 0.15, 1.0 );

  let update = move | _ |
  {
    // === Main Pass: Render scene with shadows ===
    gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );

    let view = camera.get_view_matrix();
    let projection = camera.get_projection_matrix();
    let view_projection = projection * view;

    shader.activate();
    shader.uniform_upload( "u_light_pos", &light_pos );
    shader.uniform_upload( "u_view_pos", camera.get_eye().as_slice() );
    shader.uniform_upload( "u_light_color", &light_color );
    // shader.uniform_upload( "u_shadow_map", &2 );
    shader.uniform_upload( "u_lightmap", &1 );

    gl.active_texture( gl::TEXTURE1 );
    gl.bind_texture( gl::TEXTURE_2D, None );

    // Render skull
    gl.enable( gl::CULL_FACE );
    let skull_mvp = view_projection * skull_model;
    shader.uniform_matrix_upload( "u_mvp", skull_mvp.raw_slice(), true );
    shader.uniform_matrix_upload( "u_model", skull_model.raw_slice(), true );
    shader.uniform_upload( "u_object_color", &skull_color );
    for mesh in &skull.meshes
    {
      for primitive in &mesh.borrow().primitives
      {
        let primitive = primitive.borrow();
        primitive.geometry.borrow().bind( &gl );
        primitive.draw( &gl );
      }
    }
    gl.disable( gl::CULL_FACE );

    gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );
    // Render plane
    let plane_mvp = view_projection * plane_model;
    shader.uniform_matrix_upload( "u_mvp", plane_mvp.raw_slice(), true );
    shader.uniform_matrix_upload( "u_model", plane_model.raw_slice(), true );
    shader.uniform_upload( "u_object_color", &plane_color );
    gl.bind_vertex_array( Some( &plane_vao ) );
    gl.draw_elements_with_i32( gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, 0 );
    gl.bind_vertex_array( None );

    // === Debug: Visualize lightmap in corner ===
    // Set viewport to bottom-left corner (quarter size)
    let debug_size = ( width / 4 ).min( height / 4 );
    gl.viewport( 0, 0, debug_size, debug_size );

    // Disable depth test for overlay
    gl.disable( gl::DEPTH_TEST );

    debug_shader.activate();
    debug_shader.uniform_upload( "u_depth_texture", &0 );
    // debug_shader.uniform_upload( "u_near", &near );
    // debug_shader.uniform_upload( "u_far", &far );

    // Bind lightmap texture to texture unit 0
    gl.active_texture( gl::TEXTURE0 );
    gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );

    // === OLD: Display depth texture ===
    // gl.bind_texture( gl::TEXTURE_2D, shadowmap.depth_texture() );

    // Draw fullscreen triangle (no vertex buffer needed)
    gl.draw_arrays( gl::TRIANGLES, 0, 3 );

    // Restore viewport and depth test
    gl.viewport( 0, 0, width, height );
    gl.enable( gl::DEPTH_TEST );

    true
  };

  gl::exec_loop::run( update );

  Ok( () )
}

/// Creates a simple plane geometry with positions, normals, and UV coordinates
fn create_plane_vao( gl : &gl::GL ) -> Result< web_sys::WebGlVertexArrayObject, gl::WebglError >
{
  // Plane vertices: two triangles forming a quad
  // Each vertex: [x, y, z, nx, ny, nz, u, v]
  let vertices : [ f32; 32 ] =
  [
    // Position           Normal           UV
    -1.0, 0.0, -1.0,  0.0, 1.0, 0.0,  0.0, 0.0, // Bottom-left
     1.0, 0.0, -1.0,  0.0, 1.0, 0.0,  1.0, 0.0, // Bottom-right
     1.0, 0.0,  1.0,  0.0, 1.0, 0.0,  1.0, 1.0, // Top-right
    -1.0, 0.0,  1.0,  0.0, 1.0, 0.0,  0.0, 1.0, // Top-left
  ];

  let indices : [ u16; 6 ] = [ 0, 1, 2, 0, 2, 3 ];

  let vao = gl::vao::create( gl )?;
  gl.bind_vertex_array( Some( &vao ) );

  // Create and upload vertex buffer
  let vbo = gl::buffer::create( gl )?;
  gl.bind_buffer( gl::ARRAY_BUFFER, Some( &vbo ) );
  gl.buffer_data_with_u8_array( gl::ARRAY_BUFFER, vertices.as_bytes(), gl::STATIC_DRAW );

  let stride = 32; // 8 floats * 4 bytes = 32 bytes per vertex

  // Position attribute (location 0)
  gl.vertex_attrib_pointer_with_i32( 0, 3, gl::FLOAT, false, stride, 0 );
  gl.enable_vertex_attrib_array( 0 );

  // Normal attribute (location 1)
  gl.vertex_attrib_pointer_with_i32( 1, 3, gl::FLOAT, false, stride, 12 );
  gl.enable_vertex_attrib_array( 1 );

  // UV attribute (location 2) - for lightmap baking
  gl.vertex_attrib_pointer_with_i32( 2, 2, gl::FLOAT, false, stride, 24 );
  gl.enable_vertex_attrib_array( 2 );

  // Create and upload index buffer
  let ibo = gl::buffer::create( gl )?;
  gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &ibo ) );
  gl.buffer_data_with_u8_array( gl::ELEMENT_ARRAY_BUFFER, indices.as_bytes(), gl::STATIC_DRAW );

  gl.bind_vertex_array( None );

  Ok( vao )
}
