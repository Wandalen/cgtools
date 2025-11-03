//! Simple skull rendering with basic lighting and shadowmapping

use minwebgl as gl;
use gl::{ JsCast as _, math::mat3x3h, QuatF32 };
use web_sys::HtmlCanvasElement;
use renderer::webgl::loaders::gltf;
use renderer::webgl::{ Object3D, shadow::* };

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

  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );

  let canvas = gl.canvas()
  .unwrap()
  .dyn_into::< HtmlCanvasElement >()
  .unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  let ext_color_buffer_float = gl.get_extension( "EXT_color_buffer_float" );
  let ext_float_linear = gl.get_extension( "OES_texture_float_linear" );
  gl::info!( "{ext_color_buffer_float:?}" );
  gl::info!( "{ext_float_linear:?}" );

  let mut camera = renderer::webgl::Camera::new
  (
    [ 0.0, 1.5, 5.0 ].into(),
    [ 0.0, 1.0, 0.0 ].into(),
    [ 0.0, 0.0, 0.0 ].into(),
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

  // let debug_vert_src = include_str!( "shaders/debug_shadowmap.vert" );
  // let debug_frag_src = include_str!( "shaders/debug_shadowmap.frag" );
  // let debug_shader = gl::Program::new( gl.clone(), debug_vert_src, debug_frag_src )?;

  let mesh = gltf::load( &document, "skull_salazar_downloadable.glb", &gl ).await?;

  let cube_mesh = gltf::load( &document, "cube.glb", &gl ).await?;
  let cube_model = mat3x3h::translation( [ 0.0, -1.0, 0.0 ] )
    * mat3x3h::scale( [ 8.0, 0.25, 8.0 ] );

  let mut main_scene = renderer::webgl::Scene::new();

  for scene in mesh.scenes
  {
    let mut scene = scene.borrow_mut();
    for node in core::mem::take( &mut scene.children )
    {
      main_scene.add( node );
    }
  }

  for scene in cube_mesh.scenes
  {
    let mut scene = scene.borrow_mut();
    for node in core::mem::take( &mut scene.children )
    {
      node.borrow_mut().set_world_matrix( cube_model );
      main_scene.add( node );
    }
  }

  _ = main_scene.traverse( &mut | node | {
    node.borrow_mut().is_shadow_caster = true;
    node.borrow_mut().is_shadow_receiver = true;
    Ok( () )
  } );

  let mesh_color = [ 0.8, 0.75, 0.7 ];

  let light_pos = [ 0.0, 2.0, 4.0 ];
  let light_color = [ 1.0, 1.0, 1.0 ];
  let light_orientation = QuatF32::from_euler_xyz( [ -30.0_f32.to_radians(), 0.0, 0.0 ] );
  let near = 1.0;
  let far = 30.0;
  let mut light = Light::new
  (
    light_pos.into(),
    light_orientation,
    // mat3x3h::orthographic_rh_gl( -5.0, 5.0, -5.0, 5.0, near, far ),
    mat3x3h::perspective_rh_gl( 30.0_f32.to_radians(), 1.0, near, far ),
    0.05
  );

  let shadowmap_res = 4096;
  let lightmap_res = 4096;
  bake_shadows( &gl, &main_scene, &mut light, lightmap_res, shadowmap_res ).unwrap();

  gl.active_texture( gl::TEXTURE0 );
  gl.enable( gl::CULL_FACE );
  gl.cull_face( gl::BACK );
  gl.enable( gl::DEPTH_TEST );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  gl.viewport( 0, 0, width, height );
  gl.clear_color( 0.1, 0.1, 0.15, 1.0 );

  let update = move | _ |
  {
    gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );

    let view = camera.get_view_matrix();
    let projection = camera.get_projection_matrix();
    let view_projection = projection * view;

    shader.activate();
    shader.uniform_upload( "u_light_pos", &light_pos );
    shader.uniform_upload( "u_view_pos", camera.get_eye().as_slice() );
    shader.uniform_upload( "u_light_color", &light_color );

    _ = main_scene.traverse( &mut | node | {
      let node = node.borrow();

      if let Object3D::Mesh( mesh ) = &node.object
      {
        let model = node.get_world_matrix();
        let mesh_mvp = view_projection * model;

        shader.uniform_matrix_upload( "u_mvp", mesh_mvp.raw_slice(), true );
        shader.uniform_matrix_upload( "u_model", model.raw_slice(), true );
        shader.uniform_upload( "u_object_color", &mesh_color );

        for primitive in &mesh.borrow().primitives
        {
          let primitive = primitive.borrow();
          primitive.material.borrow().light_map.as_ref().unwrap().bind( &gl );
          primitive.geometry.borrow().bind( &gl );
          primitive.draw( &gl );
        }
      }

      Ok( () )
    } );

    // Set viewport to bottom-left corner (quarter size)
    // let debug_size = ( width / 4 ).min( height / 4 );
    // gl.viewport( 0, 0, debug_size, debug_size );
    // gl.disable( gl::DEPTH_TEST );

    // debug_shader.activate();
    // debug_shader.uniform_upload( "u_depth_texture", &0 );
    // debug_shader.uniform_upload( "u_near", &near );
    // debug_shader.uniform_upload( "u_far", &far );

    // gl.bind_texture( gl::TEXTURE_2D, shadowmap.depth_buffer() );
    // gl.bind_texture( gl::TEXTURE_2D, plane_lightmap.as_ref() );
    // gl.draw_arrays( gl::TRIANGLES, 0, 3 );

    // gl.viewport( 0, 0, width, height );
    // gl.enable( gl::DEPTH_TEST );

    true
  };

  gl::exec_loop::run( update );

  Ok( () )
}
