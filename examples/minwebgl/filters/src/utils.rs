#![ allow( clippy::expect_fun_call ) ]

use web_sys::
{
  wasm_bindgen,
  HtmlImageElement,
  HtmlCanvasElement,
  Blob,
  File,
  FileReader,
  DragEvent,
  Event,
};
use wasm_bindgen::{ prelude::*, JsCast };
use minwebgl as gl;

/// Load image from a File object
pub fn load_image_from_file( file : &File, on_load_callback : Box< dyn Fn( &HtmlImageElement ) > )
{
  use std::rc::Rc;

  let file_reader = FileReader::new().expect( "Should be able to create FileReader" );
  let fr = file_reader.clone();
  let callback_rc = Rc::new( on_load_callback );

  let onload : Closure< dyn FnMut() > = Closure::new( move ||
  {
    if let Ok( result ) = fr.result()
    {
      if let Some( data_url ) = result.as_string()
      {
        let document = web_sys::window().unwrap().document().unwrap();
        let image = document.create_element( "img" ).unwrap().dyn_into::< HtmlImageElement >().unwrap();
        let img = image.clone();
        let cb = callback_rc.clone();
        let callback : Closure< dyn Fn() > = Closure::new( move || cb( &img ) );
        image.set_onload( Some( callback.as_ref().unchecked_ref() ) );
        callback.forget();
        image.set_src( &data_url );
      }
    }
  });

  file_reader.set_onload( Some( onload.as_ref().unchecked_ref() ) );
  onload.forget();

  if let Err( e ) = file_reader.read_as_data_url( &file )
  {
    minwebgl::warn!( "Failed to read file: {:?}", e );
  }
}

/// Setup file upload button
pub fn setup_file_upload< F >( button_id : &str, input_id : &str, on_file_selected : F )
where
  F : Fn( File ) + 'static
{
  let upload_btn = get_element_by_id_unchecked::< web_sys::HtmlElement >( button_id );
  let file_input = get_element_by_id_unchecked::< web_sys::HtmlInputElement >( input_id );

  // Button click triggers file input
  let file_input_clone = file_input.clone();
  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    file_input_clone.click();
  });
  upload_btn.set_onclick( Some( onclick.as_ref().unchecked_ref() ) );
  onclick.forget();

  // Handle file selection
  let onchange : Closure< dyn Fn( Event ) > = Closure::new( move | event : Event |
  {
    let target = event.target().unwrap();
    let input = target.dyn_into::< web_sys::HtmlInputElement >().unwrap();

    if let Some( files ) = input.files()
    {
      if let Some( file ) = files.get( 0 )
      {
        on_file_selected( file );
      }
    }
  });

  file_input.set_onchange( Some( onchange.as_ref().unchecked_ref() ) );
  onchange.forget();
}

/// Setup drag and drop for images
pub fn setup_drag_and_drop< F >( on_file_dropped : F )
where
  F : Fn( File ) + 'static + Clone
{
  let document = web_sys::window().unwrap().document().unwrap();
  let body = document.body().expect( "Should have a body" );
  let overlay = get_element_by_id_unchecked::< web_sys::HtmlElement >( "drop-overlay" );

  // Prevent default drag behavior
  let prevent_default : Closure< dyn Fn( DragEvent ) > = Closure::new
  (
    | event : DragEvent |
    {
      event.prevent_default();
      event.stop_propagation();
    }
  );

  body.add_event_listener_with_callback( "dragover", prevent_default.as_ref().unchecked_ref() ).unwrap();
  body.add_event_listener_with_callback( "dragenter", prevent_default.as_ref().unchecked_ref() ).unwrap();
  prevent_default.forget();

  // Show overlay on drag enter
  let overlay_clone = overlay.clone();
  let dragenter : Closure< dyn Fn( DragEvent ) > = Closure::new
  (
    move | event : DragEvent |
    {
      event.prevent_default();
      let _ = overlay_clone.class_list().add_1( "active" );
    }
  );
  body.add_event_listener_with_callback( "dragenter", dragenter.as_ref().unchecked_ref() ).unwrap();
  dragenter.forget();

  // Hide overlay on drag leave
  let overlay_clone = overlay.clone();
  let dragleave : Closure< dyn Fn( DragEvent ) > = Closure::new
  (
    move | event : DragEvent |
    {
      event.prevent_default();
      if let Some( target ) = event.target()
      {
        if let Ok( element ) = target.dyn_into::< web_sys::HtmlElement >()
        {
          if element.tag_name() == "BODY"
          {
            let _ = overlay_clone.class_list().remove_1( "active" );
          }
        }
      }
    }
  );
  body.add_event_listener_with_callback( "dragleave", dragleave.as_ref().unchecked_ref() ).unwrap();
  dragleave.forget();

  // Handle drop
  let overlay_clone = overlay.clone();
  let drop : Closure< dyn Fn( DragEvent ) > = Closure::new( move | event : DragEvent |
  {
    event.prevent_default();
    event.stop_propagation();
    let _ = overlay_clone.class_list().remove_1( "active" );

    if let Some( data_transfer ) = event.data_transfer()
    {
      if let Some( files ) = data_transfer.files()
      {
        if let Some( file ) = files.get( 0 )
        {
          on_file_dropped( file );
        }
      }
    }
  });
  body.add_event_listener_with_callback( "drop", drop.as_ref().unchecked_ref() ).unwrap();
  drop.forget();
}

