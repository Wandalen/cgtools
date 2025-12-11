//! Renders GLTF files using postprocess effects.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/pbr_lighting/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Shows point light usage in renderer" ) ]

#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::match_like_matches_macro ) ]
#![ allow( clippy::iter_cloned_collect ) ]
#![ allow( clippy::collapsible_if ) ]
#![ allow( clippy::needless_range_loop ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::match_wildcard_for_single_variants ) ]
#![ allow( clippy::iter_overeager_cloned ) ]
#![ allow( clippy::collapsible_match ) ]

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

fn add_light( scene : &Rc< RefCell< Scene > >, light : Light ) -> Rc< RefCell< Node > >
{
  let light_node = Rc::new( RefCell::new( Node::new() ) );
  light_node.borrow_mut().object = Object3D::Light( light );
  scene.borrow_mut().children.push( light_node.clone() );
  light_node
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf_path = "2017_porsche_911_turbo_s_exclusive_series_991.2.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scene = gltf.scenes[ 0 ].clone();
  scene.borrow_mut().update_world_matrix();

  for node in &scene.borrow().children
  {
    let scale = node.borrow_mut().get_scale();
    node.borrow_mut().set_scale( scale * 40.0 );
  }

  let scene_bounding_box = scene.borrow().bounding_box();
  gl::info!( "Scene boudnig box: {:?}", scene_bounding_box );
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag() * 40.0;
  let exponent =
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };
  gl::info!( "Exponent: {:?}", exponent );

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
  camera.bind_controls( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_use_emission( true );
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "envMap" ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let sphere_gltf = renderer::webgl::loaders::gltf::load( &document, "sphere.glb", &gl ).await?;
  let sphere = sphere_gltf.scenes[ 0 ].borrow().children.last().cloned().unwrap();

  let mut lights = vec![];

  let colors =
  [
    F32x3::from_array( [ 1.0, 0.0, 0.0 ] ),
    F32x3::from_array( [ 0.0, 1.0, 0.0 ] ),
    F32x3::from_array( [ 0.0, 0.0, 1.0 ] ),
  ];

  for i in 0..3
  {
    let d = add_light
    (
      &scene,
      Light::Direct
      (
        DirectLight
        {
          direction : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
          color : colors[ i ],
          strength : 0.0
        }
      )
    );

    lights.push( d );
  }

  for i in 0..3
  {
    let p = add_light
    (
      &scene,
      Light::Point
      (
        PointLight
        {
          position : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
          color : colors[ i ],
          strength : 0.0,
          range : 10.0
        }
      )
    );

    lights.push( p );
  }

  let controllable_light = add_light
  (
    &scene,
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

  sphere.borrow_mut().set_scale( F32x3::splat( 0.02 ) );

  let spheres = lights.iter()
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
      };

      let sphere_clone = sphere.borrow().clone_tree();
      sphere_clone.borrow_mut().set_translation( position );

      Some( sphere_clone )
    }
  )
  .collect::< Vec< _ > >();

  let controllable_sphere = sphere.borrow().clone_tree();
  controllable_sphere.borrow_mut().set_translation( F32x3::splat( 1.0 ) );

  scene.borrow_mut().children.extend_from_slice( &lights );
  scene.borrow_mut().children.extend_from_slice( &spheres );
  scene.borrow_mut().add( controllable_sphere.clone() );

  let _settings = gui_setup::setup( renderer.clone(), lights.clone(), controllable_light.clone() ).unwrap();

  let light_radius = 1.0;
  let light_speed = 50.0;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      if let Object3D::Light( light ) = &controllable_light.borrow().object
      {
        let position = match light
        {
          Light::Point( point_light ) => point_light.position,
          Light::Direct( direct_light ) => direct_light.direction,
        };

        controllable_sphere.borrow_mut().set_translation( position );
      }

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
                mingl::Spherical
                {
                  radius : light_radius,
                  theta : i as f32 * 120.0 + ( t as f32 * light_speed / 1000.0 ),
                  phi : 45.0
                }
              );

              direct.direction = direction;
              spheres[ i ].borrow_mut().set_translation( direction );
            },
            Light::Point( point ) =>
            {
              let position = F32x3::from_spherical
              (
                mingl::Spherical
                {
                  radius : light_radius,
                  theta : i as f32 * 120.0 + ( t as f32 * light_speed / 1000.0 ),
                  phi : 45.0
                }
              );

              point.position = position;
              spheres[ i ].borrow_mut().set_translation( position );
            }
          }
        }
      }

      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

      renderer.borrow_mut().render( &gl, &mut scene.borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.borrow().get_main_texture() );

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
  gl::spawn_local( async move { run().await.unwrap() } );
}
