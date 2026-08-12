//! Renders GLTF files using postprocess effects.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/pbr_lighting/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Shows point light usage in renderer" ) ]

use std::{ cell::RefCell, rc::Rc };
use mingl::F32x3;
use minwebgl as gl;
use renderer::webgl::
{
  post_processing::{self, Pass, SwapFramebuffer},
  Camera,
  Scene,
  DirectLight,
  PointLight,
  Light,
  Node,
  Object3D,
  Renderer
};

mod lil_gui;
mod gui_setup;

fn light_add( scene : &Rc< RefCell< Scene > >, light : Light ) -> Rc< RefCell< Node > >
{
  let light_node = Rc::new( RefCell::new( Node::new() ) );
  light_node.borrow_mut().object = Object3D::Light( light );
  scene.borrow_mut().children.push( light_node.clone() );
  light_node
}

/// Creates the orbit camera framed on the scene's bounding box and binds its controls.
fn camera_setup( canvas : &gl::web_sys::HtmlCanvasElement, scene_bounding_box : &mingl::geometry::BoundingBox, width : f32, height : f32 ) -> Camera
{
  gl::info!( "Scene boudnig box: {scene_bounding_box:?}" );
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag() * 40.0;
  let exponent =
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };
  gl::info!( "Exponent: {exponent:?}" );

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  //eye = gl::math::F32x3::from( [ 0.0, 0.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 ) * 100.0;
  let far = near * 100.0f32.powi( exponent.abs() );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.bind_controls( canvas );

  camera
}

/// Adds the three rotating direct lights, three rotating point lights, and the named controllable light to the scene.
fn lights_create( scene : &Rc< RefCell< Scene > > ) -> ( Vec< Rc< RefCell< Node > > >, Rc< RefCell< Node > > )
{
  let mut lights = vec![];

  let colors =
  [
    F32x3::from_array( [ 1.0, 0.0, 0.0 ] ),
    F32x3::from_array( [ 0.0, 1.0, 0.0 ] ),
    F32x3::from_array( [ 0.0, 0.0, 1.0 ] ),
  ];

  for color in colors
  {
    let d = light_add
    (
      scene,
      Light::Direct
      (
        DirectLight
        {
          direction : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
          color,
          strength : 0.0
        }
      )
    );

    lights.push( d );
  }

  for color in colors
  {
    let p = light_add
    (
      scene,
      Light::Point
      (
        PointLight
        {
          position : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
          color,
          strength : 0.0,
          range : 10.0
        }
      )
    );

    lights.push( p );
  }

  let controllable_light = light_add
  (
    scene,
    Light::Direct
    (
      DirectLight
      {
        direction : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
        color : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
        strength : 0.0
      }
    )
  );
  controllable_light.borrow_mut().set_name( "controllable" );

  ( lights, controllable_light )
}

/// Clones the marker sphere onto every light's position or direction.
fn light_spheres_create( sphere : &Rc< RefCell< Node > >, lights : &[ Rc< RefCell< Node > > ] ) -> Vec< Rc< RefCell< Node > > >
{
  lights.iter()
  .filter_map
  (
    | node |
    {
      let node = node.borrow();
      let Object3D::Light( light ) = &node.object
      else
      {
        return None;
      };
      let position = match light
      {
        Light::Point( point_light ) => point_light.position,
        Light::Direct( direct_light ) => direct_light.direction,
        Light::Spot( _ ) => return None
      };

      let sphere_clone = sphere.borrow().clone_tree();
      sphere_clone.borrow_mut().set_translation( position );

      Some( sphere_clone )
    }
  )
  .collect::< Vec< _ > >()
}

/// Rotates every non-controllable light around the scene and moves its marker sphere with it.
fn lights_animate( lights : &[ Rc< RefCell< Node > > ], spheres : &[ Rc< RefCell< Node > > ], t : f64, light_radius : f32, light_speed : f32 )
{
  for ( i, light ) in lights.iter().enumerate()
  {
    if let Some( name ) = light.borrow().get_name()
    {
      if name.to_string().as_str() == "controllable"
      {
        continue;
      }
    }
    if let Object3D::Light( light ) = &mut light.borrow_mut().object
    {
      match light
      {
        Light::Direct( direct ) =>
        {
          let direction = F32x3::from_spherical
          (
            light_radius,
            i as f32 * 120.0 + ( t as f32 * light_speed / 1000.0 ),
            45.0
          );

          direct.direction = direction;
          spheres[ i ].borrow_mut().set_translation( direction );
        },
        Light::Point( point ) =>
        {
          let position = F32x3::from_spherical
          (
            light_radius,
            i as f32 * 120.0 + ( t as f32 * light_speed / 1000.0 ),
            45.0
          );

          point.position = position;
          spheres[ i ].borrow_mut().set_translation( position );
        }
        Light::Spot( _ ) => ()
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

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf_path = "static/2017_porsche_911_turbo_s_exclusive_series_991.2.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scene = gltf.scenes[ 0 ].clone();
  scene.borrow_mut().update_world_matrix();

  for node in &scene.borrow().children
  {
    let scale = node.borrow_mut().get_scale();
    node.borrow_mut().set_scale( scale * 40.0 );
  }

  let scene_bounding_box = scene.borrow().bounding_box();
  let camera = camera_setup( &canvas, &scene_bounding_box, width, height );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_use_emission( &gl, true );
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "static/envMap", Some( 0..0 ) ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let sphere_gltf = renderer::webgl::loaders::gltf::load( &document, "static/sphere.glb", &gl ).await?;
  let sphere = sphere_gltf.scenes[ 0 ].borrow().children.last().cloned().unwrap();

  let ( lights, controllable_light ) = lights_create( &scene );

  sphere.borrow_mut().set_scale( F32x3::splat( 0.02 ) );

  let spheres = light_spheres_create( &sphere, &lights );

  let controllable_sphere = sphere.borrow().clone_tree();
  controllable_sphere.borrow_mut().set_translation( F32x3::splat( 1.0 ) );

  scene.borrow_mut().children.extend_from_slice( &lights );
  scene.borrow_mut().children.extend_from_slice( &spheres );
  scene.borrow_mut().add( controllable_sphere.clone() );

  let _settings = gui_setup::setup( &renderer, lights.clone(), &controllable_light ).unwrap();

  let light_radius = 1.0;
  let light_speed = 50.0;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      if let Object3D::Light( light ) = &controllable_light.borrow().object
      {
        if let Some( position ) = match light
        {
          Light::Point( point_light ) => Some( point_light.position ),
          Light::Direct( direct_light ) => Some( direct_light.direction ),
          Light::Spot( _ ) => None
        }
        {
          controllable_sphere.borrow_mut().set_translation( position );
        }
      }

      lights_animate( &lights, &spheres, t, light_radius, light_speed );

      // If textures are of different size, gl.view_port needs to be called

      renderer.borrow_mut().render( &gl, &mut scene.borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.borrow().main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.set_output( t );
      swap_buffer.swap();

      let _ = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
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
