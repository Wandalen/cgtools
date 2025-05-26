
use minwebgl as gl;

use renderer::webgl::
{
  post_processing::{self, Pass, SwapFramebuffer}, Camera, Renderer
};

mod camera_controls;
mod loaders;
//mod post_processing;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  //let gl = gl::context::from_canvas( &canvas )?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  //let _ = gl.get_extension( "EXT_color_buffer_half_float" ).expect( "Failed to enable EXT_color_buffer_half_float extension" );

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
  let far = 100.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );

  camera_controls::setup_controls( &canvas, &camera.get_controls() );

  let gltf_path = "dodge-challenger/gltf";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 );
  renderer.set_use_emission( true );
  renderer.set_ibl( loaders::ibl::load( &gl, "envMap" ).await );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl, canvas.width(), canvas.height() )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;
  //let bloom = post_processing::UnrealBloomPass::new( &gl, width, height, format)

  scenes[ 0 ].borrow_mut().update_world_matrix();
  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

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
