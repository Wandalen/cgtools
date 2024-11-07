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

// Update the video texture for each frame
pub fn update_video( gl : &GL, texture : &web_sys::WebGlTexture, video_element : &web_sys::HtmlVideoElement )
{
  gl.bind_texture( GL::TEXTURE_2D, Some( texture ) );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_video_element
  (
    GL::TEXTURE_2D,
    0,
    GL::RGBA as i32,
    video_element.width() as i32,
    video_element.height() as i32,
    0,
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    &video_element
  ).expect( "Failed to upload data to texture" );
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