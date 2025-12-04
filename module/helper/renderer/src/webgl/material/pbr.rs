mod private
{
  use crate::webgl::material::*;
  use minwebgl as gl;
  use gl::GL;
  use mingl::Former;
  use rustc_hash::FxHashMap;
  use crate::webgl::{ Node, program::{ ProgramInfo, ShaderProgram, PBRShader } };
  use std:: { cell::RefCell, rc::Rc };

  /// The source code for the main vertex shader.
  const MAIN_VERTEX_SHADER : &'static str = include_str!( "../shaders/main.vert" );
  /// The source code for the main fragment shader.
  const MAIN_FRAGMENT_SHADER : &'static str = include_str!( "../shaders/main.frag" );

  /// Represents the visual properties of a surface.
  #[ derive( Former, Debug ) ]
  pub struct PBRMaterial
  {
    /// A unique identifier for the material.
    pub id : uuid::Uuid,
    /// Shader program info
    program : ProgramInfo,
    /// The base color factor, multiplied with the base color texture. Defaults to white (1, 1, 1, 1).
    pub base_color_factor : gl::F32x4,
    /// Optional texture providing the base color.
    pub base_color_texture : Option< TextureInfo >,
    /// Scaling factor for the metallic component.
    pub metallic_factor : f32,
    /// Scaling factor for the roughness component.
    pub roughness_factor : f32,
    /// Optional texture providing the metallic and roughness values. Metalness is sampled from the B channel and roughness from the G channel.
    pub metallic_roughness_texture : Option< TextureInfo >,

    /// Scaling factor applied to each normal vector of the normal texture.
    pub normal_scale : f32,
    /// Optional texture containing normal vectors.
    pub normal_texture : Option< TextureInfo >,

    /// Scalar multiplier applied to the AO values sampled from the occlusion texture.
    pub occlusion_strength : f32,
    /// Optional texture providing ambient occlusion values.
    pub occlusion_texture : Option< TextureInfo >,

    /// Optional texture providing the emission color of the material.
    pub emissive_texture : Option< TextureInfo >,
    /// Scaling factor for the emission intensity
    pub emissive_factor : gl::F32x3,

    /// Optional scaling factor for the specular intensity. (KHR_materials_specular extension)
    pub specular_factor : Option< f32 >,
    /// Optional texture providing the specular intensity. (KHR_materials_specular extension)
    pub specular_texture : Option< TextureInfo >,
    /// Optional color factor for the specular highlight. (KHR_materials_specular extension)
    pub specular_color_factor : Option< gl::F32x3 >,
    /// Optional texture providing the specular color. (KHR_materials_specular extension)
    pub specular_color_texture : Option< TextureInfo >,

    /// Optional lightmap texture containing pre-baked lighting (shadows)
    pub light_map : Option< TextureInfo >,

    /// Alpha cutoff value for mask mode. Fragments with alpha below this value are discarded.
    pub alpha_cutoff : f32,
    /// The alpha blending mode for the material. Defaults to `Opaque`.
    pub alpha_mode : AlphaMode,
    /// Determines wheter to draw both or one side of the primitive
    pub double_sided : bool,

    /// Range of distances in which environment map's mipmap switching is applied
    pub mipmap_distance_range : std::ops::Range< f32 >,

    /// Hash map of defines in (value, name) format
    pub vertex_defines : FxHashMap< Box< str >, String >,
    /// Hash map of defines in (value, name) format
    pub fragment_defines : FxHashMap< Box< str >, String >,

    /// Returns answer need use IBL for current material instance or not
    pub need_use_ibl : bool,
    /// Signal for updating material uniforms
    pub need_update : bool
  }

  impl PBRMaterial
  {
    /// Creates new [`PBRMaterial`] with predefined optimal parameters
    pub fn new( gl : &GL ) -> Self
    {
      let vertex_shader_src = MAIN_VERTEX_SHADER;
      let fragment_shader_src = MAIN_FRAGMENT_SHADER;
      let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src )
      .compile_and_link( &gl )
      .unwrap();

      let id = uuid::Uuid::new_v4();
      let program = ProgramInfo::new( gl, &program, PBRShader.dyn_clone() );
      let base_color_factor = gl::F32x4::from( [ 1.0, 1.0, 1.0, 1.0 ] );

      let base_color_texture = Default::default();
      let metallic_factor = 1.0;
      let roughness_factor = 1.0;
      let metallic_roughness_texture = Default::default();

      let normal_scale = 1.0;
      let normal_texture = Default::default();

      let occlusion_strength = 1.0;
      let occlusion_texture = Default::default();

      let emissive_texture = Default::default();
      let emissive_factor = gl::F32x3::from( [ 0.0, 0.0, 0.0 ] );

      let specular_factor = Default::default();
      let specular_texture = Default::default();
      let specular_color_factor = Default::default();
      let specular_color_texture = Default::default();

      let light_map = Default::default();

      let alpha_mode = AlphaMode::default();
      let alpha_cutoff = 0.5;
      let double_sided = false;

      let mipmap_distance_range = 0.0..200.0;

      let vertex_defines = FxHashMap::default();
      let fragment_defines = FxHashMap::default();

      let need_use_ibl = true;

      return Self
      {
        id,
        program,
        base_color_factor,
        base_color_texture,
        metallic_factor,
        roughness_factor,
        metallic_roughness_texture,
        normal_scale,
        normal_texture,
        occlusion_strength,
        occlusion_texture,
        emissive_texture,
        emissive_factor,
        specular_factor,
        specular_texture,
        specular_color_factor,
        specular_color_texture,
        alpha_mode,
        alpha_cutoff,
        double_sided,
        mipmap_distance_range,
        light_map,
        vertex_defines,
        fragment_defines,
        need_use_ibl,
        need_update : true
      };
    }

    /// Added the specified name and value is #define directive to the material
    pub fn add_vertex_define< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      self.vertex_defines.insert( name.into(), value.into() );
    }

    /// Added the specified name and value is #define directive to the material
    pub fn add_fragment_define< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      self.fragment_defines.insert( name.into(), value.into() );
    }

    /// Added the specified name and value is #define directive to the material
    pub fn add_define< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      let name = name.into();
      let value = value.into();
      self.add_vertex_define( name.clone(), value.clone() );
      self.add_fragment_define( name, value );
    }

    /// Generates `#define` directives to be inserted into the fragment shader based on the material's properties.
    fn get_local_defines( &self ) -> String
    {
      let use_base_color_texture = self.base_color_texture.is_some();
      let use_metallic_roughness_texture = self.metallic_roughness_texture.is_some();

      let use_emissive_texture = self.emissive_texture.is_some();

      let use_specular_texture = self.specular_texture.is_some();
      let use_specular_color_texture = self.specular_color_texture.is_some();

      let use_khr_materials_specular = self.specular_factor.is_some()
      || self.specular_color_factor.is_some()
      || use_specular_texture
      || use_specular_color_texture;

      let use_light_map = self.light_map.is_some();

      let use_normal_texture = self.normal_texture.is_some();
      let use_occlusion_texture = self.occlusion_texture.is_some();
      let use_alpha_cutoff = self.alpha_mode == AlphaMode::Mask;

      let mut defines = String::new();
      let add_texture = | defines : &mut String, name : &str, uv_name : &str, info : Option< &TextureInfo > |
      {
        defines.push_str( &format!( "#define {}\n", name ) );
        defines.push_str( &format!( "#define {} vUv_{}\n", uv_name, info.unwrap().uv_position ) );
      };

      // Base color texture related
      if use_base_color_texture
      {
        add_texture( &mut defines, "USE_BASE_COLOR_TEXTURE", "vBaseColorUv", self.base_color_texture.as_ref() );
      }

      // Metallic roughness texture related
      if use_metallic_roughness_texture
      {
        add_texture( &mut defines, "USE_MR_TEXTURE", "vMRUv", self.metallic_roughness_texture.as_ref() );
      }

      // Emission texture related
      if use_emissive_texture
      {
        add_texture( &mut defines, "USE_EMISSION_TEXTURE", "vEmissionUv", self.emissive_texture.as_ref() );
      }

      // KHR_Materials_Specular extension related
      if use_khr_materials_specular
      {
        defines.push_str( "#define USE_KHR_materials_specular\n" );
        if use_specular_texture
        {
          add_texture( &mut defines, "USE_SPECULAR_TEXTURE", "vSpecularUv", self.specular_texture.as_ref() );
        }

        if use_specular_color_texture
        {
          add_texture( &mut defines, "USE_SPECULAR_COLOR_TEXTURE", "vSpecularColorUv", self.specular_color_texture.as_ref() );
        }
      }

      // Normal texture related
      if use_normal_texture
      {
        add_texture( &mut defines, "USE_NORMAL_TEXTURE", "vNormalUv", self.normal_texture.as_ref() );
      }

      // Occlusion texture related
      if use_occlusion_texture
      {
        add_texture( &mut defines, "USE_OCCLUSION_TEXTURE", "vOcclusionUv", self.occlusion_texture.as_ref() );
      }

      if use_alpha_cutoff
      {
        defines.push_str( "#define USE_ALPHA_CUTOFF\n" );
      }

      if use_light_map
      {
        add_texture( &mut defines, "USE_LIGHT_MAP", "vLightMapUv", self.light_map.as_ref() );
      }

      defines
    }

    /// Returns an immutable reference to the local vertex defines map
    pub fn get_vertex_defines( &self ) -> &FxHashMap< Box< str >, String >
    {
      &self.vertex_defines
    }

    /// Returns an immutable reference to the local fragment defines map
    pub fn get_fragment_defines( &self ) -> &FxHashMap< Box< str >, String >
    {
      &self.fragment_defines
    }
  }

  impl Material for PBRMaterial
  {
    fn get_id( &self ) -> uuid::Uuid
    {
      self.id
    }

    fn needs_update( &self ) -> bool
    {
      self.need_update
    }

    fn needs_ibl( &self ) -> bool
    {
      self.can_use_ibl() && self.need_use_ibl
    }

    fn can_use_ibl( &self ) -> bool
    {
      true
    }

    fn get_program_info( &self ) -> &ProgramInfo
    {
      &self.program
    }

    fn get_program_info_mut( &mut self ) -> &mut ProgramInfo
    {
      &mut self.program
    }

    fn configure
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >,
      ibl_base_location : u32,
    )
    {
      let ibl_base_location = ibl_base_location as i32;
      // Assign a texture unit for each type of texture
      gl.uniform1i( locations.get( "metallicRoughnessTexture" ).unwrap().clone().as_ref() , 0 );
      gl.uniform1i( locations.get( "baseColorTexture" ).unwrap().clone().as_ref() , 1 );
      gl.uniform1i( locations.get( "normalTexture" ).unwrap().clone().as_ref() , 2 );
      gl.uniform1i( locations.get( "occlusionTexture" ).unwrap().clone().as_ref() , 3 );
      gl.uniform1i( locations.get( "emissiveTexture" ).unwrap().clone().as_ref() , 4 );
      gl.uniform1i( locations.get( "specularTexture" ).unwrap().clone().as_ref() , 5 );
      gl.uniform1i( locations.get( "specularColorTexture" ).unwrap().clone().as_ref() , 6 );
      gl.uniform1i( locations.get( "lightMap" ).unwrap().clone().as_ref() , 7 );

      gl.uniform1i( locations.get( "irradianceTexture" ).unwrap().clone().as_ref() , ibl_base_location );
      gl.uniform1i( locations.get( "prefilterEnvMap" ).unwrap().clone().as_ref() , ibl_base_location + 1 );
      gl.uniform1i( locations.get( "integrateBRDF" ).unwrap().clone().as_ref() , ibl_base_location + 2 );
    }

    fn upload
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      _node : Rc< RefCell< Node > >,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    ) -> Result< (), gl::WebglError >
    {
      let upload = | loc, value : Option< f32 > | -> Result< (), gl::WebglError >
      {
        if let Some( v ) = value
        {
          gl::uniform::upload( gl, locations.get( loc ).unwrap().clone(), &v )?;
        }
        Ok( () )
      };

      let upload_array = | loc, value : Option< &[ f32 ] > | -> Result< (), gl::WebglError >
      {
        if let Some( v ) = value
        {
          gl::uniform::upload( gl, locations.get( loc ).unwrap().clone(), v )?;
        }
        Ok( () )
      };

      upload( "specularFactor", self.specular_factor )?;

      gl::uniform::upload( gl, locations.get( "baseColorFactor" ).unwrap().clone(), self.base_color_factor.as_slice() )?;
      gl::uniform::upload( gl, locations.get( "metallicFactor" ).unwrap().clone(), &self.metallic_factor )?;
      gl::uniform::upload( gl, locations.get( "roughnessFactor" ).unwrap().clone(), &self.roughness_factor )?;
      gl::uniform::upload( gl, locations.get( "normalScale" ).unwrap().clone(), &self.normal_scale )?;
      gl::uniform::upload( gl, locations.get( "occlusionStrength" ).unwrap().clone(), &self.occlusion_strength )?;
      gl::uniform::upload( gl, locations.get( "alphaCutoff" ).unwrap().clone(), &self.alpha_cutoff )?;
      gl::uniform::upload( gl, locations.get( "emissiveFactor" ).unwrap().clone(), self.emissive_factor.as_slice() )?;
      if let Some( mipmap_distance_range_loc ) = locations.get( "mipmapDistanceRange" )
      {
        let r = &self.mipmap_distance_range;
        gl::uniform::upload( gl, mipmap_distance_range_loc.clone(), &[ r.start, r.end ] )?;
      }

      upload_array( "specularColorFactor", self.specular_color_factor.as_ref().map( | v | v.as_slice() ) )?;

      self.upload_textures( gl );

      Ok( () )
    }

    fn upload_textures( &self, gl : &gl::WebGl2RenderingContext )
    {
      if let Some( ref t ) = self.metallic_roughness_texture { t.upload( gl ); }
      if let Some( ref t ) = self.base_color_texture { t.upload( gl ); }
      if let Some( ref t ) = self.normal_texture { t.upload( gl ); }
      if let Some( ref t ) = self.occlusion_texture { t.upload( gl ); }
      if let Some( ref t ) = self.emissive_texture { t.upload( gl ); }
      if let Some( ref t ) = self.specular_texture { t.upload( gl ); }
      if let Some( ref t ) = self.specular_color_texture { t.upload( gl ); }
      if let Some( ref t ) = self.light_map { t.upload( gl ); }
    }

    fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      let bind = | texture : &Option< TextureInfo >, i |
      {
        if let Some( ref t ) = texture
        {
          gl.active_texture( gl::TEXTURE0 + i );
          t.bind( gl );
        }
      };

      bind( &self.metallic_roughness_texture, 0 );
      bind( &self.base_color_texture, 1 );
      bind( &self.normal_texture, 2 );
      bind( &self.occlusion_texture, 3 );
      bind( &self.emissive_texture, 4 );
      bind( &self.specular_texture, 5 );
      bind( &self.specular_color_texture, 6 );
    }

    fn get_defines_str( &self ) -> String
    {
      let mut result = self.get_local_defines();

      for ( name, value ) in self.vertex_defines.iter()
      {
        result.push_str( &format!( "#define {} {}", name, value ) );
      }

      for ( name, value ) in self.fragment_defines.iter()
      {
        result.push_str( &format!( "#define {} {}", name, value ) );
      }

      result
    }

    fn get_vertex_defines_str( &self ) -> String
    {
      let mut result = String::new();

      for ( name, value ) in self.vertex_defines.iter()
      {
        result.push_str( &format!( "#define {} {}", name, value ) );
      }

      result
    }

    fn get_fragment_defines_str( &self ) -> String
    {
      let mut result = self.get_local_defines();

      for ( name, value ) in self.fragment_defines.iter()
      {
        result.push_str( &format!( "#define {} {}", name, value ) );
      }

      result
    }

    fn get_fragment_shader( &self ) -> String
    {
      MAIN_FRAGMENT_SHADER.into()
    }

    fn get_vertex_shader( &self ) -> String
    {
      MAIN_VERTEX_SHADER.into()
    }

    fn dyn_clone( &self ) -> Box< dyn Material >
    {
      Box::new( self.clone() )
    }

    fn get_alpha_mode( &self ) -> AlphaMode
    {
      self.alpha_mode
    }

    fn get_type_name(&self) -> &'static str
    {
      "PBRMaterial"
    }
  }

  impl Clone for PBRMaterial
  {
    fn clone( &self ) -> Self
    {
      PBRMaterial
      {
        id : uuid::Uuid::new_v4(),
        program : self.program.clone(),
        base_color_factor : self.base_color_factor,
        base_color_texture : self.base_color_texture.clone(),
        metallic_factor : self.metallic_factor,
        roughness_factor : self.roughness_factor,
        metallic_roughness_texture : self.metallic_roughness_texture.clone(),
        normal_scale : self.normal_scale,
        normal_texture : self.normal_texture.clone(),
        occlusion_strength : self.occlusion_strength,
        occlusion_texture : self.occlusion_texture.clone(),
        emissive_texture : self.emissive_texture.clone(),
        emissive_factor : self.emissive_factor,
        specular_factor : self.specular_factor,
        specular_texture : self.specular_texture.clone(),
        specular_color_factor : self.specular_color_factor,
        specular_color_texture : self.specular_color_texture.clone(),
        alpha_cutoff : self.alpha_cutoff,
        alpha_mode : self.alpha_mode,
        double_sided : self.double_sided,
        mipmap_distance_range : self.mipmap_distance_range.clone(),
        light_map : self.light_map.clone(),
        vertex_defines : self.vertex_defines.clone(),
        fragment_defines : self.fragment_defines.clone(),
        need_use_ibl : self.need_use_ibl,
        need_update : self.need_update
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    PBRMaterial
  };
}
