mod private
{
  use minwebgl as gl;
  use crate::webgl::{ Node, ShaderProgram, Texture };
  use std::{ cell::RefCell, fmt::Debug, rc::Rc };
  use rustc_hash::FxHasher;

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
    Blend,
  }

  /// Represents the cull mode for the material
  #[ derive( Default, Clone, Copy, PartialEq, Eq, Debug ) ]
  #[ repr( u32 ) ]
  pub enum CullMode
  {
    /// Cull front face
    Front = gl::FRONT,
    /// Cull back face
    #[ default ]
    Back = gl::BACK,
    /// Cull back and front face
    FrontAndBack = gl::FRONT_AND_BACK
  }

  /// Defines order in which faces will be treated as front faces
  #[ derive( Default, Clone, Copy, PartialEq, Eq, Debug ) ]
  #[ repr( u32 ) ]
  pub enum FrontFace
  {
    /// Clockwise face order
    Cw = gl::CW,
    /// Counter clockwise face order
    #[ default ]
    Ccw = gl::CCW,
  }

  /// Represents the depth function
  #[ derive( Default, Clone, Copy, PartialEq, Eq, Debug ) ]
  #[ repr( u32 ) ]
  pub enum DepthFunc
  {
    /// Never pass
    Never = gl::NEVER,
    /// Pass if the incoming value is less than the depth buffer value
    #[ default ]
    Less = gl::LESS,
    /// Pass if the incoming value equals the depth buffer value
    Equal = gl::EQUAL,
    /// Pass if the incoming value is less than or equal to the depth buffer value
    LEqual = gl::LEQUAL,
    /// Pass if the incoming value is greater than the depth buffer value
    Greater = gl::GREATER,
    /// Pass if the incoming value is not equal to the depth buffer value
    NotEqual = gl::NOTEQUAL,
    /// Pass if the incoming value is greater than or equal to the depth buffer value
    GEqual = gl::GEQUAL,
    /// Always pass
    Always = gl::ALWAYS
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

  /// Used to get additional information for material upload
  #[ derive( Debug, Clone ) ]
  pub struct MaterialUploadContext<'a>
  {
    /// current processed [`Node`]
    pub node : &'a Node,
    /// id of current processed primitive of inner mesh
    pub primitive_id : Option< usize >
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

    /// Signal for updating material uniforms
    fn needs_update( &self ) -> bool;

    /// Defines if current material instance uses IBL now
    fn needs_ibl( &self ) -> bool
    {
      false
    }

    /// Returns reference to [`ProgramInfo`] with shader locations and used [`ShaderProgram`]
    fn shader( &self ) -> &dyn ShaderProgram;

    /// Returns mutable reference to [`ProgramInfo`] with shader locations and used [`ShaderProgram`]
    fn shader_mut( &mut self ) -> &mut dyn ShaderProgram;

    /// Returns the material type identifier (e.g., "PBR", "Unlit", "Custom").
    fn type_name( &self ) -> &'static str;

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
      ibl_base_location : u32,
    );

    /// Uploads the material properties to the GPU as uniforms. Use this when material state is changed.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `locations`: A hash map of uniform locations in the shader program.
    fn upload_on_state_change
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      context : &MaterialUploadContext< '_ >
    )
    -> Result< (), gl::WebglError >;

    /// Uploads the material properties that need update every frame to the GPU.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `locations`: A hash map of uniform locations in the shader program.
    fn upload
    (
      &self,
      _gl : &gl::WebGl2RenderingContext,
      _context : &MaterialUploadContext< '_ >
    )
    -> Result< (), gl::WebglError >
    {
      Ok( () )
    }

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
    fn get_cull_mode( &self ) -> Option< CullMode >
    {
      None
    }

    /// Returns the front face order
    fn get_front_face( &self ) -> FrontFace
    {
      FrontFace::default()
    }

    /// Returns whether depth testing is enabled.
    fn is_depth_test_enabled( &self ) -> bool
    {
      true
    }

    /// Returns whether depth writing is enabled.
    fn is_depth_write_enabled( &self ) -> bool
    {
      true
    }

    /// Returns the depth comparison function.
    fn get_depth_func( &self ) -> DepthFunc
    {
      DepthFunc::default()
    }

    /// Returns the color write mask (R, G, B, A).
    fn get_color_write_mask( &self ) -> ( bool, bool, bool, bool )
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

// visibility

crate::mod_interface!
{
  /// PBR Material
  layer pbr;

  orphan use
  {
    AlphaMode,
    CullMode,
    DepthFunc,
    FrontFace,
    TextureInfo,
    MaterialUploadContext,
    Material
  };
}
