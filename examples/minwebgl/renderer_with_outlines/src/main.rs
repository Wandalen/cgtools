
use std::collections::{HashMap, HashSet};

use minwebgl as gl;
use rand::Rng;
use gl::math::F32x4;
use renderer::webgl::
{
  loaders::gltf::GLTF,
  geometry::AttributeInfo, 
  Camera, 
  Renderer,
  post_processing::{
    self, add_buffer, add_attributes, outline::narrow_outline::{ NarrowOutlinePass, MAX_OBJECT_COUNT }, upload_buffer_data, GBuffer, GBufferAttachment, Pass, SwapFramebuffer
  }
};

mod camera_controls;
mod loaders;

fn random_color( rng : &mut impl rand::Rng ) -> F32x4
{
  let range = 0.2..1.0;
  F32x4::from
  (
    [ 
      rng.random_range( range.clone() ), 
      rng.random_range( range.clone() ),
      rng.random_range( range.clone() ),
      1.0
    ] 
  )
}

fn generate_object_colors( object_count : usize ) -> Vec< [ f32; 4 ] > 
{
  let mut object_colors = vec![ [ 0.0; 4 ]; MAX_OBJECT_COUNT ];
  let mut rng = rand::rng();

  let range = 0.2..1.0;
  ( 0..object_count ).for_each
  ( 
    | i |
    {
      object_colors[ i ] = 
      [ 
        rng.random_range( range.clone() ), 
        rng.random_range( range.clone() ),
        rng.random_range( range.clone() ),
        1.0
      ];
    } 
  );

  object_colors
}

fn get_attributes( gltf : &GLTF ) -> Result< HashMap< Box< str >, AttributeInfo >, gl::WebglError >
{
  for mesh in &gltf.meshes
  {
    let mesh_ref = mesh.as_ref().borrow();
    for primitive in &mesh_ref.primitives
    {
      let primitive_ref = primitive.as_ref().borrow();
      return Ok( primitive_ref.geometry.as_ref().borrow().get_attributes() );
    }
  }

  Err( gl::WebglError::MissingDataError( "Primitive".to_string() ) )
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

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 20.0, 20.0 ] );
  eye /= 500.0;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.001;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );

  camera_controls::setup_controls( &canvas, &camera.get_controls() );

  let gltf_path = "dodge-challenger/gltf";
  let mut gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;

  let object_id_data = add_attributes( 
    &gl, 
    &mut gltf, 
    HashSet::from( [ GBufferAttachment::PbrInfo ] ) 
  )?;

  let object_color_id_buffer = add_buffer( &gl, &mut gltf, data )?;

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_use_emission( true );
  renderer.set_ibl( loaders::ibl::load( &gl, "envMap" ).await );

  let attributes = get_attributes( &gltf )?;  

  let get_buffer = | name | attributes.get( name ).unwrap().buffer;

  let gl_buffers_iter = gltf.gl_buffers.iter().rev();

  let ( material_id_buffer, object_id_buffer ) = ( gl_buffers_iter.next().unwrap(), gl_buffers_iter.next().unwrap() );

  let attachments = HashMap::from(
    [
      ( GBufferAttachment::Position, vec![ get_buffer( "positions" ) ] ),
      ( GBufferAttachment::Albedo, vec![ get_buffer( "colors_7" ) ] ),
      ( GBufferAttachment::Normal, vec![ get_buffer( "normals" ) ] ),
      ( 
        GBufferAttachment::PbrInfo, 
        vec![ 
          object_id_buffer.clone(),
          material_id_buffer.clone(),
          get_buffer( "texture_coordinates_2" )
        ] 
      ),
      ( GBufferAttachment::ObjectColorId, vec![ object_color_id_buffer ] )
    ]
  );

  let mut gbuffer = GBuffer::new( &gl, canvas.width(), canvas.height(), attachments )?;

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;
  let mut outline = NarrowOutlinePass::new
  ( 
    &gl, 
    gbuffer.get_texture( GBufferAttachment::Normal ), 
    gbuffer.get_texture( GBufferAttachment::PbrInfo ), 
    2.0, 
    canvas.width(), 
    canvas.height() 
  )?;

  let object_count = scenes[ 0 ].as_ref().borrow().children.len();
  let object_colors = generate_object_colors( object_count );
  outline.set_object_colors( &gl, object_colors );

  scenes[ 0 ].borrow_mut().update_world_matrix();
  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let time = t as f32 / 1000.0;

      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      gbuffer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render gbuffer" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.get_main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.bind( &gl );
      swap_buffer.set_output( t );
      swap_buffer.swap();
    
      let t = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render to srgb pass" );

      // swap_buffer.bind( &gl );
      // swap_buffer.set_output( t );
      // swap_buffer.swap();

      // let outline_thickness = ( 2.0 * ( time / 1000.0 ).sin().abs() ) as f32;
      // outline.set_outline_thickness( outline_thickness );
      // let _ = outline.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      // .expect( "Failed to render outline pass" );

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
