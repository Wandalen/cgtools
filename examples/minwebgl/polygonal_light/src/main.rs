//! Polygonal light demo

mod precompute;

use minwebgl as gl;
use gl::{ JsCast as _, GL, AsBytes as _ };
use web_sys::{ HtmlCanvasElement, WebGlTexture };

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async move { gl::info!( "{:?}", run().await ) } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let width = ( fwidth * dpr ) as i32;
  let height = ( fheight * dpr ) as i32;
  let aspect = width as f32 / height as f32;

  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  let ltc1 = load_table( &gl, precompute::LTC1 );
  let ltc2 = load_table( &gl, precompute::LTC2 );

  todo!()
}

fn load_table( gl : &GL, table : &[ f32 ] ) -> Option< WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
  (
    GL::TEXTURE_2D,
    0,
    GL::RGBA as i32,
    64,
    64,
    0,
    GL::RGBA,
    GL::FLOAT,
    Some( table.as_bytes() )
  ).expect( "Failed to load data" );
  gl::texture::d2::filter_linear( gl );
  gl::texture::d2::wrap_clamp( gl );

  texture
}
