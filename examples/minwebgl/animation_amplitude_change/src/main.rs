//! Renders skeletal animations with opportunity to change its amplitude.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/skeletal_animation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders skeletal animations with opportunity to change its amplitude" ) ]

use std::collections::{ HashMap, HashSet };
use std::{ cell::RefCell, rc::Rc };
use std::fmt::Write as _;
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
  Renderer,
  Node,
  animation::AnimatableComposition
};

mod lil_gui;
mod gui_setup;

fn tree_write( node : &Rc< RefCell< Node > >, depth : usize, output : &mut String )
{
  let name = node
  .borrow()
  .name_get()
  .unwrap_or( "<none>".into() );

  let indent = "-".repeat( depth );
  let _ = writeln!( output, "{indent}{name}" );

  for child in node.borrow().children_get()
  {
    tree_write( child, depth + 1, output );
  }
}

fn tree_print( node : &Rc< RefCell< Node > > )
{
  let mut tree_str = String::new();
  tree_write( node, 1, &mut tree_str );
  gl::info!( "{tree_str}" );
}

/// Splits root sub [`Node`]s names into named subtrees
/// Not mentioned nodes from root subnodes in parts
/// argument list will be added as separated node names group
fn node_names_split_into_parts
(
  root : &Rc< RefCell< Node > >,
  part_names : &[ &str ]
)
-> HashMap< Box< str >, Vec< Box< str > > >
{
  fn names_collect( node : &Rc< RefCell< Node > >, out : &mut Vec< Box< str > > )
  {
    let Some( name ) = node.borrow().name_get()
    else
    {
      return;
    };

    out.push( name );
    for child in node.borrow().children_get()
    {
      names_collect( child, out );
    }
  }

  let part_names = part_names.iter()
  .map( | n | (*n).into() )
  .collect::< HashSet< Box< str > > >();
  let mut not_mentioned = HashSet::new();

  let mut parts = HashMap::new();

  let _ = root.borrow()
  .traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let Some( name ) = node.borrow().name_get()
      else
      {
        return Ok( () );
      };

      not_mentioned.insert( name );

      Ok( () )
    }
  );

  let _ = root.borrow()
  .traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let Some( name ) = node.borrow().name_get()
      else
      {
        return Ok( () );
      };

      let mut part = vec![];
      if part_names.contains( &name )
      {
        names_collect( &node, &mut part );
      }
      else
      {
        return Ok( ( ) );
      }

      not_mentioned.retain( | n | !part.contains( n ) );
      parts.insert( name, part );

      Ok( () )
    }
  );

  parts.insert
  (
    root.borrow().name_get().unwrap_or( "<none>".into() ),
    not_mentioned.into_iter().collect::< Vec< _ > >()
  );

  parts
}

/// Creates the orbit camera framed on the scene's bounding box and binds its controls.
fn camera_setup( canvas : &gl::web_sys::HtmlCanvasElement, scene_bounding_box : &mingl::geometry::BoundingBox, width : f32, height : f32 ) -> Camera
{
  gl::info!( "Scene boudnig box: {scene_bounding_box:?}" );
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag();

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  // Fix(BUG-320): `near`/`far` used to be derived from a scale value read out of the raw
  // IEEE-754 bit layout of `diagonal` — a base-2 quantity by construction — then fed into
  // a base-10 power function. That base mismatch, combined with a `far` formula that
  // isn't monotonically greater than `near` across its own input domain, collapsed to
  // `far <= near` for an ordinary scene scale ( including this crate's own bundled
  // `multi_animation.glb` ), which `Camera::new` rejects outright ( `far` must be
  // strictly greater than `near` ), panicking this demo's own
  // `.expect( "camera parameters are valid" )` at startup.
  // Root cause: a base-2-derived scale factor paired with a base-10 power computation.
  // Pitfall: don't reintroduce raw floating-point bit-layout inspection here — `f32::log10`
  // gives the scene's true base-10 order of magnitude directly, and a fixed `far`/`near`
  // ratio ( here 1_000_000 ) guarantees `far > near` for every finite positive `diagonal`.
  let magnitude = diagonal.max( f32::EPSILON ).log10().floor();
  let scale = 10.0f32.powf( magnitude );
  let near = ( scale * 0.01 ).max( 1e-5 );
  let far = scale * 10_000.0;
  gl::info!( "near: {near:?}, far: {far:?}" );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far ).expect( "camera parameters are valid" );
  camera.window_size_set( [ width, height ].into() );
  camera.controls_bind( canvas );

  camera
}