/// Save canvas as PNG image
pub fn save_canvas( canvas_id : &str, filename : &str )
{
  use web_sys::console;

  let canvas = get_element_by_id_unchecked::< HtmlCanvasElement >( canvas_id );
  let filename_owned = filename.to_string();

  gl::info!( "Saving canvas: {}x{}", canvas.width(), canvas.height() );

  let callback : Closure< dyn Fn( Option< Blob > ) > = Closure::new( move | blob_opt : Option< Blob > |
  {
    if let Some( blob ) = blob_opt
    {
      gl::info!( "Blob size: {} bytes", blob.size() as u32 );
      let url = web_sys::Url::create_object_url_with_blob( &blob ).unwrap();
      let document = web_sys::window().unwrap().document().unwrap();
      let link = document.create_element( "a" ).unwrap().dyn_into::< web_sys::HtmlAnchorElement >().unwrap();
      link.set_href( &url );
      link.set_download( &filename_owned );
      link.click();
      web_sys::Url::revoke_object_url( &url ).unwrap();
      gl::info!( "Download triggered" );
    }
    else
    {
      console::warn_1( &"Failed to create blob from canvas".into() );
    }
  });

  canvas.to_blob( callback.as_ref().unchecked_ref() ).unwrap();
  callback.forget();
}

/// Capture canvas content and create a WebGL texture from it
pub fn canvas_to_texture( canvas_id : &str, gl : &minwebgl::GL ) -> Option< web_sys::WebGlTexture >
{
  // use web_sys::console;
  use minwebgl::GL;

  let canvas = get_element_by_id_unchecked::< HtmlCanvasElement >( canvas_id );

  // Ensure WebGL has finished rendering
  gl.flush();
  gl.finish();

  gl::info!( "Capturing canvas: {}x{}", canvas.width(), canvas.height() );

  // Create a new texture
  let texture = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );

  // Set texture parameters
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );

  // Upload canvas content to texture
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 1 );
  let result = gl.tex_image_2d_with_u32_and_u32_and_html_canvas_element
  (
    GL::TEXTURE_2D,
    0,
    GL::RGBA as i32,
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    &canvas,
  );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );

  if result.is_err()
  {
    gl::warn!( "Failed to create texture from canvas" );
    return None;
  }

  // Generate mipmaps for better quality
  gl.generate_mipmap( GL::TEXTURE_2D );

  gl::info!( "Texture created from canvas successfully" );

  Some( texture )
}

/// Load image from a Blob object (for bg removal result)
pub fn load_image_from_blob( blob : &Blob, on_load_callback : Box< dyn Fn( &HtmlImageElement ) > )
{
  use std::rc::Rc;

  let url = web_sys::Url::create_object_url_with_blob( blob ).expect( "Should create object URL" );
  let document = web_sys::window().unwrap().document().unwrap();
  let image = document.create_element( "img" ).unwrap().dyn_into::< HtmlImageElement >().unwrap();
  let img = image.clone();
  let callback_rc = Rc::new( on_load_callback );
  let url_clone = url.clone();

  let callback : Closure< dyn Fn() > = Closure::new( move ||
  {
    callback_rc( &img );
    let _ = web_sys::Url::revoke_object_url( &url_clone );
  });
  image.set_onload( Some( callback.as_ref().unchecked_ref() ) );
  callback.forget();
  image.set_src( &url );
}

/// Show canvas and hide placeholder text
pub fn show_canvas()
{
  if let Some( canvas ) = web_sys::window()
    .and_then( | w | w.document() )
    .and_then( | d | d.get_element_by_id( "canvas" ) )
  {
    let _ = canvas.class_list().remove_1( "hidden" );
  }

  if let Some( placeholder ) = web_sys::window()
    .and_then( | w | w.document() )
    .and_then( | d | d.get_element_by_id( "placeholder-text" ) )
  {
    let _ = placeholder.class_list().add_1( "hidden" );
  }
}

pub fn get_element_by_id_unchecked< T : JsCast >( id : &str ) -> T
{
  let document = web_sys::window()
  .expect( "Should have a window" )
  .document()
  .expect( "Should have a document" );
  document.get_element_by_id( id )
  .expect( &format!( "No element with id '{id}'" ) )
  .dyn_into::< T >()
  .expect( &format!( "Element is not of type {}", std::any::type_name::< T >() ) )
}
