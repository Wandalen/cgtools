use crate::*;

type GL = web_sys::WebGl2RenderingContext;

/// Creates a 2D texture with parameters.
pub fn setup( gl : &GL ) -> Option< web_sys::WebGlTexture >
{
  let texture = gl.create_texture();

  if texture.is_none() { return None; }

  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_R, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );

  texture
}

// Update the texture for each frame
pub fn update( gl : &GL, texture : &web_sys::WebGlTexture, video_element : &web_sys::HtmlVideoElement )
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
