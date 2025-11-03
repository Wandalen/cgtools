mod private
{
  use minwebgl as gl;
  use crate::webgl::Texture;
  use std:: { cell::RefCell, rc::Rc };
  use rustc_hash::FxHashMap;

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

  pub trait Material : std::any::Any
  {
    /// Returns the unique identifier of the material.
    fn get_id( &self ) -> uuid::Uuid;

    /// Returns the vertex shader of the material
    fn get_vertex_shader( &self ) -> String;

    /// Return the fragment shader of the material
    fn get_fragment_shader( &self ) -> String;

    /// Generates `#define` directives to be inserted into the fragment shader based on the material's properties.
    fn get_defines( &self ) -> String
    {
      String::new()
    }

    /// Configures the position of the uniform texture samplers in the shader program.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `locations`: A hash map of uniform locations.
    /// * `ibl_base_location`: The starting texture unit index for Image-Based Lighting (IBL) textures.
    fn configure
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >,
      ibl_base_location : u32,
    );

    /// Uploads the material properties to the GPU as uniforms.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `locations`: A hash map of uniform locations in the shader program.
    fn upload
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    ) -> Result< (), gl::WebglError >;

    /// Uploads the texture data of all used textures to the GPU.
    fn upload_textures( &self, gl : &gl::WebGl2RenderingContext );

    /// Binds all used textures to their respective texture units.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    fn bind( &self, gl : &gl::WebGl2RenderingContext );

    /// Dyn safe clone method
    fn dyn_clone( &self ) -> Box< dyn Material >;

    fn get_alpha_mode( &self ) -> AlphaMode
    {
      AlphaMode::Opaque
    }
  }

}


crate::mod_interface!
{
  layer pbr;

  orphan use
  {
    AlphaMode,
    TextureInfo,
    Material
  };
}