//! Filtering example using WebGL
//! This example demonstrates how to apply various filters to an image using WebGL.

mod ui_setup;
mod utils;
mod filters;
mod framebuffer;
mod renderer;
mod controls;
mod zoom_pan;
mod sidebar_toggle;
mod bg_removal_bindgen;

use ui_setup::setup_ui;
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
  gl::browser::setup( gl::browser::Config::default() );
  run();
}

/// Creates a reusable handler that uploads an `HtmlImageElement` into a GL texture,
/// resizes the canvas to match, and applies the original filter.
fn create_image_handler( renderer : Rc< RefCell< Renderer > >, gl : GL ) -> Box< dyn Fn( &HtmlImageElement ) >
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
        img,
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
}

/// Sets up the file upload button.
fn setup_upload_button( filter_renderer : &Rc< RefCell< Renderer > >, gl : &GL )
{
  let filter_renderer_upload = filter_renderer.clone();
  let gl_upload = gl.clone();
  utils::setup_file_upload( "upload-btn", "file-input", move | file : File |
  {
    let onload = create_image_handler( filter_renderer_upload.clone(), gl_upload.clone() );
    utils::load_image_from_file( &file, onload );
  });
}

/// Sets up drag-and-drop image loading.
fn setup_drag_drop_handler( filter_renderer : &Rc< RefCell< Renderer > >, gl : &GL )
{
  let filter_renderer_drop = filter_renderer.clone();
  let gl_drop = gl.clone();
  utils::setup_drag_and_drop( move | file : File |
  {
    let onload = create_image_handler( filter_renderer_drop.clone(), gl_drop.clone() );
    utils::load_image_from_file( &file, onload );
  });
}

/// Sets up the save button.
fn setup_save_button( gl : &GL )
{
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
}

/// Sets up the apply button - captures current canvas state and makes it the new source texture.
fn setup_apply_button
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  gl : &GL,
  current_filter : &Rc< RefCell< String > >
)
{
  let apply_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "apply-btn" );
  let filter_renderer_apply = filter_renderer.clone();
  let gl_apply = gl.clone();
  let current_filter_apply = current_filter.clone();
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

          // Clear previous state so restore won't undo the apply
          filter_renderer_apply.borrow_mut().clear_previous_state();

          // Reset current filter so it can be re-applied
          *current_filter_apply.borrow_mut() = String::from( "none" );

          // Hide controls bar
          ui_setup::hide_controls_bar();

          gl::info!( "✅ Filter applied! Ready for next filter." );
        }
      }
    }
    else
    {
      gl::warn!( "Failed to capture canvas for applying filter" );
    }
  });
  apply_btn.set_onclick( Some( onclick_apply.as_ref().unchecked_ref() ) );
  onclick_apply.forget();
}

/// Sets up the cancel button - restores previous texture and hides buttons.
fn setup_cancel_button
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >
)
{
  let cancel_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "cancel-btn" );
  let filter_renderer_cancel = filter_renderer.clone();
  let current_filter_cancel = current_filter.clone();
  let onclick_cancel : Closure< dyn Fn() > = Closure::new( move ||
  {
    // Reset current filter so it can be re-applied
    *current_filter_cancel.borrow_mut() = String::from( "none" );

    // Restore previous texture and canvas size
    filter_renderer_cancel.borrow_mut().restore_previous_texture();
    filter_renderer_cancel.borrow_mut().clear_previous_state();
    filter_renderer_cancel.borrow_mut().apply_filter( &filters::original::Original );

    // Hide controls bar
    ui_setup::hide_controls_bar();

    gl::info!( "❌ Filter cancelled." );
  });
  cancel_btn.set_onclick( Some( onclick_cancel.as_ref().unchecked_ref() ) );
  onclick_cancel.forget();
}

/// Sets up the revert button - restores original texture immediately.
fn setup_revert_button( filter_renderer : &Rc< RefCell< Renderer > > )
{
  let revert_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "revert-btn" );
  let filter_renderer_revert = filter_renderer.clone();
  let onclick_revert : Closure< dyn Fn() > = Closure::new( move ||
  {
    // Restore original texture
    filter_renderer_revert.borrow_mut().restore_original_texture();
    filter_renderer_revert.borrow_mut().apply_filter( &filters::original::Original );

    // Hide controls bar if visible
    ui_setup::hide_controls_bar();

    gl::info!( "⏮️ Reverted to original image!" );
  });
  revert_btn.set_onclick( Some( onclick_revert.as_ref().unchecked_ref() ) );
  onclick_revert.forget();
}

