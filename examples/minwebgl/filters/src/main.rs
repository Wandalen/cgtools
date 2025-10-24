//! Filtering example using WebGL
//! This example demonstrates how to apply various filters to an image using WebGL.

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::assign_op_pattern ) ]
#![ allow( clippy::semicolon_if_nothing_returned ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::redundant_field_names ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::let_unit_value ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::needless_continue ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::collapsible_if ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::needless_borrows_for_generic_args ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::unnecessary_wraps ) ]

mod ui_setup;
mod utils;
mod filters;
mod framebuffer;
mod renderer;
mod lil_gui;
mod zoom_pan;
mod sidebar_toggle;

use ui_setup::*;
use renderer::Renderer;
use minwebgl as gl;
use gl::GL;
use web_sys::
{
  wasm_bindgen,
  HtmlCanvasElement,
  HtmlImageElement,
  File,
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
  // Create GL context with preserveDrawingBuffer enabled for saving
  let context_options = gl::context::ContexOptions
  {
    preserve_drawing_buffer : true,
    ..Default::default()
  };
  let gl = gl::context::retrieve_or_make_with( context_options ).expect( "Can't retrieve GL context" );

  let filter_renderer = Renderer::new( &gl, None );
  let filter_renderer = Rc::new( RefCell::new( filter_renderer ) );

  let _current_filter = setup_ui( &filter_renderer );

  // Setup zoom and pan controls
  zoom_pan::setup_zoom_pan();

  // Setup sidebar toggle
  sidebar_toggle::setup_sidebar_toggle();

  // Create image loading handler that can be reused for upload and drag-drop
  let create_image_handler = | renderer : Rc< RefCell< Renderer > >, gl : GL | -> Box< dyn Fn( &HtmlImageElement ) >
  {
    Box::new
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

        // Show canvas and hide placeholder
        utils::show_canvas();

        renderer.borrow_mut().update_framebuffer_size( img.width() as i32, img.height() as i32 );
        renderer.borrow_mut().set_original_texture( texture.clone() );
        renderer.borrow_mut().set_image_texture( texture );
        renderer.borrow_mut().apply_filter( &filters::original::Original );
      }
    )
  };

  // Setup file upload button
  let filter_renderer_upload = filter_renderer.clone();
  let gl_upload = gl.clone();
  utils::setup_file_upload( "upload-btn", "file-input", move | file : File |
  {
    let onload = create_image_handler( filter_renderer_upload.clone(), gl_upload.clone() );
    utils::load_image_from_file( file, onload );
  });

  // Setup drag and drop
  let filter_renderer_drop = filter_renderer.clone();
  let gl_drop = gl.clone();
  utils::setup_drag_and_drop( move | file : File |
  {
    let onload = create_image_handler( filter_renderer_drop.clone(), gl_drop.clone() );
    utils::load_image_from_file( file, onload );
  });

  // Setup save button
  let save_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "save-btn" );
  let gl_save = gl.clone();
  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    // Ensure WebGL has finished rendering before saving
    gl_save.flush();
    gl_save.finish();

    utils::save_canvas( "canvas", "filtered-image.png" );
  });
  save_btn.set_onclick( Some( onclick.as_ref().unchecked_ref() ) );
  onclick.forget();

  // Setup apply button - capture current canvas state and make it the new source texture
  let apply_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "apply-btn" );
  let filter_renderer_apply = filter_renderer.clone();
  let gl_apply = gl.clone();
  let onclick_apply : Closure< dyn Fn() > = Closure::new( move ||
  {
    // Capture current canvas content and create a texture from it
    if let Some( new_texture ) = utils::canvas_to_texture( "canvas", &gl_apply )
    {
      // Get canvas dimensions to update framebuffer
      if let Some( canvas ) = gl_apply.canvas()
      {
        if let Ok( canvas_element ) = canvas.dyn_into::< HtmlCanvasElement >()
        {
          let width = canvas_element.width() as i32;
          let height = canvas_element.height() as i32;

          // Update renderer with new texture and framebuffer size
          filter_renderer_apply.borrow_mut().update_framebuffer_size( width, height );
          filter_renderer_apply.borrow_mut().set_image_texture( Some( new_texture ) );

          // Re-render with original filter to show the applied result
          filter_renderer_apply.borrow_mut().apply_filter( &filters::original::Original );

          // Hide apply and cancel buttons
          ui_setup::hide_apply_cancel_buttons();

          web_sys::console::log_1( &"✅ Filter applied! Ready for next filter.".into() );
        }
      }
    }
    else
    {
      web_sys::console::warn_1( &"Failed to capture canvas for applying filter".into() );
    }
  });
  apply_btn.set_onclick( Some( onclick_apply.as_ref().unchecked_ref() ) );
  onclick_apply.forget();

  // Setup cancel button - restore previous texture and hide buttons
  let cancel_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "cancel-btn" );
  let filter_renderer_cancel = filter_renderer.clone();
  let onclick_cancel : Closure< dyn Fn() > = Closure::new( move ||
  {
    // Restore previous texture
    filter_renderer_cancel.borrow_mut().restore_previous_texture();
    filter_renderer_cancel.borrow_mut().apply_filter( &filters::original::Original );

    // Hide apply and cancel buttons
    ui_setup::hide_apply_cancel_buttons();

    web_sys::console::log_1( &"❌ Filter cancelled.".into() );
  });
  cancel_btn.set_onclick( Some( onclick_cancel.as_ref().unchecked_ref() ) );
  onclick_cancel.forget();

  // Setup revert button - restore original texture immediately
  let revert_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "revert-btn" );
  let filter_renderer_revert = filter_renderer.clone();
  let onclick_revert : Closure< dyn Fn() > = Closure::new( move ||
  {
    // Restore original texture
    filter_renderer_revert.borrow_mut().restore_original_texture();
    filter_renderer_revert.borrow_mut().apply_filter( &filters::original::Original );

    // Hide apply and cancel buttons if they're visible
    ui_setup::hide_apply_cancel_buttons();

    web_sys::console::log_1( &"⏮️ Reverted to original image!".into() );
  });
  revert_btn.set_onclick( Some( onclick_revert.as_ref().unchecked_ref() ) );
  onclick_revert.forget();

  Ok( () )
}
