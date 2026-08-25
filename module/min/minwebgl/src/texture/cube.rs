use crate::web_sys;

/// A type alias for the WebGL2 rendering context.
type GL = web_sys::WebGl2RenderingContext;

// WebGL2 filter/wrap parameter constants ( `LINEAR`, `NEAREST`, `REPEAT`, `CLAMP_TO_EDGE`, etc. )
// are small enum values fixed by the WebGL2 spec, far below `i32::MAX` -- `tex_parameteri`
// requires `i32` per its WebIDL `GLint` signature, so this narrow, single-purpose conversion
// point is safe by construction for every constant it is called with in this file.
fn param_as_i32( value : u32 ) -> i32
{
  value as i32
}

/// Set default parameters for cube texture (linear filtering, repeat wrapping).
#[ inline ]
pub fn default_parameters( gl : &GL )
{
  filter_linear( gl );
  wrap_repeat( gl );
}

/// Sets the magnification and minification filters to LINEAR for a cube map.
#[ inline ]
pub fn filter_linear( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MIN_FILTER, param_as_i32( GL::LINEAR ) );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MAG_FILTER, param_as_i32( GL::LINEAR ) );
}

/// Sets the magnification and minification filters to NEAREST for a cube map.
#[ inline ]
pub fn filter_nearest( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MIN_FILTER, param_as_i32( GL::NEAREST ) );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MAG_FILTER, param_as_i32( GL::NEAREST ) );
}

/// Sets the wrap mode for S, T, and R dimensions to REPEAT for a cube map.
#[ inline ]
pub fn wrap_repeat( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_S, param_as_i32( GL::REPEAT ) );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_T, param_as_i32( GL::REPEAT ) );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_R, param_as_i32( GL::REPEAT ) );
}

/// Sets the wrap mode for S, T, and R dimensions to CLAMP_TO_EDGE for a cube map.
#[ inline ]
pub fn wrap_clamp( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_S, param_as_i32( GL::CLAMP_TO_EDGE ) );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_T, param_as_i32( GL::CLAMP_TO_EDGE ) );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_R, param_as_i32( GL::CLAMP_TO_EDGE ) );
}
