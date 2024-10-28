use minwebgl as gl;
use gl::GL;
use web_sys::
{
  wasm_bindgen,
  HtmlCanvasElement,
  HtmlImageElement,
};
use wasm_bindgen::prelude::*;

fn main()
{
  gl::browser::setup( Default::default() );
  run();
}

fn run()
{
  let image_path = "static/download.png";
  let gl = gl::context::retrieve_or_make().expect( "Can't retrieve GL context" );

  let on_load = move | img : &HtmlImageElement |
  {
    let texture = gl.create_texture();
    gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

    gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 1 );
    gl.tex_image_2d_with_u32_and_u32_and_html_image_element
    (
      GL::TEXTURE_2D,
      0,
      GL::RGBA as i32,
      GL::RGBA,
      GL::UNSIGNED_BYTE,
      &img,
    ).expect( "Can't load an image" );
    gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );
    gl.generate_mipmap( GL::TEXTURE_2D );

    let canvas = gl.canvas().expect( "Canvas should exist" ).dyn_into::< HtmlCanvasElement >().unwrap();
    canvas.set_width( img.width() );
    canvas.set_height( img.height() );
    gl.viewport( 0, 0, img.width() as i32, img.height() as i32 );

    let vertex_source = include_str!( "shaders/main.vert" );
    let fragment_source = include_str!( "shaders/main.frag" );
    let program = gl::ProgramFromSources::new( vertex_source, fragment_source )
    .compile_and_link( &gl )
    .unwrap();

    let texel_size = [ 1.0 / img.width() as f32, 1.0 / img.height() as f32 ];
    let texel_size_location = gl.get_uniform_location( &program, "u_texel_size" );
    gl.use_program( Some( &program ) );
    gl::uniform::upload( &gl, texel_size_location, texel_size.as_slice() ).unwrap();
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  };

  load_image( &image_path, Box::new( on_load ) );
}


/// Provide full path to image like `"static/image.png"`
fn load_image( path : &str, on_load_callback : Box< dyn Fn( &HtmlImageElement ) > )
{
  let window = web_sys::window().expect( "Should have a window" );
  let document = window.document().expect( "Should have a document" );
  let image = document.create_element( "img" ).unwrap().dyn_into::< HtmlImageElement >().unwrap();
  let img = image.clone();
  let on_load_callback : Closure< dyn Fn() > = Closure::new( move || on_load_callback( &img ) );
  image.set_onload( Some( on_load_callback.as_ref().unchecked_ref() ) );
  on_load_callback.forget();
  let origin = window.location().origin().expect( "Should have an origin" );
  let url = format!( "{origin}/{path}" );
  image.set_src( &url );
}