/// Groups the skeleton's node names into named body-part lists for the scaler.
fn parts_assemble( root : &Rc< RefCell< Node > > ) -> HashMap< Box< str >, Vec< Box< str > > >
{
  let parts = vec!
  [
    "mixamorig:Neck",
    "mixamorig:RightShoulder",
    "mixamorig:LeftShoulder",
    "mixamorig:RightUpLeg",
    "mixamorig:LeftUpLeg"
  ];

  let mut parts = node_names_split_into_parts( root, &parts );

  let mut hands = parts.remove( "mixamorig:RightShoulder" ).unwrap();
  hands.extend( parts.remove( "mixamorig:LeftShoulder" ).unwrap() );

  parts.insert( "hands".into(), hands );

  let mut legs = parts.remove( "mixamorig:RightUpLeg" ).unwrap();
  legs.extend( parts.remove( "mixamorig:LeftUpLeg" ).unwrap() );

  parts.insert( "legs".into(), legs );

  let mut replace_key = | key : &str, new_key : &str |
  {
    if let Some( nodes ) = parts.remove::< Box< str > >( &key.into() )
    {
      parts.insert( new_key.into(), nodes );
    }
  };

  replace_key( "mixamorig:Neck", "head" );
  replace_key( "Armature", "body" );

  gl::info!( "{parts:#?}" );

  parts
}

async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  // Fix(BUG-453): chained a second `.expect()` onto the inner `Option`, matching
  // `area_light/src/main.rs`'s existing 2-layer pattern.
  // Root cause: `get_extension` returns `Ok( None )` (not a JS exception) for an
  // unsupported extension; a single `.expect()` only covers the outer `Result`.
  // Pitfall: `Result< Option< T >, JsValue >` has two independent failure layers --
  // unwrapping only the outer one silently passes through the inner `None`.
  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to query EXT_color_buffer_float extension" )
  .expect( "EXT_color_buffer_float extension is not supported" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" )
  .expect( "Failed to query EXT_shader_image_load_store extension" )
  .expect( "EXT_shader_image_load_store extension is not supported" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf_path = "static/gltf/multi_animation.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;
  scenes[ 0 ].borrow_mut().world_matrix_update();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  let camera = camera_setup( &canvas, &scene_bounding_box, width, height );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.ibl_set( renderer::webgl::loaders::ibl::load( &gl, "static/envMap", None ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  camera.controls_get().borrow_mut().up = F32x3::from_array( [ 0.0, -1.0, 0.0 ] );
  camera.controls_get().borrow_mut().eye = F32x3::from_array( [-5.341_171e-6, -0.015_823_878, 0.007_656_166] );

  let last_time = Rc::new( RefCell::new( 0.0 ) );

  let scaler = gui_setup::setup( gltf.animations.clone() );
  tree_print( &scenes[ 0 ].borrow().children[ 0 ] );
  let parts = parts_assemble( &scenes[ 0 ].borrow().children[ 0 ] );

  if let Some( scaler ) = scaler.borrow_mut().as_mut()
  {
    for ( part, nodes ) in parts
    {
      if let Some( group ) = scaler.group_get_mut( &part )
      {
        *group = nodes;
      }
    }
  }

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = t / 1000.0;

      if let Some( scaler ) = scaler.borrow_mut().as_mut()
      {
        let last_time = last_time.clone();

        let delta_time = time - *last_time.borrow();
        *last_time.borrow_mut() = time;

        if scaler.animation.is_completed()
        {
          scaler.animation.reset();
        }

        scaler.update( delta_time );
        scaler.set( &gltf.animations[ 0 ].nodes );
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
