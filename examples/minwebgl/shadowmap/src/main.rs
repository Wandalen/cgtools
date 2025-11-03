//! Simple skull rendering with basic lighting and shadowmapping

mod shadowmap;

use minwebgl as gl;
use gl::{ JsCast as _, math::mat3x3h, QuatF32 };
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

  let canvas = gl.canvas()
  .unwrap()
  .dyn_into::< HtmlCanvasElement >()
  .unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  // Enable extensions for float texture rendering
  let ext_color_buffer_float = gl.get_extension( "EXT_color_buffer_float" );
  let ext_float_linear = gl.get_extension( "OES_texture_float_linear" );
  gl::info!( "{ext_color_buffer_float:?}" );
  gl::info!( "{ext_float_linear:?}" );

  let mut camera = renderer::webgl::Camera::new
  (
    [ 0.0, 1.5, 5.0 ].into(),
    [ 0.0, 1.0, 0.0 ].into(),
    [ 0.0, 0.2, 0.0 ].into(),
    aspect,
    45.0_f32.to_radians(),
    0.1,
    100.0
  );
  camera.set_window_size( [ width as f32, height as f32 ].into() );
  camera.bind_controls( &canvas );

  let vertex_src = include_str!( "shaders/main.vert" );
  let fragment_src = include_str!( "shaders/main.frag" );
  let shader = gl::Program::new( gl.clone(), vertex_src, fragment_src )?;

  let debug_vert_src = include_str!( "shaders/debug_shadowmap.vert" );
  let debug_frag_src = include_str!( "shaders/debug_shadowmap.frag" );
  let debug_shader = gl::Program::new( gl.clone(), debug_vert_src, debug_frag_src )?;

  let mesh = gltf::load( &document, "ring.glb", &gl ).await?;
  let mesh_model = mat3x3h::translation( [ 0.0, 0.2, 0.0 ] );
  gl.bind_vertex_array( None );

  let cube_mesh = gltf::load( &document, "cube.glb", &gl ).await?;
  let plane_model = mat3x3h::translation( [ 0.0, -1.0, 0.0 ] ) * mat3x3h::scale( [ 8.0, 0.5, 8.0 ] );

  let shadowmap_resolution = 4096;
  let shadowmap = shadowmap::Shadowmap::new( &gl, shadowmap_resolution )?;

  let shadow_renderer = shadowmap::ShadowRenderer::new( &gl )?;

  let lightmap_res = 8192;
  let mip_levels = ( lightmap_res as f32 ).log2().floor() as i32 + 1;
  let plane_lightmap = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );
  gl.tex_storage_2d( gl::TEXTURE_2D, mip_levels, gl::RGBA16F, lightmap_res, lightmap_res );
  gl::texture::d2::wrap_clamp( &gl );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );


  // Lighting parameters
  let mesh_color = [ 0.9, 0.85, 0.8 ];
  let plane_color = [ 0.3, 0.3, 0.3 ];

  // Setup light source for shadows
  let light_pos = [ 0.0, 3.0, 5.0 ];
  let light_color = [ 1.0, 1.0, 1.0 ];
  let light_orientation = QuatF32::from_euler_xyz( [ -30.0_f32.to_radians(), 0.0, 0.0 ] );
  let near = 0.5;
  let far = 30.0;
  let mut light_source = shadowmap::LightSource::new
  (
    light_pos.into(),
    light_orientation,
    // mat3x3h::orthographic_rh_gl( -5.0, 5.0, -5.0, 5.0, near, far ),
    mat3x3h::perspective_rh_gl( 30.0_f32.to_radians(), 1.0, near, far ),
    0.8
  );


  // === Shadow Pass: Render depth from light's perspective ===
  // Render back faces to shadow map to reduce shadow acne
  // This provides natural geometric offset instead of relying only on depth bias
  shadowmap.bind();
  shadowmap.clear();

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::CULL_FACE );
  gl.cull_face( gl::FRONT );
  let light_view_projection = light_source.view_projection();

  shadowmap.upload_mvp( light_view_projection * mesh_model );
  for mesh in &mesh.meshes
  {
    for primitive in &mesh.borrow().primitives
    {
      let primitive = primitive.borrow();
      primitive.geometry.borrow().bind( &gl );
      primitive.draw( &gl );
    }
  }

  shadowmap.upload_mvp( light_view_projection * plane_model );
  for mesh in &cube_mesh.meshes
  {
    for primitive in &mesh.borrow().primitives
    {
      let primitive = primitive.borrow();
      primitive.geometry.borrow().bind( &gl );
      primitive.draw( &gl );
    }
  }

  gl.cull_face( gl::BACK );
  gl.disable( gl::CULL_FACE );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  shadowmap.generate_mipmaps();


  // === Lightmap Baking Pass: Bake PCSS shadows into lightmap ===
  shadow_renderer.bind( lightmap_res as u32, lightmap_res as u32 );
  shadow_renderer.set_shadowmap( shadowmap.depth_buffer() );
  shadow_renderer.set_target( plane_lightmap.as_ref() );
  shadow_renderer.upload_model( plane_model );
  shadow_renderer.upload_light_source( &mut light_source );

  for mesh in &cube_mesh.meshes
  {
    for primitive in &mesh.borrow().primitives
    {
      let primitive = primitive.borrow();
      primitive.geometry.borrow().bind( &gl );
      primitive.draw( &gl );
    }
  }
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

    gl.bind_texture( gl::TEXTURE_2D, None );

    // Render skull
    gl.enable( gl::CULL_FACE );

    let mesh_mvp = view_projection * mesh_model;
    shader.uniform_matrix_upload( "u_mvp", mesh_mvp.raw_slice(), true );
    shader.uniform_matrix_upload( "u_model", mesh_model.raw_slice(), true );
    shader.uniform_upload( "u_object_color", &mesh_color );
    for mesh in &mesh.meshes
    {
      for primitive in &mesh.borrow().primitives
      {
        let primitive = primitive.borrow();
        primitive.geometry.borrow().bind( &gl );
        primitive.draw( &gl );
      }
    }

    let plane_mvp = view_projection * plane_model;
    shader.uniform_matrix_upload( "u_mvp", plane_mvp.raw_slice(), true );
    shader.uniform_matrix_upload( "u_model", plane_model.raw_slice(), true );
    shader.uniform_upload( "u_object_color", &plane_color );
    gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );
    for mesh in &cube_mesh.meshes
    {
      for primitive in &mesh.borrow().primitives
      {
        let primitive = primitive.borrow();
        primitive.geometry.borrow().bind( &gl );
        primitive.draw( &gl );
      }
    }

    gl.disable( gl::CULL_FACE );

    // === Debug: Visualize lightmap in corner ===
    // Set viewport to bottom-left corner (quarter size)
    let debug_size = ( width / 4 ).min( height / 4 );
    gl.viewport( 0, 0, debug_size, debug_size );
    gl.disable( gl::DEPTH_TEST );

    debug_shader.activate();
    debug_shader.uniform_upload( "u_depth_texture", &0 );
    debug_shader.uniform_upload( "u_near", &near );
    debug_shader.uniform_upload( "u_far", &far );

    // gl.bind_texture( gl::TEXTURE_2D, shadowmap.depth_buffer() );
    gl.bind_texture( gl::TEXTURE_2D, shadowmap.depth_buffer() );
    // gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );
    gl.draw_arrays( gl::TRIANGLES, 0, 3 );

    gl.viewport( 0, 0, width, height );
    gl.enable( gl::DEPTH_TEST );

    true
  };

  gl::exec_loop::run( update );

  Ok( () )
}
