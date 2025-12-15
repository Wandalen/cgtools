//! Simple rendering with PBR lighting and shadowmapping

use minwebgl as gl;
use gl::{ JsCast as _, math::mat3x3h, GL };
use web_sys::HtmlCanvasElement;
use std::rc::Rc;
use core::cell::RefCell;
use renderer::webgl::{ Light, Node, Object3D, SpotLight, loaders::gltf, post_processing, shadow::{self, ShadowBaker, ShadowMap} };
use post_processing::{ Pass, SwapFramebuffer };

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

  let mut renderer = renderer::webgl::Renderer::new( &gl, width as u32, height as u32, 4 )?;
  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;
  let mut swap_buffer = SwapFramebuffer::new( &gl, width as u32, height as u32 );

  let mesh = gltf::load( &document, "skull_salazar_downloadable.glb", &gl ).await?;

  // let cube_mesh = gltf::load( &document, "cube.glb", &gl ).await?;
  let cube_mesh = gltf::load( &document, "plane.glb", &gl ).await?;
  let cube_model = mat3x3h::translation( [ 0.0, -1.0, 0.0 ] )
    * mat3x3h::scale( [ 8.0, 1.0, 8.0 ] );

  let mut main_scene = renderer::webgl::Scene::new();

  for scene in mesh.scenes
  {
    let mut scene = scene.borrow_mut();
    for node in core::mem::take( &mut scene.children )
    {
      main_scene.add( node );
    }
  }
  main_scene.update_world_matrix();

  for scene in cube_mesh.scenes
  {
    let mut scene = scene.borrow_mut();
    for node in core::mem::take( &mut scene.children )
    {
      node.borrow_mut().set_local_matrix( cube_model );
      main_scene.add( node );
    }
  }

  let light_pos = gl::F32x3::from_array( [ 0.0, 3.0, 3.0 ] );
  let light_dir = gl::F32x3::from_array( [ 0.0, -1.0, -1.0 ] ).normalize();

  let mut node = Node::new();
  node.object = Object3D::Light
  (
    Light::Spot
    (
      SpotLight
      {
        position : light_pos,
        direction : light_dir,
        color : [ 1.0, 1.0, 1.0 ].into(),
        strength : 300.0,
        range : 100.0,
        inner_cone_angle : 40.0_f32.to_radians(),
        outer_cone_angle : 60.0_f32.to_radians(),
        use_light_map : true
      }
    )
  );
  main_scene.add( Rc::new( RefCell::new( node ) ) );

  _ = main_scene.traverse
  (
    &mut | node |
    {
      let node = node.borrow();
      if let Object3D::Mesh( mesh ) = &node.object
      {
        let mut mesh = mesh.borrow_mut();
        mesh.is_shadow_caster = true;
        // mesh.is_shadow_receiver = true;
      }
      Ok( () )
    }
  );

  let near = 0.1;
  let far = 30.0;
  let light = shadow::Light::new
  (
    light_pos,
    light_dir,
    mat3x3h::perspective_rh_gl( 60.0_f32.to_radians(), 1.0, near, far ),
    0.5
  );

  let shadowmap_res = 2048;
  let lightmap_res = 2048;
  let shadowmap = ShadowMap::new( &gl, shadowmap_res )?;
  shadowmap.render( &main_scene, light )?;

  let shadow_baker = ShadowBaker::new( &gl )?;
  // shadow_baker.render_soft_shadow_texture( node, texture, width, height, shadowmap, light);
  // shadow::render_soft_shadow_texture( &gl, &main_scene, &mut light, lightmap_res, shadowmap_res ).unwrap();

  let update = move | _ |
  {
    renderer.render( &gl, &mut main_scene, &camera ).expect( "Failed to render" );

    swap_buffer.reset();
    swap_buffer.bind( &gl );
    swap_buffer.set_input( renderer.get_main_texture() );

    let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
    .expect( "Failed to render tonemapping pass" );

    swap_buffer.set_output( t );
    swap_buffer.swap();

    let _ = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
    .expect( "Failed to render ToSrgbPass" );

    true
  };

  gl::exec_loop::run( update );

  Ok( () )
}

fn create_texture( gl : &GL, lightmap_res : u32, mip_levels : i32 ) -> Option< web_sys::WebGlTexture >
{
  let light_map = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, light_map.as_ref() );
  gl.tex_storage_2d( gl::TEXTURE_2D, mip_levels, gl::R16F, lightmap_res as i32, lightmap_res as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
  gl::texture::d2::wrap_clamp( &gl );


  light_map
}
