use crate::*;

type GL = web_sys::WebGl2RenderingContext;

/// Creates a 2D texture from HtmlImageElement.
/// Get pixel data from the HtmlImageElement using the 2d context of temporary canvas and load it into the texture array element by element.
pub fn upload( gl : &GL, img : &web_sys::HtmlImageElement, sprites_in_row : u32, sprites_in_col : u32, sprite_width : u32, sprite_height : u32, amount : u32 ) -> Result< web_sys::WebGlTexture, WebglError >
{
  let texture = gl.create_texture().ok_or( WebglError::FailedToAllocateResource( "Sprite texture" ) )?;
  gl.bind_texture( GL::TEXTURE_2D_ARRAY, Some( &texture ) );

  let ( img_width, img_height ) = ( img.width(), img.height() );

  let image_data =
  {
    let tmp_canvas = canvas::make()?;
    tmp_canvas.style().remove_property( "width" ).unwrap();
    tmp_canvas.style().remove_property( "height" ).unwrap();
    tmp_canvas.set_width( img_width );
    tmp_canvas.set_height( img_height );

    let ctx = context::from_canvas_2d( &tmp_canvas )?;

    ctx.draw_image_with_html_image_element( img, 0.0, 0.0 ).unwrap();

    let data = ctx.get_image_data( 0.0, 0.0, img_width as f64, img_height as f64 ).unwrap().data().to_vec();

    tmp_canvas.remove();

    data
  };

  gl.tex_storage_3d
  (
    GL::TEXTURE_2D_ARRAY,
    8,
    GL::RGBA8,
    sprite_width as i32,
    sprite_height as i32,
    amount as i32
  );

  let pbo = buffer::create( &gl )?;
  gl.bind_buffer( GL::PIXEL_UNPACK_BUFFER, Some( &pbo ) );
  gl.buffer_data_with_js_u8_array
  (
    GL::PIXEL_UNPACK_BUFFER,
    &js_sys::Uint8Array::from( image_data.as_bytes() ),
    GL::STATIC_DRAW
  );

  gl.pixel_storei( GL::UNPACK_ROW_LENGTH, img_width as i32 );
  gl.pixel_storei( GL::UNPACK_IMAGE_HEIGHT, img_height as i32 );

  let sprites_in_row = sprites_in_row as f64;
  let sprites_in_col = sprites_in_col as f64;
  let sprite_width_f64 = sprite_width as f64;
  let sprite_height_f64 = sprite_height as f64;
  for i in 0..amount
  {
    let row = ( i as f64 / sprites_in_row ).floor() * sprite_width_f64;
    let col =
    {
      match ( sprites_in_row, sprites_in_col )
      {
        ( 1.0, _ ) | ( _, 1.0 ) => ( i as f64 / sprites_in_col ).floor(),
        _ => i as f64 % sprites_in_col,
      }
    } * sprite_height_f64;

    gl.pixel_storei( GL::UNPACK_SKIP_PIXELS, col as i32 );
    gl.pixel_storei( GL::UNPACK_SKIP_ROWS, row as i32 );

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
