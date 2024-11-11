use crate::*;

type GL = web_sys::WebGl2RenderingContext;

/// Creates a 2D texture from HtmlImageElement.
/// Image format and internal format are assumed to be RGBA unsigned bytes.
/// Flips the texture in Y direction.
/// Returns created texture.
/// 
/// Using HtmlImageElement is recommended, as it is the most natural 
/// and the least expensive way to parse images on the web.
pub fn upload( gl : &GL, img : &web_sys::HtmlImageElement ) -> Option< web_sys::WebGlTexture >
{
  let texture = gl.create_texture();

  if texture.is_none() { return None; }

  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 1 );
  gl.tex_image_2d_with_u32_and_u32_and_html_image_element
  (
    GL::TEXTURE_2D,
    0,
    GL::RGBA as i32,
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    &img
  ).expect( "Failed to upload data to texture" );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );

  texture
}

/// Creates a 2D texture from HtmlImageElement.
/// Image format and internal format are assumed to be RGBA unsigned bytes.
/// Does not flip the texture in Y direction.
/// Returns created texture.
pub fn upload_no_flip( gl : &GL, img : &web_sys::HtmlImageElement ) -> Option< web_sys::WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );
  gl.tex_image_2d_with_u32_and_u32_and_html_image_element
  (
    GL::TEXTURE_2D,
    0,
    GL::RGBA as i32,
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    &img
  ).expect( "Failed to upload data to texture" );

  texture
}

/// Creates a 2D texture from HtmlImageElement.
/// Get pixel data from the HtmlImageElement using the 2d context of temporary canvas and load it into the texture array element by element.
pub async fn upload_sprite( gl : &GL, img : &web_sys::HtmlImageElement, sprites_in_row : u32, sprites_in_col : u32, sprite_width : u32, sprite_height : u32, amount : u32 ) -> Result< web_sys::WebGlTexture, WebglError >
{
  let load_promise = js_sys::Promise::new
  (
    &mut | resolve, reject |
    {
      let on_load = wasm_bindgen::prelude::Closure::once_into_js
      (
        move || { resolve.call0( &JsValue::NULL ).unwrap() }
      );

      let on_error = wasm_bindgen::prelude::Closure::once_into_js
      (
        move || { reject.call1( &JsValue::NULL, &JsValue::from_str( "Failed to load image" ) ).unwrap() }
      );

      img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
      img.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );
    }
  );

  JsFuture::from( load_promise ).await.unwrap();

  let texture = gl.create_texture().ok_or( WebglError::FailedToAllocateResource( "Sprite texture" ) )?;
  gl.bind_texture( GL::TEXTURE_2D_ARRAY, Some( &texture ) );

  let ( img_width, img_height ) = ( img.width(), img.height() );

  let image_data =
  {
    let tmp_canvas = canvas::make()?;
    // Remove global canvas properties.
    tmp_canvas.style().remove_property( "width" ).unwrap();
    tmp_canvas.style().remove_property( "height" ).unwrap();
    // Set custom properties.
    tmp_canvas.set_width( img_width );
    tmp_canvas.set_height( img_height );

    // Get 2d context of the temp canvas.
    let ctx = context::from_canvas_2d( &tmp_canvas )?;

    // Draw image to temp canvas.
    ctx.draw_image_with_html_image_element( img, 0.0, 0.0 ).unwrap();

    // Get pixel array of the image.
    let data = ctx.get_image_data( 0.0, 0.0, img_width as f64, img_height as f64 ).unwrap().data().to_vec();

    tmp_canvas.remove();

    data
  };

  // Allocate memory for the 3D texture.
  gl.tex_storage_3d
  (
    GL::TEXTURE_2D_ARRAY,
    8,
    GL::RGBA8,
    sprite_width as i32,
    sprite_height as i32,
    amount as i32
  );

  // Create a Pixel Buffer Object (PBO) and copy the image data into it.
  let pbo = buffer::create( &gl )?;
  gl.bind_buffer( GL::PIXEL_UNPACK_BUFFER, Some( &pbo ) );
  gl.buffer_data_with_js_u8_array
  (
    GL::PIXEL_UNPACK_BUFFER,
    &js_sys::Uint8Array::from( image_data.as_bytes() ),
    GL::STATIC_DRAW
  );

  // Set the pixel store parameters for 3D texture uploads.
  gl.pixel_storei( GL::UNPACK_ROW_LENGTH, img_width as i32 );
  gl.pixel_storei( GL::UNPACK_IMAGE_HEIGHT, img_height as i32 );

  let sprites_in_row = sprites_in_row as f64;
  let sprites_in_col = sprites_in_col as f64;
  let sprite_width_f64 = sprite_width as f64;
  let sprite_height_f64 = sprite_height as f64;
  for i in 0..amount
  {
    // Calculate the row and column coordinates for the current sprite based on the total number of sprites and their size.
    let row = ( i as f64 / sprites_in_row ).floor() * sprite_width_f64;
    let col =
    {
      match ( sprites_in_row, sprites_in_col )
      {
        ( 1.0, _ ) | ( _, 1.0 ) => ( i as f64 / sprites_in_col ).floor(),
        _ => i as f64 % sprites_in_col,
      }
    } * sprite_height_f64;

    // Set the correct position of the sprite in the PBO.
    gl.pixel_storei( GL::UNPACK_SKIP_PIXELS, col as i32 );
    gl.pixel_storei( GL::UNPACK_SKIP_ROWS, row as i32 );

    // Copy the current sprite data from PBO to a 3D texture.
    gl.tex_sub_image_3d_with_i32(
      GL::TEXTURE_2D_ARRAY,
      0,
      0,
      0,
      i as i32,
      sprite_width as i32,
      sprite_height as i32,
      1,
      GL::RGBA,
      GL::UNSIGNED_BYTE,
      0
    ).unwrap();
  }

  gl.tex_parameteri( GL::TEXTURE_2D_ARRAY, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D_ARRAY, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

  gl.generate_mipmap( GL::TEXTURE_2D_ARRAY );
  gl.tex_parameteri( GL::TEXTURE_2D_ARRAY, GL::TEXTURE_BASE_LEVEL, 0 );

  Ok( texture )
}

/// Set the default parameters for the texture
/// Sets MAG and MIN filters to LINEAR
/// Set wrap mode for S, R, T dimensions to REPEAT
pub fn default_parameters( gl : &GL )
{
  filter_linear( gl );
  wrap_repeat( gl );
}

/// Set the magnification and minification filters to LINEAR
pub fn filter_linear( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
}

/// Set the magnification and minification filters to NEAREST
pub fn filter_nearest( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );
}

/// Set the wrap mode for S, T and R dimensions to REPEAT
pub fn wrap_repeat( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_R, GL::REPEAT as i32 );
}

/// Set the wrap mode for S, T and R dimensions to CLAMP_TO_EDGE
pub fn wrap_clamp( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_R, GL::CLAMP_TO_EDGE as i32 );
}