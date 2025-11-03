mod private
{
  use mingl::Former;
  use minwebgl as gl;
  use crate::webgl::Texture;
  use std:: { cell::RefCell, collections::HashMap, rc::Rc };

  /// Represents the alpha blending mode of the material.
  #[ derive( Default, Clone, Copy, PartialEq, Eq, Debug ) ]
  pub enum AlphaMode
  {
    /// The material is fully opaque.
    #[ default ]
    Opaque,
    /// The material uses a mask based on an alpha cutoff value.
    Mask,
    /// The material uses standard alpha blending.
    Blend
  }

  /// Stores information about a texture used by the material, including the texture itself and its UV coordinates.
  /// 
  /// You may have several attibutes for the UV coordinates in the shader:
  /// `
  /// layout( location = 0 ) in vec2 uv_0;
  /// layout( location = 1 ) in vec2 uv_1;
  /// `
  /// uv_position will defines which UV to use
  #[ derive( Default, Clone, Debug ) ]
  pub struct TextureInfo
  {
    /// The texture object.
    pub texture : Rc< RefCell< Texture > >,
    /// The UV coordinate set index to use for this texture.
    pub uv_position : u32
  }

  impl TextureInfo 
  {
    /// Uploads the texture data to the GPU.
    pub fn upload( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.texture.borrow().upload( gl );
    }

    /// Binds the texture to a texture unit.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.texture.borrow().bind( gl );
    }
  }

}


crate::mod_interface!
{
  layer pbr;

  orphan use
  {
    AlphaMode,
    TextureInfo
  };
}