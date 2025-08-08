//! Filtering example using WebGL
//! This example demonstrates how to apply various filters to an image using WebGL.

mod ui_setup;
mod utils;
mod filters;
mod framebuffer;
mod renderer;
mod lil_gui;

use ui_setup::*;
use renderer::Renderer;
use minwebgl as gl;
use gl::GL;
use web_sys::
{
  wasm_bindgen,
  HtmlCanvasElement,
  HtmlImageElement,
};
use wasm_bindgen::prelude::*;
use std::{ rc::Rc, cell::RefCell };

fn main()
{
  gl::browser::setup( Default::default() );
  if let Err( e ) = run()
  {
    gl::warn!( "{e:?}" );
  }
}

fn run() -> Result< (), gl::WebglError >
{
  let image_path = "static/unnamed.png";
  let gl = gl::context::retrieve_or_make().expect( "Can't retrieve GL context" );
  let filter_renderer = Renderer::new( &gl, None );
  let filter_renderer = Rc::new( RefCell::new( filter_renderer ) );

  setup_ui( &filter_renderer );

  let onload : Box< dyn Fn( &HtmlImageElement )> = Box::new
  (
    move | img |
    {
      let texture = gl.create_texture();
      gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

      gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 1 );
      let res = gl.tex_image_2d_with_u32_and_u32_and_html_image_element
      (
        GL::TEXTURE_2D,
        0,
        GL::RGBA as i32,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        &img,
      );
      gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );

      if res.is_err()
      {
        gl::warn!( "{res:?}" );
        return;
      }

      gl.generate_mipmap( GL::TEXTURE_2D );

      let canvas = gl.canvas().expect( "Canvas should exist" ).dyn_into::< HtmlCanvasElement >().unwrap();
      canvas.set_width( img.width() );
      canvas.set_height( img.height() );

      filter_renderer.borrow_mut().update_framebuffer_size( img.width() as i32, img.height() as i32 );
      filter_renderer.borrow_mut().set_image_texture( texture );
      filter_renderer.borrow_mut().apply_filter( &filters::original::Original );
    }
  );
  utils::load_image( &image_path,  onload );

  Ok( () )
}
