use minwebgl::{ self as gl };

pub struct Sampler
{
  magnification : Option< u32 >,
  minification : Option< u32 >,
  wrapping_s : u32,
  wrapping_t : u32
}

impl Sampler 
{
  pub fn new( s : &gltf::texture::Sampler ) -> Self
  {
    let magnification = s.mag_filter().map( | m | m.as_gl_enum() );
    let minification = s.min_filter().map( | m | m.as_gl_enum() );
    let wrapping_s = s.wrap_s().as_gl_enum();
    let wrapping_t = s.wrap_t().as_gl_enum();

    Self
    {
      magnification,
      minification,
      wrapping_s,
      wrapping_t
    }
  }

  pub fn apply( &self, gl : &gl::WebGl2RenderingContext )
  {
    if let Some( mag ) = self.magnification
    {
      gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag as i32 );
    }

    if let Some( min ) = self.minification
    {
      gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min as i32 );
    }

    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, self.wrapping_s as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, self.wrapping_t as i32 );
  }
}