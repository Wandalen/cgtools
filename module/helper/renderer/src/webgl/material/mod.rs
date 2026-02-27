mod private
{
  use minwebgl as gl;
  use crate::webgl::{ Node, ShaderProgram, Texture };
  use std::{ cell::RefCell, fmt::Debug, rc::Rc };
  use std::hash::{ Hash, Hasher };
  use rustc_hash::FxHasher;
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
    pub primitive_id : Option< usize >,
    /// A hash map storing the locations of uniform variables in the program.
    /// The keys are the names of the uniforms.
    pub locations : &'a FxHashMap< String, Option< gl::WebGlUniformLocation > >,
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

    /// Returns `true` if the material's uniform data has changed and needs
    /// to be re-uploaded to the GPU via [`upload_on_state_change`].
    fn needs_update( &self ) -> bool;

    /// Called by the renderer after [`upload_on_state_change`] to clear the dirty flag.
    /// Implementations should use interior mutability (e.g. `Cell<bool>`) to reset
    /// the flag from a shared reference.
    ///
    /// Default implementation does nothing, which causes [`upload_on_state_change`]
    /// to be called every frame. Override this together with [`needs_update`]
    /// to enable incremental updates.
    fn clear_needs_update( &self ) {}

    /// Returns the base texture unit for IBL textures.
    ///
    /// Enables IBL usage for the material if this returns `Some`.
    /// Also your shader must contain these texture uniforms:
    ///
    /// ```glsl
    /// uniform samplerCube irradianceTexture;
    /// uniform samplerCube prefilterEnvMap;
    /// uniform sampler2D integrateBRDF;
    /// ```
    ///
    /// You can put them under `#ifdef USE_IBL` for conditional compilation.
    ///
    /// IBL will use 3 consequtive texture units based on what is returned by this function,
    /// for example, if this function returns `Some(4)`, then it will use texture units `4`, `5`, and `6`
    /// for the aforementioned texture uniforms.
    fn get_ibl_base_texture_unit( &self ) -> Option< u32 >
    {
      None
    }

    /// Returns reference to [`ProgramInfo`] with shader locations and used [`ShaderProgram`]
    fn make_shader_program( &self, gl : &gl::WebGl2RenderingContext, program : &gl::WebGlProgram ) -> Box< dyn ShaderProgram >;

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
      let mut hasher = FxHasher::default();
      self.get_vertex_shader().hash( &mut hasher );
      self.get_fragment_shader().hash( &mut hasher );
      self.get_defines_str().hash( &mut hasher );
      hasher.finish()
    }

    /// Configures the position of the uniform texture samplers in the shader program.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    fn configure
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      ctx : &MaterialUploadContext< '_ >
    );

    /// Uploads the material properties to the GPU as uniforms. Use this when material state is changed.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `locations`: A hash map of uniform locations in the shader program.
    fn upload_on_state_change
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      ctx : &MaterialUploadContext< '_ >
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
      _ctx : &MaterialUploadContext< '_ >
    )
    -> Result< (), gl::WebglError >
    {
      Ok( () )
    }

    /// Activates and binds all material textures to their assigned texture units,
    /// and uploads sampler parameters (filtering, wrapping).
    ///
    /// Each texture must be bound to the same unit that was assigned in [`configure`].
    /// Implementations **must** call `gl.active_texture( gl::TEXTURE0 + unit )` before
    /// binding each texture to ensure correct unit targeting.
    ///
    /// The renderer may bind additional textures (e.g. IBL) to higher units after
    /// this call, so implementations should only use the units they configured.
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
