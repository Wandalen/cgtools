//! Renders GLTF files using postprocess effects.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/skeletal_animation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders skeleton animation from GLTF files" ) ]

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

use std::collections::{ HashMap, HashSet };
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
  Renderer,
  Node,
  animation::AnimatableComposition
};

mod lil_gui;
mod gui_setup;

fn write_tree( node : Rc< RefCell< Node > >, depth : usize, output : &mut String )
{
  let name = node
  .borrow()
  .get_name()
  .unwrap_or( "<none>".into() );

  let indent = "-".repeat( depth );
  output.push_str( &format!("{}{}\n", indent, name ) );

  for child in node.borrow().get_children()
  {
    write_tree( Rc::clone( child ), depth + 1, output );
  }
}

fn print_tree( node : Rc< RefCell< Node > > )
{
  let mut tree_str = String::new();
  write_tree( node, 1, &mut tree_str );
  gl::info!( "{}", tree_str );
}

/// Splits root sub [`Node`]s names into named subtrees
/// Not mentioned nodes from root subnodes in parts
/// argument list will be added as separated node names group
fn split_node_names_into_parts
(
  root : Rc< RefCell< Node > >,
  part_names : &[ &str ]
)
-> HashMap< Box< str >, Vec< Box< str > > >
{
  fn collect_names( node : Rc< RefCell< Node > >, out : &mut Vec< Box< str > > )
  {
    let Some( name ) = node.borrow().get_name()
    else
    {
      return;
    };

    out.push( name );
    for child in node.borrow().get_children()
    {
      collect_names( Rc::clone( &child ), out );
    }
  }

  let part_names = HashSet::< Box< str > >::from_iter
  (
    part_names.iter().map( | n | (*n).into() )
  );
  let mut not_mentioned = HashSet::new();

  let mut parts = HashMap::new();

  let _ = root.borrow()
  .traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let Some( name ) = node.borrow().get_name()
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
      let Some( name ) = node.borrow().get_name()
      else
      {
        return Ok( () );
      };

      let mut part = vec![];
      if part_names.contains( &name )
      {
        collect_names( node, &mut part );
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
    root.borrow().get_name().unwrap_or( "<none>".into() ),
    not_mentioned.into_iter().collect::< Vec< _ > >()
  );

  parts
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf_path = "gltf/multi_animation.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  gl::info!( "Scene boudnig box: {:?}", scene_bounding_box );
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag();
  let exponent =
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };
  gl::info!( "Exponent: {:?}", exponent );

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 ) * 10.0;
  let far = near * 100.0f32.powi( exponent.abs() ) / 100.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.bind_controls( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "envMap", None ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  camera.get_controls().borrow_mut().up = F32x3::from_array( [ 0.0, -1.0, 0.0 ] );
  camera.get_controls().borrow_mut().eye = F32x3::from_array( [-5.341171e-6, -0.015823878, 0.007656166] );

  let last_time = Rc::new( RefCell::new( 0.0 ) );

  let scaler = gui_setup::setup( gltf.animations.clone() );
  print_tree( scenes[ 0 ].borrow().children[ 0 ].clone() );
  let parts = vec!
  [
    "mixamorig:Neck",
    "mixamorig:RightShoulder",
    "mixamorig:LeftShoulder",
    "mixamorig:RightUpLeg",
    "mixamorig:LeftUpLeg"
  ];

  let mut parts = split_node_names_into_parts
  (
    scenes[ 0 ].borrow().children[ 0 ].clone(),
    &parts
  );

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

  gl::info!( "{:#?}", parts );

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
