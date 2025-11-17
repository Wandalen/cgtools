mod private
{
  use minwebgl as gl;
  use gl::GL;
  use crate::webgl::{ ProgramInfo, Texture, Node };
  use std:: { cell::RefCell, fmt::Debug, rc::Rc };
  use rustc_hash::{ FxHashMap, FxHasher };

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

  /// Represents the cull mode for the material
  #[ derive( Default, Clone, Copy, PartialEq, Eq, Debug ) ]
  pub enum CullMode
  {
    /// Cull front face
    Front,
    /// Cull back face
    #[ default ]
    Back,
    /// Cull back and front face
    FrontAndBack
  }

  /// Represents the depth function
  #[ derive( Default, Clone, Copy, PartialEq, Eq, Debug ) ]
  pub enum DepthFunc
  {
    /// Never pass
    Never,
    /// Pass if the incoming value is less than the depth buffer value
    #[ default ]
    Less,
    /// Pass if the incoming value equals the depth buffer value
    Equal,
    /// Pass if the incoming value is less than or equal to the depth buffer value
    LEqual,
    /// Pass if the incoming value is greater than the depth buffer value
    Greater,
    /// Pass if the incoming value is not equal to the depth buffer value
    NotEqual,
    /// Pass if the incoming value is greater than or equal to the depth buffer value
    GEqual,
    /// Always pass
    Always
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

  /// A trait representin a generic material
  pub trait Material : std::any::Any + Debug
  {
    /// Returns the unique identifier of the material.
    fn get_id( &self ) -> uuid::Uuid;

    /// Returns a human-readable name for the material (for debugging/editor).
    fn get_name( &self ) -> Option< &str >
    {
      None
    }

    /// Returns answer need use IBL for current material instance or not
    fn need_use_ibl( &self ) -> bool
    {
      false
    }

    /// Can or not use this material IBL
    fn can_use_ibl( &self ) -> bool
    {
      false
    }

    /// Returns [`ProgramInfo`] with shader locations and used [`ShaderProgram`]
    fn get_program_info( &self, gl : &GL, program : &gl::WebGlProgram ) -> ProgramInfo;

    /// Returns the material type identifier (e.g., "PBR", "Unlit", "Custom").
    fn get_type_name(&self) -> &'static str;

    /// Returns the vertex shader of the material
    fn get_vertex_shader( &self ) -> String;

    /// Return the fragment shader of the material
    fn get_fragment_shader( &self ) -> String;

    /// Return a string containing combined version of the vertex and fragment defines
    fn get_defines_str( &self ) -> String
    {
      String::new()
    }

    /// Returns a string containing vertex shader related defines
    fn get_vertex_defines_str( &self ) -> String
    {
      String::new()
    }

    /// Returns a string containing fragment shader related defines
    fn get_fragment_defines_str( &self ) -> String
    {
      String::new()
    }

    /// Returns a hash representing the current shader configuration.
    /// Used for shader caching and variant management.
    fn get_shader_hash( &self ) -> u64
    {
      use std::hash::{ Hash, Hasher };

      let mut hasher = FxHasher::default();
      self.get_vertex_shader().hash( &mut hasher );
      self.get_fragment_shader().hash( &mut hasher );
      self.get_defines_str().hash( &mut hasher );
      hasher.finish()
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
      node : Rc< RefCell< Node > >,
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

    /// Returns an alpha mode for the current materials
    fn get_alpha_mode( &self ) -> AlphaMode
    {
      AlphaMode::Opaque
    }

    /// Returns the face culling mode.
    fn get_cull_mode( &self ) -> CullMode
    {
      CullMode::default()
    }

    /// Returns whether depth testing is enabled.
    fn is_depth_test_enabled(&self) -> bool {
      true
    }

    /// Returns whether depth writing is enabled.
    fn is_depth_write_enabled(&self) -> bool {
      true
    }

    /// Returns the depth comparison function.
    fn get_depth_func( &self ) -> DepthFunc
    {
      DepthFunc::default()
    }

    /// Returns the color write mask (R, G, B, A).
    fn get_color_write_mask(&self) -> ( bool, bool, bool, bool )
    {
      ( true, true, true, true )
    }

    /// Returns whether this material is transparent and should be rendered
    /// in the transparency pass.
    fn is_transparent( &self ) -> bool
    {
      matches!( self.get_alpha_mode(), AlphaMode::Blend )
    }
  }

}


crate::mod_interface!
{
  /// PBR Material
  layer pbr;

  orphan use
  {
    AlphaMode,
    TextureInfo,
    Material
  };
}
