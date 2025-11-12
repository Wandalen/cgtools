mod private
{
  use minwebgl as gl;
  use gl::GL;
  use web_sys::WebGlProgram;
  use rustc_hash::FxHashMap;

  /// Shader program generalization for getting access to used shader locations
  pub trait ShaderProgram : clone_dyn_types::CloneDyn
  {
    /// Returns a reference to the hash map containing uniform locations
    fn get_locations( &self, gl : &GL, program : &gl::WebGlProgram ) -> FxHashMap< String, Option< gl::WebGlUniformLocation > >;

    /// Returns a reference to the hash map containing UBO indices
    fn get_ubo_indices( &self, gl : &GL, program : &gl::WebGlProgram ) -> FxHashMap< String, u32 >;

    /// Get [`ShaderProgram`] type name
    fn get_type_name( &self ) -> &'static str;

    /// Create new boxed clone of self
    fn dyn_clone( &self ) -> Box< dyn ShaderProgram >;
  }

  /// Macros for simplified [`ShaderProgram`] implementation
  #[ macro_export ]
  macro_rules! impl_locations
  {
    ( $program_type:ty, $( $location_name:literal ),* ) =>
    {
      impl Clone for $program_type
      {
        fn clone( &self ) -> Self
        {
          Self
        }
      }

      impl ShaderProgram for $program_type
      {
        #[ allow( unused_variables ) ]
        fn get_locations( &self, gl : &GL, program : &gl::WebGlProgram ) -> FxHashMap< String, Option< gl::WebGlUniformLocation > >
        {
          #[ allow( unused_mut ) ]
          let mut locations = FxHashMap::default();

          $(
            locations.insert( $location_name.to_string(), gl.get_uniform_location( program, $location_name ) );
          )*

          locations
        }

        #[ allow( unused_variables ) ]
        fn get_ubo_indices( &self, _gl : &GL, _program : &gl::WebGlProgram ) -> FxHashMap< String, u32 >
        {
          FxHashMap::default()
        }

        fn get_type_name( &self ) -> &'static str
        {
          stringify!( $program_type )
        }

        fn dyn_clone( &self ) -> Box< dyn ShaderProgram >
        {
          Box::new( self.clone() )
        }
      }
    };
    ( $program_type:ty, $( $location_name:literal ),* -- $( $ubo_name:literal ),* ) =>
    {
      impl Clone for $program_type
      {
        fn clone( &self ) -> Self
        {
          Self
        }
      }

      impl ShaderProgram for $program_type
      {
        #[ allow( unused_variables ) ]
        fn get_locations( &self, gl : &GL, program : &gl::WebGlProgram ) -> FxHashMap< String, Option< gl::WebGlUniformLocation > >
        {
          #[ allow( unused_mut ) ]
          let mut locations = FxHashMap::default();

          $(
            locations.insert( $location_name.to_string(), gl.get_uniform_location( program, $location_name ) );
          )*

          locations
        }

        #[ allow( unused_variables ) ]
        fn get_ubo_indices( &self, gl : &GL, program : &gl::WebGlProgram ) -> FxHashMap< String, u32 >
        {
          #[ allow( unused_mut ) ]
          let mut ubo_indices = FxHashMap::default();

          $(
            ubo_indices.insert( $ubo_name.to_string(), gl.get_uniform_block_index( program, $ubo_name ) );
          )*

          ubo_indices
        }

        fn get_type_name( &self ) -> &'static str
        {
          stringify!( $program_type )
        }

        fn dyn_clone( &self ) -> Box< dyn ShaderProgram >
        {
          Box::new( self.clone() )
        }
      }
    };
  }

  /// An empty shader program.
  ///
  /// This is typically used as a placeholder or for a simple pass-through rendering pipeline.
  pub struct EmptyShader;
  /// A Physically Based Rendering (PBR) shader.
  pub struct PBRShader;
  /// Shader used for drawing background from equirectangular map
  pub struct SkyboxShader;
  /// A Gaussian filter shader
  ///
  /// This type of shader is commonly used for post-processing effects like
  /// blurring, often as part of a bloom effect.
  pub struct GaussianFilterShader;
  /// An Unreal Bloom shader
  ///
  /// This shader implements a bloom effect similar to the one used in the
  /// Unreal Engine, which simulates a camera's lens reacting to bright light.
  pub struct UnrealBloomShader;
  /// A public struct for a Geometry Buffer (GBuffer) shader.
  pub struct GBufferShader;
  /// A public struct for a composite shader.
  pub struct CompositeShader;
  /// A public struct for an outline shader that uses Jump Flood Algorithm (JFA)
  /// to draw outlines around objects.
  ///
  /// This shader is part of a multi-pass JFA outlining technique.
  pub struct JfaOutlineObjectShader;
  /// A public struct for the initialization step of a JFA outline.
  ///
  /// This shader is the first pass of the JFA, which sets up the initial
  /// state for the algorithm.
  pub struct JfaOutlineInitShader;
  /// A public struct for the stepping pass of a JFA outline.
  ///
  /// This shader is used in the iterative step of the JFA to propagate
  /// information and find the nearest edge.
  pub struct JfaOutlineStepShader;
  /// A public struct representing the final JFA outline shader.
  ///
  /// This shader combines the results of the JFA passes to draw the final outline.
  pub struct JfaOutlineShader;
  /// A public struct for an outline shader based on normal and depth buffers.
  ///
  /// This shader is used to render an object's outline by comparing the normal
  /// and depth values of adjacent pixels.
  pub struct NormalDepthOutlineObjectShader;
  /// A public struct representing the final Normal/Depth outline shader.
  ///
  /// This shader uses the Normal and Depth buffers to create the final outline.
  pub struct NormalDepthOutlineShader;
  /// A public struct for the base Normal/Depth outline shader.
  ///
  /// This is likely the first pass that generates the necessary data for the final
  /// Normal/Depth outline.
  pub struct NormalDepthOutlineBaseShader;
  /// A public struct for a shader that draws narrow outlines.
  pub struct NarrowOutlineShader;
  /// A public struct for the initialization step of a wide outline.
  ///
  /// This shader is part of a multi-pass technique to create thick, wide outlines.
  pub struct WideOutlineInitShader;
  /// A public struct for the stepping pass of a wide outline.
  ///
  /// This is the iterative pass that propagates information for a wide outline.
  pub struct WideOutlineStepShader;
  /// A public struct representing the final wide outline shader.
  ///
  /// This shader combines the results of the previous passes to draw the final wide outline.
  pub struct WideOutlineShader;
  /// Gem shader
  pub struct GemShader;

  /// Stores information about a WebGL program, including the program object and the locations of its uniforms.
  /// This struct is intended for use by the renderer.
  pub struct ProgramInfo
  {
    /// The WebGL program object.
    program : gl::WebGlProgram,
    /// A hash map storing the locations of uniform variables in the program.
    /// The keys are the names of the uniforms.
    locations : FxHashMap< String, Option< gl::WebGlUniformLocation > >,
    /// A hash map storing the locations of UBO variables in the program.
    /// The keys are the names of the uniform block.
    ubo_indices : FxHashMap< String, u32 >,
    /// ShaderProgram instace
    shader : Box< dyn ShaderProgram >
  }

  impl ProgramInfo
  {
    /// Creates a new `ProgramInfo` instance.
    pub fn new( gl : &gl::WebGl2RenderingContext, program : &gl::WebGlProgram, shader : Box< dyn ShaderProgram > ) -> Self
    {
      let locations = shader.get_locations( gl, program );
      let ubo_indices = shader.get_ubo_indices( gl, program );

      Self
      {
        program : program.clone(),
        locations,
        ubo_indices,
        shader
      }
    }

    /// Returns a reference to the hash map containing uniform locations.
    pub fn get_locations( &self ) -> &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    {
      &self.locations
    }

    /// Returns a mutable reference to the hash map containing uniform locations.
    pub fn get_locations_mut( &mut self ) ->  &mut FxHashMap< String, Option< gl::WebGlUniformLocation > >
    {
      &mut self.locations
    }

    /// Returns a reference to the hash map containing UBO indices.
    pub fn get_ubo_indices( &self ) -> &FxHashMap< String, u32 >
    {
      &self.ubo_indices
    }

    /// Returns a mutable reference to the hash map containing UBO indices.
    pub fn get_ubo_indices_mut( &mut self ) ->  &mut FxHashMap< String, u32 >
    {
      &mut self.ubo_indices
    }

    /// Binds the WebGL program for use.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.use_program( Some( &self.program ) );
    }

    /// Return inner program
    pub fn get_program( &self ) -> WebGlProgram
    {
      self.program.clone()
    }
  }

  impl Clone for ProgramInfo
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        program : self.program.clone(),
        locations : self.locations.clone(),
        ubo_indices : self.ubo_indices.clone(),
        shader : clone_dyn_types::clone_into_box( &*self.shader )
      }
    }
  }

  impl_locations!
  (
    PBRShader,
    "cameraPosition",
    "viewMatrix",
    "projectionMatrix",

    // Node uniform locations
    "worldMatrix",
    "normalMatrix",

    // Skeleton uniform locations
    "inverseBindMatrices",
    "globalJointTransformMatrices",
    "matricesTextureSize",

    // Light uniform locations
    "pointLights",
    "pointLightsCount",
    "directLights",
    "directLightsCount",
    "spotLights",
    "spotLightsCount",

    // Material uniform  locations
    //// Textures uniform locations
    "metallicRoughnessTexture",
    "baseColorTexture",
    "normalTexture",
    "occlusionTexture",
    "emissiveTexture",
    "specularTexture",
    "specularColorTexture",
    "lightMap",
    //// IBL uniform locations
    "irradianceTexture",
    "prefilterEnvMap",
    "integrateBRDF",
    //// Scalers uniform locations
    "baseColorFactor",
    "metallicFactor",
    "roughnessFactor",
    "normalScale",
    "occlusionStrength",
    "specularFactor",
    "specularColorFactor",
    "emissiveFactor",
    // Luminosity
    "alphaCutoff",
    "exposure"
  );

  impl_locations!
  (
    GemShader,
    "worldMatrix",
    "inverseWorldMatrix",
    "viewMatrix",
    "projectionMatrix",

    "envMap",
    "cubeNormalMap",

    "rayBounces",
    "color",
    "boostFactors",

    "envMapIntensity",
    "rainbowDelta",
    "squashFactor",
    "radius",
    "geometryFactor",
    "absorptionFactor",
    "colorAbsorption",
    "cameraPosition"
  );

  impl_locations!
  (
    SkyboxShader,
    "uEquirectMap",
    "uInvProjection",
    "uInvView"
  );

  impl_locations!
  (
    GaussianFilterShader,
    "sourceTexture",
    "invSize",
    "blurDir",
    "kernel"
  );

  impl_locations!
  (
    UnrealBloomShader,
    "blurTexture0",
    "blurTexture1",
    "blurTexture2",
    "blurTexture3",
    "blurTexture4",

    "bloomStrength",
    "bloomRadius",

    "bloomFactors",
    "bloomTintColors"
  );

  impl_locations!
  (
    EmptyShader,
  );

  impl_locations!
  (
    GBufferShader,
    "worldMatrix",
    "viewMatrix",
    "projectionMatrix",
    "normalMatrix",
    "near_far",
    "albedoTexture",
    "objectId",
    "materialId",
    "objectColor"
  );

  impl_locations!
  (
    CompositeShader,
    "transparentA",
    "transparentB"
  );

  impl_locations!
  (
    JfaOutlineObjectShader,
    "u_projection",
    "u_view",
    "u_model"
  );

  impl_locations!
  (
    JfaOutlineInitShader,
    "u_object_texture"
  );

  impl_locations!
  (
    JfaOutlineStepShader,
    "u_jfa_texture",
    "u_resolution",
    "u_step_size"
  );

  impl_locations!
  (
    JfaOutlineShader,
    "u_object_texture",
    "u_jfa_texture",
    "u_resolution",
    "u_outline_thickness",
    "u_outline_color",
    "u_object_color",
    "u_background_color"
  );

  impl_locations!
  (
    NormalDepthOutlineObjectShader,
    "u_projection",
    "u_view",
    "u_model",
    "u_normal_matrix",
    "near",
    "far"
  );

  impl_locations!
  (
    NormalDepthOutlineShader,
    "u_color_texture",
    "u_depth_texture",
    "u_norm_texture",
    "u_projection",
    "u_resolution",
    "u_outline_thickness",
    "u_background_color"
  );

  impl_locations!
  (
    NormalDepthOutlineBaseShader,
    "sourceTexture",
    "positionTexture",
    "normalTexture",
    "objectColorTexture",
    "projection",
    "resolution",
    "outlineThickness"
  );

  impl_locations!
  (
    NarrowOutlineShader,
    "sourceTexture",
    "objectColorTexture",
    "positionTexture",
    "resolution",
    "outlineThickness"
  );

  impl_locations!
  (
    WideOutlineInitShader,
    "objectColorTexture"
  );

  impl_locations!
  (
    WideOutlineStepShader,
    "jfaTexture",
    "resolution",
    "stepSize"
  );

  impl_locations!
  (
    WideOutlineShader,
    "sourceTexture",
    "objectColorTexture",
    "jfaTexture",
    "resolution"
  );
}

crate::mod_interface!
{
  own use
  {
    EmptyShader,
    GaussianFilterShader,
    UnrealBloomShader,
    PBRShader,
    SkyboxShader,
    GBufferShader,
    CompositeShader,
    JfaOutlineObjectShader,
    JfaOutlineInitShader,
    JfaOutlineStepShader,
    JfaOutlineShader,
    NormalDepthOutlineObjectShader,
    NormalDepthOutlineShader,
    NormalDepthOutlineBaseShader,
    NarrowOutlineShader,
    WideOutlineInitShader,
    WideOutlineStepShader,
    WideOutlineShader,
    GemShader
  };

  orphan use
  {
    ShaderProgram,
    ProgramInfo
  };
}
