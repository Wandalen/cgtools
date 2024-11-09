use minwebgl::{self as gl, web_sys};
use gl::JsCast;

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  let vert_shader = include_str!( "../shaders/main.vert" );
  let frag_shader = include_str!( "../shaders/main.frag" );
  let program = gl::ProgramFromSources::new( vert_shader, frag_shader ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Settings for `rock.png` sprite sheets
  let path = "static/rock.png";
  let sprties_in_row = 8;
  let sprites_in_col = 8;
  let sprite_width = 128;
  let sprite_height = 128;
  let amount = 64;
  let frame_rate = 24.0;

  let img_element = gl::file::load_media
  (
    path,
    | doc |
    {
      let img_element = doc.create_element( "img" )?
      .dyn_into::< web_sys::HtmlImageElement >()?;

      Ok( img_element )
    }
  )
  .await
  .expect( "Failed to load image" );
  gl::texture::d2::upload_sprite( &gl, &img_element, sprties_in_row, sprites_in_col, sprite_width, sprite_height, amount )?;

  let update_and_draw =
  {
    let mut step = 0.0;
    let amount = amount as f32 - 1.0;

    move | _ |
    {
      let frame = ( step / amount ).floor();

      gl.vertex_attrib1f( 0, frame % amount );
      gl.draw_arrays( gl::GL::TRIANGLE_STRIP, 0, 4 );

      step += frame_rate;

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}
