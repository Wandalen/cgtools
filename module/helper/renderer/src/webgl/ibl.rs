mod private
{
  use minwebgl as gl;

  // Struct that holds three precomputed textures to use in IBL calculations
  // Accroding to:
  // https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
  // https://learnopengl.com/PBR/IBL/Diffuse-irradiance
  #[ derive( Default ) ]
  pub struct IBL
  {
    pub diffuse_texture : Option< gl::web_sys::WebGlTexture >, 
    pub specular_1_texture : Option< gl::web_sys::WebGlTexture >, 
    pub specular_2_texture : Option< gl::web_sys::WebGlTexture >, 
  }

  impl IBL 
  {
    pub fn new() -> Self
    {
      Self::default()
    }
    
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext, base_active_texture : u32 )
    {
      gl.active_texture( gl::TEXTURE0 + base_active_texture );
      gl.bind_texture( gl::TEXTURE_CUBE_MAP, self.diffuse_texture.as_ref() );

      gl.active_texture( gl::TEXTURE0 + base_active_texture + 1 );
      gl.bind_texture( gl::TEXTURE_CUBE_MAP, self.specular_1_texture.as_ref() );

      gl.active_texture( gl::TEXTURE0 + base_active_texture + 2 );
      gl.bind_texture( gl::TEXTURE_2D, self.specular_2_texture.as_ref() );
    }    
  }
}

crate::mod_interface!
{
  orphan use
  {
    IBL
  };
}