/// Creates the closure that loads the background-removed image and updates the canvas.
fn create_bg_removal_image_handler
(
  gl : GL,
  renderer : Rc< RefCell< Renderer > >,
  is_processing : Rc< RefCell< bool > >
) -> Box< dyn Fn( &HtmlImageElement ) >
{
  Box::new( move | img |
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
      img,
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

    utils::show_canvas();

    renderer.borrow_mut().update_framebuffer_size( img.width() as i32, img.height() as i32 );
    renderer.borrow_mut().set_image_texture( texture );
    renderer.borrow_mut().apply_filter( &filters::original::Original );

    *is_processing.borrow_mut() = false;
    gl::info!( "Background removed successfully!" );
  })
}

/// Runs the background-removal pipeline: capture canvas, process, and load result.
async fn process_background_removal
(
  gl : GL,
  renderer : Rc< RefCell< Renderer > >,
  is_processing : Rc< RefCell< bool > >
)
{
  gl.flush();
  gl.finish();

  let canvas = utils::get_element_by_id_unchecked::< HtmlCanvasElement >( "canvas" );

  // Convert canvas to blob via Promise
  let promise = js_sys::Promise::new( &mut | resolve, _reject |
  {
    let cb : Closure< dyn FnMut( JsValue ) > = Closure::once( move | blob : JsValue |
    {
      let _ = resolve.call1( &JsValue::NULL, &blob );
    });
    let _ = canvas.to_blob( cb.as_ref().unchecked_ref() );
    cb.forget();
  });

  let blob_js = match wasm_bindgen_futures::JsFuture::from( promise ).await
  {
    Ok( v ) => v,
    Err( e ) =>
    {
      gl::warn!( "Failed to get canvas blob: {e:?}" );
      *is_processing.borrow_mut() = false;
      return;
    }
  };

  if blob_js.is_null() || blob_js.is_undefined()
  {
    gl::warn!( "Canvas blob is null" );
    *is_processing.borrow_mut() = false;
    return;
  }

  let blob : web_sys::Blob = blob_js.unchecked_into();
  gl::info!( "Removing background..." );

  if let Some( processed_blob ) = bg_removal_bindgen::process_image( blob ).await
  {
    // Load result as image and update canvas
    let handler = create_bg_removal_image_handler( gl.clone(), renderer.clone(), is_processing.clone() );
    utils::load_image_from_blob( &processed_blob, handler );
  }
  else
  {
    gl::warn!( "Background removal failed" );
    *is_processing.borrow_mut() = false;
  }
}

/// Sets up the remove-background button.
fn setup_bg_remove_button( filter_renderer : &Rc< RefCell< Renderer > >, gl : &GL )
{
  let bg_btn = utils::get_element_by_id_unchecked::< web_sys::HtmlElement >( "bg-remove-btn" );
  let gl_bg = gl.clone();
  let filter_renderer_bg = filter_renderer.clone();
  let is_processing = Rc::new( RefCell::new( false ) );
  let onclick_bg : Closure< dyn Fn() > = Closure::new( move ||
  {
    if *is_processing.borrow()
    {
      return;
    }
    *is_processing.borrow_mut() = true;

    let gl_inner = gl_bg.clone();
    let renderer_inner = filter_renderer_bg.clone();
    let is_processing_inner = is_processing.clone();
    wasm_bindgen_futures::spawn_local( process_background_removal( gl_inner, renderer_inner, is_processing_inner ) );
  });
  bg_btn.set_onclick( Some( onclick_bg.as_ref().unchecked_ref() ) );
  onclick_bg.forget();
}

fn run()
{
  // Create GL context with preserveDrawingBuffer enabled for saving
  // and premultiplied_alpha disabled for correct transparency handling
  let context_options = gl::context::ContextOptions
  {
    preserve_drawing_buffer : true,
    premultiplied_alpha : false,
    ..Default::default()
  };
  let gl = gl::context::retrieve_or_make_with( context_options ).expect( "Can't retrieve GL context" );

  let filter_renderer = Renderer::new( &gl, None );
  let filter_renderer = Rc::new( RefCell::new( filter_renderer ) );

  let current_filter = setup_ui( &filter_renderer );

  // Setup zoom and pan controls
  zoom_pan::setup_zoom_pan();

  // Setup sidebar toggle
  sidebar_toggle::setup_sidebar_toggle();

  setup_upload_button( &filter_renderer, &gl );
  setup_drag_drop_handler( &filter_renderer, &gl );
  setup_save_button( &gl );
  setup_apply_button( &filter_renderer, &gl, &current_filter );
  setup_cancel_button( &filter_renderer, &current_filter );
  setup_revert_button( &filter_renderer );
  setup_bg_remove_button( &filter_renderer, &gl );
}
