use minwebgl::{self as gl, web_sys, JsCast};

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  let vertex_shader_src = include_str!( "../shaders/main.vert" );
  let fragment_shader_src = include_str!( "../shaders/main.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let file_name = "simon's_cat.mp4";
  let video_width = 640;
  let video_height = 480;

  let video_element = gl::file::load_media
  (
    file_name,
    | doc |
    {
      let video_element = doc.create_element( "video" )?
      .dyn_into::< web_sys::HtmlVideoElement >()?;
      video_element.set_width( video_width );
      video_element.set_height( video_height );
      video_element.set_muted( true );
      video_element.set_loop( true );
      Ok( video_element )
    }
  )
  .await
  .expect( "Failed to load video" );
  let texture = gl.create_texture().expect( "Failed to create texture" );
  gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
  gl::texture::d2::default_parameters( &gl );
  
  let update_and_draw =
  {
    move | _ |
    {
      gl.clear_color( 0.8, 0.8, 0.8, 1.0 );
      gl.clear( gl::COLOR_BUFFER_BIT );
      gl::texture::d2::update_video( &gl, &texture, &video_element );

      gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}
