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

/// Computes ( near, far ) clip-plane distances from `exponent` ( the scene bounding-box
/// diagonal's base-2 exponent ), scaling `near` down for smaller scenes while guaranteeing
/// `far` always clears `near` by a safe margin.
fn near_far_from_exponent( exponent : i32 ) -> ( f32, f32 )
{
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 ) * 100.0;
  // Fix(BUG-332): `far`'s raw formula collapses to `far == near` at `exponent == 0`
  // ( `100.0f32.powi( 0 ) == 1.0` ), which `Camera::new` rejects ( requires `far > near` ),
  // panicking the whole demo for any scene whose bounding-box diagonal falls in [ 1.0, 2.0 )
  // -- an ordinary size for a normalized glTF asset.
  // Root cause: `100.0f32.powi( exponent.abs() )` is V-shaped in `exponent` ( `.abs()` makes
  // it shrink toward `exponent == 0` from both sides ) instead of monotonically increasing
  // with scene size, so the multiplier can collapse to 1.0 and erase the near/far margin.
  // Pitfall: deriving `far` as a pure multiple of `near` with no floor lets that multiplier
  // silently erase the required margin at whatever exponent makes it collapse, instead of
  // failing loudly at the formula's own definition site.
  let far = ( near * 100.0f32.powi( exponent.abs() ) ).max( near * 10.0 );
  ( near, far )
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
  let ( near, far ) = near_far_from_exponent( exponent );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far ).expect( "camera parameters are valid" );
  camera.window_size_set( [ width, height ].into() );
  camera.controls_bind( canvas );

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
  controllable_light.borrow_mut().name_set( "controllable" );

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

      let sphere_clone = sphere.borrow().tree_clone();
      sphere_clone.borrow_mut().translation_set( position );

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
    if let Some( name ) = light.borrow().name_get()
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
          spheres[ i ].borrow_mut().translation_set( direction );
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
          spheres[ i ].borrow_mut().translation_set( position );
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
  scene.borrow_mut().world_matrix_update();

  for node in &scene.borrow().children
  {
    let scale = node.borrow_mut().scale_get();
    node.borrow_mut().scale_set( scale * 40.0 );
  }

  let scene_bounding_box = scene.borrow().bounding_box();
  let camera = camera_setup( &canvas, &scene_bounding_box, width, height );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.use_emission_set( &gl, true );
  renderer.ibl_set( renderer::webgl::loaders::ibl::load( &gl, "static/envMap", Some( 0..0 ) ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let sphere_gltf = renderer::webgl::loaders::gltf::load( &document, "static/sphere.glb", &gl ).await?;
  let sphere = sphere_gltf.scenes[ 0 ].borrow().children.last().cloned().unwrap();

  let ( lights, controllable_light ) = lights_create( &scene );

  sphere.borrow_mut().scale_set( F32x3::splat( 0.02 ) );

  let spheres = light_spheres_create( &sphere, &lights );

  let controllable_sphere = sphere.borrow().tree_clone();
  controllable_sphere.borrow_mut().translation_set( F32x3::splat( 1.0 ) );

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
          controllable_sphere.borrow_mut().translation_set( position );
        }
      }

      lights_animate( &lights, &spheres, t, light_radius, light_speed );

      // If textures are of different size, gl.view_port needs to be called

      renderer.borrow_mut().render( &gl, &mut scene.borrow_mut(), &camera )
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

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// ## Root Cause
  /// `near_far_from_exponent`'s `far` formula ( `near * 100.0f32.powi( exponent.abs() )` )
  /// used `exponent.abs()`, making the scaling multiplier V-shaped around `exponent == 0`
  /// instead of monotonically increasing with scene size. At `exponent == 0` the multiplier
  /// collapses to exactly `1.0`, so `far` equals `near` -- a degenerate frustum `Camera::new`
  /// rejects ( it requires `far > near` ), which `camera_setup`'s `.expect( "camera
  /// parameters are valid" )` then turns into a panic.
  ///
  /// ## Why Not Caught
  /// This crate has no test file -- it is a `fn main()`-only WebGL demo binary, verified by
  /// running it in a browser against whatever glTF asset happens to be loaded. The panic only
  /// triggers when the loaded scene's bounding-box diagonal falls in `[ 1.0, 2.0 )`, a plausible
  /// but not-yet-exercised size for this demo's own asset.
  ///
  /// ## Fix Applied
  /// Extracted the near/far computation into `near_far_from_exponent`, keeping `near`'s
  /// formula unchanged and wrapping `far` in `.max( near * 10.0 )` -- a floor guaranteeing a
  /// minimum 10x margin regardless of what the exponent-scaled term evaluates to.
  ///
  /// ## Prevention
  /// This test sweeps every exponent in a wide, representative range and asserts `far > near`
  /// unconditionally, rather than checking only the one exponent that happened to break.
  ///
  /// ## Pitfall
  /// A multiplier derived from the same shared input as the value it scales can collapse to a
  /// value that erases the relationship the caller depends on ( here, `far > near` ) -- always
  /// floor/ceiling such a derived value against its sibling rather than trusting the formula's
  /// shape to hold across the whole input domain.
  // Fix(BUG-332): reproducer for `far == near` at `exponent == 0`, rejected by `Camera::new`.
  // test_kind: bug_reproducer(BUG-332)
  #[ test ]
  fn test_far_always_exceeds_near_across_exponent_range()
  {
    for exponent in -10_i32 ..= 10_i32
    {
      let ( near, far ) = near_far_from_exponent( exponent );
      assert!( near.is_finite() && near > 0.0, "near must be finite and positive at exponent {exponent}" );
      assert!( far.is_finite() && far > near, "far ({far}) must exceed near ({near}) at exponent {exponent}" );
    }
  }

  /// Pins the pre-fix formula's exact failure at `exponent == 0` ( `far == near == 10.0` ),
  /// confirming the bug was real and not a hypothetical edge case.
  #[ test ]
  fn test_pre_fix_formula_was_degenerate_at_exponent_zero()
  {
    let exponent = 0_i32;
    let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 ) * 100.0;
    let buggy_far = near * 100.0f32.powi( exponent.abs() );
    // Compared as `i32`, not raw `f32` equality ( clippy::float_cmp ) -- exact for these
    // whole-number inputs, matching this crate family's established BUG-313 precedent.
    assert_eq!( buggy_far as i32, near as i32, "pre-fix formula collapsed far to exactly near at exponent 0" );

    let ( _, fixed_far ) = near_far_from_exponent( exponent );
    assert!( fixed_far > near, "fixed formula must clear the degenerate case" );
  }
}
