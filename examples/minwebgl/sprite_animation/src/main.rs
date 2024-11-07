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

  let img_element = load_img( path ).await.expect( "Failed to load image" );
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

async fn load_img( path : &str ) -> Result< web_sys::HtmlImageElement, gl::wasm_bindgen::JsValue >
{
  let document = gl::web_sys::window().unwrap().document().unwrap();
  let img_element = document.create_element( "img" )?
  .dyn_into::< gl::web_sys::HtmlImageElement >()?;
  img_element.style().set_property( "display", "none" )?;

  // Creates a new Promise instance that will execute when the image is loaded.
  let load_promise = gl::js_sys::Promise::new
  (
    &mut | resolve, _ |
    {
      // Create a Closure that will be called when the image is loaded.
      let onload = gl::wasm_bindgen::closure::Closure::once_into_js
      (
        move ||
        {
          // When the onload event is triggered, call the resolve function to complete the Promise.
          resolve.call0( &gl::JsValue::NULL ).unwrap();
        }
      );

      // The onload callback calls resolve to complete the Promise.
      img_element.set_onload( Some( onload.as_ref().unchecked_ref() ) );
    }
  );

  img_element.set_src( path );

  // Waits for the Promise created earlier to complete when the image is loaded.
  gl::JsFuture::from( load_promise ).await?;

  Ok( img_element )
}
