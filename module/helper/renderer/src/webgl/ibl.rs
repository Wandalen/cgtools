mod private
{
  use minwebgl as gl;

  /// Holds three precomputed textures used for Image-Based Lighting (IBL) calculations.
  ///
  /// According to:
  /// - https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
  /// - https://learnopengl.com/PBR/IBL/Diffuse-irradiance
  #[ derive( Default, Clone ) ]
  pub struct IBL
  {
    /// The diffuse irradiance cubemap texture.
    pub diffuse_texture : Option< gl::web_sys::WebGlTexture >,
    /// The prefiltered specular environment map (cubemap) texture.
    pub specular_1_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D lookup texture containing the BRDF (Bidirectional Reflectance Distribution Function) integration result.
    pub specular_2_texture : Option< gl::web_sys::WebGlTexture >,
  }

  impl IBL
  {
    /// Creates a new `IBL` instance with default (empty) texture options.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Binds the IBL textures to specific texture units.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `base_active_texture`: The starting texture unit index to which the diffuse texture will be bound.
    ///                          Subsequent specular textures will be bound to the following units.
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
