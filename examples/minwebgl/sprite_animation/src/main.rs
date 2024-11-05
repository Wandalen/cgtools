use minwebgl as gl;
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

  let path = "static/rock.png";
  let sprties_in_row = 8;
  let sprites_in_col = 8;
  let sprite_width = 128;
  let sprite_height = 128;
  let amount = 64;
  let frame_rate = 24.0;

  let document = gl::web_sys::window().unwrap().document().unwrap();
  let img_element = document.create_element( "img" )
  .unwrap()
  .dyn_into::< gl::web_sys::HtmlImageElement >()
  .unwrap();
  img_element.style().set_property( "display", "none" ).unwrap();

  let load_promise = gl::js_sys::Promise::new
  (
    &mut | resolve, _ |
    {
      let onload = gl::wasm_bindgen::closure::Closure::once_into_js
      (
        move ||
        {
          resolve.call0( &gl::JsValue::NULL ).unwrap();
        }
      );
      
      img_element.set_onload( Some( onload.as_ref().unchecked_ref() ) );
    }
  );

  img_element.set_src( path );

  wasm_bindgen_futures::JsFuture::from( load_promise ).await.unwrap();

  gl::texture::sprite::upload( &gl, &img_element, sprties_in_row, sprites_in_col, sprite_width, sprite_height, amount )?;

  let data : [ f32; 24 ] =
  [//x      y      t_x   t_y
    -0.25, -0.50,  0.0,  1.0,
     0.25,  0.50,  1.0,  0.0,
    -0.25,  0.50,  0.0,  0.0,
    -0.25, -0.50,  0.0,  1.0,
     0.25, -0.50,  1.0,  1.0,
     0.25,  0.50,  1.0,  0.0,
  ];

  let position_attribute_position = 0;
  let tex_coord_attribute_position = 1;
  let depth_attribute_position = 2;

  let data_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &data_buffer, &data, gl::GL::STATIC_DRAW );
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .stride( 4 )
  .offset( 0 )
  .attribute_pointer( &gl, position_attribute_position, &data_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .stride( 4 )
  .offset( 2 )
  .attribute_pointer( &gl, tex_coord_attribute_position, &data_buffer )?;

  let update_and_draw =
  {
    let mut step = 0.0;
    let amount = amount as f32 - 1.0;

    move | _ |
    {
      let frame = ( step / amount ).floor();

      gl.vertex_attrib1f( depth_attribute_position, frame % amount );
      gl.draw_arrays( gl::GL::TRIANGLES, 0, 6 );

      step += frame_rate;

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}
