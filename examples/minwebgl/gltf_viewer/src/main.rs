
use minwebgl as gl;

use renderer::webgl::
{
  post_processing::{self, Pass}, Camera, Renderer
};

mod camera_controls;
mod loaders;
//mod post_processing;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;
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

  let framebuffer = gl.create_framebuffer();

  gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
  gl::drawbuffers::drawbuffers( &gl, &[ 
    gl::drawbuffers::ColorAttachment::N0, 
    //gl::drawbuffers::ColorAttachment::N1 
  ] );

  let renderbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( gl::RENDERBUFFER, renderbuffer.as_ref() );
  gl.renderbuffer_storage( gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, canvas.width() as i32, canvas.height() as i32 );
  gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, renderbuffer.as_ref() );
  gl.bind_renderbuffer( gl::RENDERBUFFER, None );

  let framebuffer2 = gl.create_framebuffer();
  gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer2.as_ref() );

  gl::drawbuffers::drawbuffers( &gl, &[ 
    gl::drawbuffers::ColorAttachment::N0
  ] );

  let renderbuffer2 = gl.create_renderbuffer();
  gl.bind_renderbuffer( gl::RENDERBUFFER, renderbuffer2.as_ref() );
  gl.renderbuffer_storage( gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, canvas.width() as i32, canvas.height() as i32 );
  gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, renderbuffer2.as_ref() );
  gl.bind_renderbuffer( gl::RENDERBUFFER, None );



  let main_texture = gl.create_texture();
  let emissive_texture = gl.create_texture();

  gl.bind_texture( gl::TEXTURE_2D, main_texture.as_ref() );
  //gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, canvas.width() as i32, canvas.height()  as i32 );
  gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA32F, canvas.width() as i32, canvas.height()  as i32 );
  gl::texture::d2::filter_linear( &gl );
  gl.bind_texture( gl::TEXTURE_2D, emissive_texture.as_ref() );
  gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, canvas.width()  as i32, canvas.height()  as i32 );
  //gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA32F, canvas.width()  as i32, canvas.height()  as i32 );
  gl::texture::d2::filter_linear( &gl );

  gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
  gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, main_texture.as_ref(), 0 );
  //gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, emissive_texture.as_ref(), 0 );


  let mut renderer = Renderer::new();
  //renderer.set_render_to_screen( false );
  renderer.set_ibl( loaders::ibl::load( &gl, "envMap" ).await );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl, canvas.width(), canvas.height() )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;
  //let bloom = post_processing::UnrealBloomPass::new( &gl, width, height, format)

  // gl.enable( gl::DEPTH_TEST );
  // gl.enable( gl::BLEND );

  // gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );

  // gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  // gl.clear_depth( 1.0 );

  scenes[ 0 ].borrow_mut().update_world_matrix();
  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, main_texture.as_ref(), 0 );
      //gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, emissive_texture.as_ref(), 0 );

      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      // gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, None, 0 );
      // gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, None, 0 );

      gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer2.as_ref() );
      let t = tonemapping.render( &gl, main_texture.clone() )
      .expect( "Failed to render tonemapping pass" );
      to_srgb.render( &gl, t  )
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
