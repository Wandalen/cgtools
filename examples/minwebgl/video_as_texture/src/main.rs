use minwebgl::{self as gl, JsCast};

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

  let path = "static/simon's_cat.mp4";
  let video_width = 640;
  let video_height = 480;

  let video_element = load_video( path, video_width, video_height )
  .await
  .expect( "Failed to load video" );
  let texture = gl::texture::video::setup( &gl ).expect( "Failed to setup texture" );

  let data : [ f32; 16 ] =
  [//x     y     t_x   t_y
    -0.3,  0.5,  0.0,  0.0,
     0.3,  0.5,  1.0,  0.0,
    -0.3, -0.5,  0.0,  1.0,
     0.3, -0.5,  1.0,  1.0,
  ];

  let position_attribute_location = 0;
  let tex_coord_attribute_location = 1;

  let data_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &data_buffer, &data, gl::GL::STATIC_DRAW );
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .stride( 4 )
  .offset( 0 )
  .attribute_pointer( &gl, position_attribute_location, &data_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .stride( 4 )
  .offset( 2 )
  .attribute_pointer( &gl, tex_coord_attribute_location, &data_buffer )?;

  let update_and_draw =
  {
    move | _ |
    {
      gl.clear_color( 0.8, 0.8, 0.8, 1.0 );
      gl.clear( gl::COLOR_BUFFER_BIT );
      gl::texture::video::update( &gl, &texture, &video_element );

      gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

async fn load_video( path : &str, video_width : u32, video_height : u32 ) -> Result< gl::web_sys::HtmlVideoElement, gl::wasm_bindgen::JsValue >
{
  let document = gl::web_sys::window().unwrap().document().unwrap();
  let video_element = document
  .create_element( "video" )?
  .dyn_into::< gl::web_sys::HtmlVideoElement >()?;

  let load_promise = gl::js_sys::Promise::new
  (
    &mut | resolve, _ |
    {
      let oncanplay = gl::wasm_bindgen::prelude::Closure::once_into_js
      (
        move ||
        {
          resolve.call0( &gl::wasm_bindgen::JsValue::NULL ).unwrap();
        }
      );

      video_element.set_oncanplay( Some( oncanplay.as_ref().unchecked_ref() ) );
    }
  );

  video_element.set_width( video_width );
  video_element.set_height( video_height );
  video_element.set_loop( true );
  video_element.set_muted( true );
  video_element.set_src( path );
  let _ = video_element.play()?;

  wasm_bindgen_futures::JsFuture::from( load_promise ).await?;

  Ok( video_element )
}
