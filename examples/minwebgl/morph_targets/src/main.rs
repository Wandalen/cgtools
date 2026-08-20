//! Renders skeletal animations with morph targets.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/skeletal_animation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders skeletal animations with morph targets" ) ]

use std::{ cell::RefCell, rc::Rc };
use mingl::F32x3;
use minwebgl as gl;
use renderer::webgl::
{
  post_processing::
  {
    self,
    Pass,
    SwapFramebuffer
  },
  Camera,
  Mesh,
  Renderer,
  Scene
};

mod lil_gui;
mod gui_setup;

/// Rescales the named mesh nodes that come in at the wrong unit scale.
fn named_nodes_rescale( scene : &Rc< RefCell< Scene > > )
{
  let need_rescale = [ "Head_Mesh", "Object_7", "Object_6" ];
  let _ = scene.borrow()
  .traverse
  (
    &mut | node |
    {
      let name = node.borrow().name_get().unwrap_or( "<none>".into() );

      if need_rescale.contains( &name.to_string().as_str() )
      {
        node.borrow_mut().scale_set( F32x3::splat( 68.0 ) );
      }

      Ok( () )
    }
  );

  scene.borrow_mut().world_matrix_update();
}

/// Creates the scene camera from the scene's bounding box and binds its controls to the canvas.
fn camera_setup( canvas : &gl::web_sys::HtmlCanvasElement, scene : &Rc< RefCell< Scene > > ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let scene_bounding_box = scene.borrow().bounding_box();
  gl::info!( "Scene boudnig box: {scene_bounding_box:?}" );

  // Camera setup: frames the scene's bounding sphere from the (0,0.1,1) direction, deriving
  // distance/near/far from the box itself and the camera's own fov/aspect_ratio.
  let direction = gl::math::F32x3::from( [ 0.0, 0.1, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 40.0f32.to_radians();

  let mut camera = Camera::from_bounding_box( &scene_bounding_box, direction, up, aspect_ratio, fov, 0.1 ).expect( "camera parameters are valid" );
  camera.window_size_set( [ width, height ].into() );
  camera.controls_bind( canvas );

  camera
}

/// Finds the first skeleton with morph displacements and returns its shared morph weights, initialized to the defaults.
fn morph_weights_find( meshes : &[ Rc< RefCell< Mesh > > ] ) -> Rc< RefCell< Vec< f32 > > >
{
  meshes.iter()
  .find_map
  (
    | m |
    {
      let m_ref = m.borrow();
      let s = m_ref.skeleton.as_ref()?;
      let s_ref = s.borrow();
      let Some( d ) = s_ref.displacements_as_ref()
      else
      {
        return None;
      };
      let weights = d.morph_weights_get();
      ( *weights.borrow_mut() ).clone_from( &d.default_weights );
      Some( weights )
    }
  )
  .unwrap()
}

/// Clears the normal displacement bindings so only position morphs apply.
fn normal_displacements_reset( meshes : &[ Rc< RefCell< Mesh > > ] )
{
  for mesh in meshes
  {
    if let Some( skeleton ) = &mesh.borrow().skeleton
    {
      if let Some( d ) = skeleton.borrow_mut().displacements_as_mut()
      {
        d.displacement_set( None, &gltf::Semantic::Normals, 0 );
      }
    }
  }
}

async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let gltf_path = "static/gltf/zophrac.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;

  named_nodes_rescale( &scenes[ 0 ] );

  let camera = camera_setup( &canvas, &scenes[ 0 ] );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.ibl_set( renderer::webgl::loaders::ibl::load( &gl, "static/envMap", None ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  camera.controls_get().borrow_mut().center.0[ 1 ] += -5.5;
  camera.controls_get().borrow_mut().center.0[ 2 ] += -2.0;

  let weights = morph_weights_find( &gltf.meshes );

  normal_displacements_reset( &gltf.meshes );

  // Fix(BUG-330): filled with 0.0, matching a slider the user has actively dragged down to its
  // minimum — `gui_weights[i] > 0.0` then treats "untouched" and "explicitly zeroed" identically,
  // so once a slider is raised above 0 it can never be reset back to 0 via that same slider.
  // Root cause: used the current value's sign as a proxy for "has this slider been touched",
  // conflating a real, meaningful value (0.0) with the sentinel for "no GUI override yet".
  // Pitfall: a min-of-range value (here, a slider's own minimum, 0.0) makes a poor "untouched"
  // sentinel whenever that same value is also a legitimate, settable state.
  let gui_weights = Rc::new( RefCell::new( vec![ f32::NAN; 60 ] ) );

  let last_time = Rc::new( RefCell::new( 0.0 ) );

  let current_animation = Rc::new( RefCell::new( Some( gltf.animations[ 0 ].clone() ) ) );

  gui_setup::setup( gltf.animations.clone(), &current_animation, &gui_weights );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = t / 1000.0;

      {
        let last_time = last_time.clone();

        let delta_time = time - *last_time.borrow();
        *last_time.borrow_mut() = time;

        if let Some( animation ) = current_animation.borrow_mut().as_mut()
        {
          if animation.inner_get::< animation::Sequencer >().unwrap().is_completed()
          {
            animation
            .inner_get_mut::< animation::Sequencer >()
            .unwrap()
            .reset();
          }

          animation.update( delta_time );
          animation.set();
        }
        else
        {
          weights.borrow_mut().fill( 0.0 );
        }

        let mut weights_mut = weights.borrow_mut();
        let gui_weights = gui_weights.borrow().clone();
        for i in 0..weights_mut.len().min( gui_weights.len() )
        {
          if !gui_weights[ i ].is_nan()
          {
            weights_mut[ i ] = gui_weights[ i ];
          }
        }
      }

      renderer.borrow_mut().render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.input_set( renderer.borrow().main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.output_set( t );
      swap_buffer.swap();

      let _ = to_srgb.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
      .expect( "Failed to render ToSrgbPass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}
