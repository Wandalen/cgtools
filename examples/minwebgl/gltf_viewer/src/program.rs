use std::collections::HashMap;

use minwebgl as gl;


pub struct ProgramInfo
{
  program : gl::WebGlProgram,
  locations : HashMap< String, Option< gl::WebGlUniformLocation > >
}

impl ProgramInfo
{
  pub fn new( gl : &gl::WebGl2RenderingContext, program : gl::WebGlProgram ) -> Self
  {
    let mut locations = HashMap::new();

    let mut add_location = | name : &str |
    {
      locations.insert( name.to_string(), gl.get_uniform_location( &program, name ) );
    };

    // Camera locaitons
    add_location( "cameraPosition" );
    add_location( "viewMatrix" );
    add_location( "projectionMatrix" );

    // Node locations
    add_location( "worldMatrix" );

    // Material locations
    //// Textures
    add_location( "metallicRoughnessTexture" );
    add_location( "baseColorTexture" );
    add_location( "normalTexture" );
    add_location( "occlusionTexture" );
    add_location( "emissiveTexture" );
    add_location( "specularTexture" );
    add_location( "specularColorTexture" );
    //// IBL
    add_location( "irradianceTexture" );
    add_location( "prefilterEnvMap" );
    add_location( "integrateBRDF" );
    //// Scalers
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

  pub fn get_program( &self ) -> &gl::WebGlProgram
  {
    &self.program
  } 

  pub fn get_locations( &self ) -> &HashMap< String, Option< gl::WebGlUniformLocation > >
  {
    &self.locations
  }

  pub fn apply( &self, gl : &gl::WebGl2RenderingContext )
  {
    gl.use_program( Some( &self.program ) );
  }
}