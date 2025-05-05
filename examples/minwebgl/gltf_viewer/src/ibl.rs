use minwebgl as gl;

// Struct that holds three precomputed textures to use in IBL calculations
// Accroding to:
// https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
// https://learnopengl.com/PBR/IBL/Diffuse-irradiance
#[ derive( Default ) ]
pub struct IBL
{
  diffuse_texture : Option< gl::web_sys::WebGlTexture >, 
  specular_1_texture : Option< gl::web_sys::WebGlTexture >, 
  specular_2_texture : Option< gl::web_sys::WebGlTexture >, 
}

impl IBL 
{
  pub fn new
  ( 
    diffuse_texture : Option< gl::web_sys::WebGlTexture >, 
    specular_1_texture : Option< gl::web_sys::WebGlTexture >, 
    specular_2_texture : Option< gl::web_sys::WebGlTexture >,
  ) -> Self
  {

    Self
    {
      diffuse_texture,
      specular_1_texture,
      specular_2_texture
    }
  }
  pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
  {
    gl.active_texture( gl::TEXTURE10 + 0 );
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, self.diffuse_texture.as_ref() );

    gl.active_texture( gl::TEXTURE10 + 1 );
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, self.specular_1_texture.as_ref() );

    gl.active_texture( gl::TEXTURE10 + 2 );
    gl.bind_texture( gl::TEXTURE_2D, self.specular_2_texture.as_ref() );
  }    
}