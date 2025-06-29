mod private
{
  use minwebgl as gl;
  use std::collections::HashMap;

  pub struct EmptyShader;
  pub struct PBRShader;
  pub struct GaussianFilterShader;
  pub struct UnrealBloomShader;
  pub struct GBufferShader;
  pub struct CompositeShader;
  pub struct JfaOutlineObjectShader;
  pub struct JfaOutlineInitShader;
  pub struct JfaOutlineStepShader;
  pub struct JfaOutlineShader;
  pub struct NormalDepthOutlineObjectShader;
  pub struct NormalDepthOutlineShader;
  pub struct NarrowOutlineShader;
  pub struct WideOutlineInitShader;
  pub struct WideOutlineStepShader;
  pub struct WideOutlineShader;

  /// Stores information about a WebGL program, including the program object and the locations of its uniforms.
  /// This struct is intended for use by the renderer.
  pub struct ProgramInfo< T >
  {
    /// The WebGL program object.
    program : gl::WebGlProgram,
    /// A hash map storing the locations of uniform variables in the program.
    /// The keys are the names of the uniforms.
    locations : HashMap< String, Option< gl::WebGlUniformLocation > >,
    phantom : std::marker::PhantomData< T >
  }

  impl< T > ProgramInfo< T > 
  {
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

  impl ProgramInfo< PBRShader >
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
      add_location( "normalMatrix" );

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
      add_location( "emissiveFactor" );

      // Luminosity
      add_location( "alphaCutoff" );
      add_location( "exposure" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }
  }

  impl ProgramInfo< GaussianFilterShader > 
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

      add_location( "sourceTexture" );
      add_location( "invSize" );
      add_location( "blurDir" );
      add_location( "kernel" );


      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< UnrealBloomShader > 
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

      add_location( "blurTexture0" );
      add_location( "blurTexture1" );
      add_location( "blurTexture2" );
      add_location( "blurTexture3" );
      add_location( "blurTexture4" );

      add_location( "bloomStrength" );
      add_location( "bloomRadius" );

      add_location( "bloomFactors" );
      add_location( "bloomTintColors" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< EmptyShader > 
  {
    /// Creates a new `ProgramInfo` instance.
    ///
    /// * `gl`: The `WebGl2RenderingContext` used to retrieve uniform locations.
    /// * `program`: The compiled WebGL program object.
    pub fn new( program : gl::WebGlProgram ) -> Self
    {
      let locations = HashMap::new();

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< GBufferShader > 
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

      add_location( "worldMatrix" );
      add_location( "viewMatrix" );
      add_location( "projectionMatrix" );
      add_location( "near" );
      add_location( "far" );
      add_location( "albedoTexture" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    } 
  }


  impl ProgramInfo< CompositeShader > 
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

      add_location( "transparentA" );
      add_location( "transparentB" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    } 
  }

  impl ProgramInfo< JfaOutlineObjectShader > 
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

      add_location( "u_projection" );
      add_location( "u_view" );
      add_location( "u_model" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< JfaOutlineInitShader > 
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

      add_location( "u_object_texture" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< JfaOutlineStepShader > 
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

      add_location( "u_jfa_texture" );
      add_location( "u_resolution" );
      add_location( "u_step_size" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< JfaOutlineShader > 
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

      add_location( "u_object_texture" );
      add_location( "u_jfa_texture" );
      add_location( "u_resolution" );
      add_location( "u_outline_thickness" );
      add_location( "u_outline_color" );
      add_location( "u_object_color" );
      add_location( "u_background_color" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< NormalDepthOutlineObjectShader > 
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

      add_location( "u_projection" );
      add_location( "u_view" );
      add_location( "u_model" );
      add_location( "u_normal_matrix" );
      add_location( "near" );
      add_location( "far" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< NormalDepthOutlineShader > 
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

      add_location( "u_color_texture" );
      add_location( "u_depth_texture" );
      add_location( "u_norm_texture" );
      add_location( "u_projection" );
      add_location( "u_resolution" );
      add_location( "u_outline_thickness" );
      add_location( "u_background_color" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< NarrowOutlineShader > 
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

      add_location( "sourceTexture" );
      add_location( "objectColorIdTexture" );
      add_location( "positionTexture" );
      add_location( "resolution" );
      add_location( "outlineThickness" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< WideOutlineInitShader > 
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

      add_location( "objectColorIdTexture" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< WideOutlineStepShader > 
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

      add_location( "jfaTexture" );
      add_location( "resolution" );
      add_location( "stepSize" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }

  impl ProgramInfo< WideOutlineShader > 
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

      add_location( "sourceTexture" );
      add_location( "objectColorIdTexture" );
      add_location( "jfaTexture" );
      add_location( "resolution" );

      Self
      {
        program,
        locations,
        phantom : std::marker::PhantomData
      }
    }    
  }
}

crate::mod_interface!
{
  own use
  {
    EmptyShader,
    GaussianFilterShader,
    UnrealBloomShader,
    PBRShader,
    GBufferShader,
    CompositeShader,
    JfaOutlineObjectShader,
    JfaOutlineInitShader,
    JfaOutlineStepShader,
    JfaOutlineShader,
    NormalDepthOutlineObjectShader,
    NormalDepthOutlineShader,
    NarrowOutlineShader,
    WideOutlineInitShader,
    WideOutlineStepShader,
    WideOutlineShader
  };
  
  orphan use
  {
    ProgramInfo
  };
}