use crate::*;

/// A type alias for the WebGL2 rendering context.
type GL = web_sys::WebGl2RenderingContext;

/// Sets the default texture parameters for a cube map, including linear filtering and repeat wrapping.
pub fn default_parameters( gl : &GL )
{
  filter_linear( gl );
  wrap_repeat( gl );
}

/// Sets the magnification and minification filters to LINEAR for a cube map.
pub fn filter_linear( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
}

/// Sets the magnification and minification filters to NEAREST for a cube map.
pub fn filter_nearest( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );
}

/// Sets the wrap mode for S, T, and R dimensions to REPEAT for a cube map.
pub fn wrap_repeat( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_S, GL::REPEAT as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_T, GL::REPEAT as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_R, GL::REPEAT as i32 );
}

/// Sets the wrap mode for S, T, and R dimensions to CLAMP_TO_EDGE for a cube map.
pub fn wrap_clamp( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_R, GL::CLAMP_TO_EDGE as i32 );
}
