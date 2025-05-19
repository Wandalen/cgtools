mod private
{
  use minwebgl as gl;
  use std::collections::HashMap;

  /// Stores information about a WebGL program, including the program object and the locations of its uniforms.
  /// This struct is intended for use by the renderer.
  pub struct ProgramInfo
  {
    /// The WebGL program object.
    program : gl::WebGlProgram,
    /// A hash map storing the locations of uniform variables in the program.
    /// The keys are the names of the uniforms.
    locations : HashMap< String, Option< gl::WebGlUniformLocation > >
  }

  impl ProgramInfo
  {
    /// Creates a new `ProgramInfo` instance.
    ///
    /// * `gl`: The `WebGl2RenderingContext` used to retrieve uniform locations.
    /// * `program`: The compiled WebGL program object.
    pub fn new( gl : &gl::WebGl2RenderingContext, program : gl::WebGlProgram ) -> Self
    {
      let mut locations = HashMap::new();

      let mut add_location = | name : &str |
      {
        locations.insert( name.to_string(), gl.get_uniform_location( &program, name ) );
      };

      // Camera uniform locations
      add_location( "cameraPosition" );
      add_location( "viewMatrix" );
      add_location( "projectionMatrix" );

      // Node uniform locations
      add_location( "worldMatrix" );

      // Material uniform  locations
      //// Textures uniform locations
      add_location( "metallicRoughnessTexture" );
      add_location( "baseColorTexture" );
      add_location( "normalTexture" );
      add_location( "occlusionTexture" );
      add_location( "emissiveTexture" );
      add_location( "specularTexture" );
      add_location( "specularColorTexture" );
      //// IBL uniform locations
      add_location( "irradianceTexture" );
      add_location( "prefilterEnvMap" );
      add_location( "integrateBRDF" );
      //// Scalers uniform locations
      add_location( "baseColorFactor" );
      add_location( "metallicFactor" );
      add_location( "roughnessFactor" );
      add_location( "normalScale" );
      add_location( "occlusionStrength" );
      add_location( "specularFactor" );
      add_location( "specularColorFactor" );

      Self
      {
        program,
        locations
      }
    }

    /// Returns a reference to the hash map containing uniform locations.
    pub fn get_locations( &self ) -> &HashMap< String, Option< gl::WebGlUniformLocation > >
    {
      &self.locations
    }

    /// Returns a mutable reference to the hash map containing uniform locations.
    pub fn get_locations_mut( &mut self ) ->  &mut HashMap< String, Option< gl::WebGlUniformLocation > >
    {
      &mut self.locations
    }

    /// Binds the WebGL program for use.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.use_program( Some( &self.program ) );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ProgramInfo
  };
}