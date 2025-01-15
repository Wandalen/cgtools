use minwebgl as gl;

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
  let image_element = gl::dom::create_image_element( path )
  .expect( "Failed to create image element" );
  let sprite_sheet = gl::texture::d2::SpriteSheet
  {
    sprites_in_row: 8,
    sprites_in_col: 8,
    sprite_width: 128,
    sprite_height: 128,
    amount: 64,
  };

  gl::texture::d2::upload_sprite( &gl, &image_element, &sprite_sheet ).await?;

  let update_and_draw =
  {
    let mut step = 0.0;
    let frame_rate = 24.0;
    let amount = sprite_sheet.amount as f32 - 1.0;

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
