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
  gl.enable( gl::DEPTH_TEST );
  gl.clear_color( 0.1, 0.1, 0.15, 1.0 );
  gl.viewport( 0, 0, width, height );

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
    [ 0.0, 1.5, 0.0 ].into(),
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
  let skull_model = mat3x3h::translation( [ 0.0, 1.5, 0.0 ] );

  // Create plane geometry
  let plane_vao = create_plane_vao( &gl )?;
  let plane_model = mat3x3h::translation( [ 0.0, -0.5, 0.0 ] ) * mat3x3h::scale( [ 8.0, 1.0, 8.0 ] );

  // Setup shadowmap
  let shadowmap_resolution = 4096;
  let shadowmap = shadowmap::Shadowmap::new( &gl, shadowmap_resolution )?;

  // Lighting parameters
  let light_pos = [ 0.0, 4.0, 5.0 ];
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
    mat3x3h::orthographic_rh_gl( -10.0, 10.0, -10.0, 10.0, near, far )
    // mat3x3h::perspective_rh_gl( 90.0f32.to_radians(), 1.0, near, far )
  );
  let is_orthographic = 1.0f32;
  let light_dir = gl::F32x3::new( 0.0, 0.0, -1.0 );  // Local forward
  let rotation_matrix = light_orientation.to_matrix();
  let light_dir = rotation_matrix * light_dir;

  // === Shadow Pass: Render depth from light's perspective ===
  shadowmap.bind( &gl );
  gl.viewport( 0, 0, shadowmap_resolution as i32, shadowmap_resolution as i32 );
  gl.clear( gl::DEPTH_BUFFER_BIT );

  let light_view_projection = light_source.view_projection();

  // Render back faces to shadow map to reduce shadow acne
  // This provides natural geometric offset instead of relying only on depth bias
  gl.enable( gl::CULL_FACE );
  gl.cull_face( gl::FRONT );

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

  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  gl.viewport( 0, 0, width, height );

  let update = move | _ |
  {
    // === Main Pass: Render scene with shadows ===
    gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );

    let view = camera.get_view_matrix();
    let projection = camera.get_projection_matrix();
    let view_projection = projection * view;

    shader.activate();
    shader.uniform_upload( "u_light_pos", &light_pos );
    shader.uniform_upload( "u_light_dir", light_dir.as_slice() );
    shader.uniform_upload( "u_view_pos", camera.get_eye().as_slice() );
    shader.uniform_upload( "u_light_color", &light_color );
    shader.uniform_matrix_upload( "u_light_view_projection", light_view_projection.raw_slice(), true );
    shader.uniform_upload( "u_is_orthographic", &is_orthographic );
    shader.uniform_upload( "u_shadow_map", &2 );

    // Bind shadow texture
    gl.active_texture( gl::TEXTURE2 );
    gl.bind_texture( gl::TEXTURE_2D, shadowmap.depth_texture() );

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

    // Render plane
    let plane_mvp = view_projection * plane_model;
    shader.uniform_matrix_upload( "u_mvp", plane_mvp.raw_slice(), true );
    shader.uniform_matrix_upload( "u_model", plane_model.raw_slice(), true );
    shader.uniform_upload( "u_object_color", &plane_color );
    gl.bind_vertex_array( Some( &plane_vao ) );
    gl.draw_elements_with_i32( gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, 0 );
    gl.bind_vertex_array( None );

    // === Debug: Visualize shadow map in corner ===
    // Set viewport to bottom-left corner (quarter size)
    let debug_size = ( width / 4 ).min( height / 4 );
    gl.viewport( 0, 0, debug_size, debug_size );

    // Disable depth test for overlay
    gl.disable( gl::DEPTH_TEST );

    debug_shader.activate();
    debug_shader.uniform_upload( "u_depth_texture", &0 );
    debug_shader.uniform_upload( "u_near", &near );
    debug_shader.uniform_upload( "u_far", &far );

    // Bind shadow map to texture unit 0
    gl.active_texture( gl::TEXTURE0 );
    gl.bind_texture( gl::TEXTURE_2D, shadowmap.depth_texture() );

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

/// Creates a simple plane geometry with positions and normals
fn create_plane_vao( gl : &gl::GL ) -> Result< web_sys::WebGlVertexArrayObject, gl::WebglError >
{
  // Plane vertices: two triangles forming a quad
  // Each vertex: [x, y, z, nx, ny, nz]
  let vertices : [ f32; 24 ] =
  [
    // First triangle
    -1.0, 0.0, -1.0,  0.0, 1.0, 0.0, // Bottom-left
     1.0, 0.0, -1.0,  0.0, 1.0, 0.0, // Bottom-right
     1.0, 0.0,  1.0,  0.0, 1.0, 0.0, // Top-right
    -1.0, 0.0,  1.0,  0.0, 1.0, 0.0, // Top-left
  ];

  let indices : [ u16; 6 ] = [ 0, 1, 2, 0, 2, 3 ];

  let vao = gl::vao::create( gl )?;
  gl.bind_vertex_array( Some( &vao ) );

  // Create and upload vertex buffer
  let vbo = gl::buffer::create( gl )?;
  gl.bind_buffer( gl::ARRAY_BUFFER, Some( &vbo ) );
  gl.buffer_data_with_u8_array( gl::ARRAY_BUFFER, vertices.as_bytes(), gl::STATIC_DRAW );

  // Position attribute (location 0)
  gl.vertex_attrib_pointer_with_i32( 0, 3, gl::FLOAT, false, 24, 0 );
  gl.enable_vertex_attrib_array( 0 );

  // Normal attribute (location 1)
  gl.vertex_attrib_pointer_with_i32( 1, 3, gl::FLOAT, false, 24, 12 );
  gl.enable_vertex_attrib_array( 1 );

  // Create and upload index buffer
  let ibo = gl::buffer::create( gl )?;
  gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &ibo ) );
  gl.buffer_data_with_u8_array( gl::ELEMENT_ARRAY_BUFFER, indices.as_bytes(), gl::STATIC_DRAW );

  gl.bind_vertex_array( None );

  Ok( vao )
}
