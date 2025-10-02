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
  Node
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

  let gltf_path = "gltf/zophrac.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;

  let need_rescale = [ "Head_Mesh", "Object_7", "Object_6" ];
  let _ = scenes[ 0 ].borrow()
  .traverse
  (
    &mut | node |
    {
      let name = node.borrow().get_name().unwrap_or( "<none>".into() );

      if need_rescale.contains( &name.to_string().as_str() )
      {
        node.borrow_mut().set_scale( F32x3::splat( 68.0 ) );
      }

      Ok( () )
    }
  );

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
  let mut eye = gl::math::F32x3::from( [ 0.0, 0.1, 1.0 ] );
  eye *= dist / 50.0;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 );
  let far = near * 100.0f32.powi( exponent.abs() );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.bind_controls( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "envMap" ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  for node in &scenes[ 0 ].borrow().children
  {
    let mut scale = node.borrow().get_scale();
    scale.0[ 0 ] *= -1.0;
    node.borrow_mut().set_scale( scale );
  }

  camera.get_controls().borrow_mut().center.0[ 1 ] += -5.5;
  camera.get_controls().borrow_mut().center.0[ 2 ] += -1.0;

  let weights = gltf.meshes.iter()
  .filter_map
  (
    | m |
    {
      let Some( ref s ) = m.borrow().skeleton
      else
      {
        return None;
      };
      let s_ref = s.borrow();
      let Some( d ) = s_ref.displacements_as_ref()
      else
      {
        return None;
      };
      let weights = d.get_morph_weights();
      *weights.borrow_mut() = d.default_weights.clone();
      Some( weights )
    }
  )
  .next()
  .unwrap();

  gui_setup::setup( weights.clone() );

  print_tree( scenes[ 0 ].borrow().children[ 0 ].clone() );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t / 1000.0;

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
