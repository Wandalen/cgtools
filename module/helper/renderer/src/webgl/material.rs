mod private
{
  use minwebgl as gl;
  use crate::webgl::Texture;
  use std:: { cell::RefCell, collections::HashMap, rc::Rc };

  #[ derive( Default, Clone, Copy, PartialEq, Eq ) ]
  pub enum AlphaMode
  {
    #[ default ]
    Opaque,
    Mask,
    Blend
  }

  #[ derive( Default, Clone ) ]
  pub struct TextureInfo
  {
    pub texture : Rc< RefCell< Texture > >,
    pub uv_position : u32
  }

  impl TextureInfo 
  {
    pub fn apply( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.texture.borrow().apply( gl );
    }

    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.texture.borrow().bind( gl );
    }
  }

  #[ derive( Clone ) ]
  pub struct Material
  {
    pub id : uuid::Uuid,
    pub base_color_factor : Option< gl::F32x4 >,
    pub base_color_texture : Option< TextureInfo >,
    pub metallic_factor : Option< f32 >,
    pub roughness_factor : Option< f32 >,
    pub metallic_roughness_texture : Option< TextureInfo >,

    pub normal_scale : Option< f32 >,
    pub normal_texture : Option< TextureInfo >,

    pub occlusion_strength : Option< f32 >,
    pub occlusion_texture : Option< TextureInfo >,

    pub emissive_texture : Option< TextureInfo >,

    pub specular_factor : Option< f32 >,
    pub specular_texture : Option< TextureInfo >,
    pub specular_color_factor : Option< gl::F32x3 >,
    pub specular_color_texture : Option< TextureInfo >,

    pub alpha_cutoff : Option< f32 >,
    pub alpha_mode : AlphaMode
  }

  impl Material
  {
    pub fn get_id( &self ) -> uuid::Uuid
    {
      self.id
    }

    pub fn configure
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      // Assign a texture unit for each type of texture
      gl.uniform1i( locations.get( "metallicRoughnessTexture" ).unwrap().clone().as_ref() , 0 );
      gl.uniform1i( locations.get( "baseColorTexture" ).unwrap().clone().as_ref() , 1 );
      gl.uniform1i( locations.get( "normalTexture" ).unwrap().clone().as_ref() , 2 );
      gl.uniform1i( locations.get( "occlusionTexture" ).unwrap().clone().as_ref() , 3 );
      gl.uniform1i( locations.get( "emissiveTexture" ).unwrap().clone().as_ref() , 4 );
      gl.uniform1i( locations.get( "specularTexture" ).unwrap().clone().as_ref() , 5 );
      gl.uniform1i( locations.get( "specularColorTexture" ).unwrap().clone().as_ref() , 6 );

      gl.uniform1i( locations.get( "irradianceTexture" ).unwrap().clone().as_ref() , 10 );
      gl.uniform1i( locations.get( "prefilterEnvMap" ).unwrap().clone().as_ref() , 11 );
      gl.uniform1i( locations.get( "integrateBRDF" ).unwrap().clone().as_ref() , 12 );
    }

    pub fn apply
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > >
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

      upload( "metallicFactor", self.metallic_factor )?;
      upload( "roughnessFactor", self.roughness_factor )?;
      upload( "normalScale", self.normal_scale )?;
      upload( "occlusionStrength", self.occlusion_strength )?;
      upload( "specularFactor", self.occlusion_strength )?;
      upload_array( "baseColorFactor", self.base_color_factor.as_ref().map( | v | v.as_slice() ) )?;
      upload_array( "specularColorFactor", self.specular_color_factor.as_ref().map( | v | v.as_slice() ) )?;

      self.apply_textures( gl );

      Ok( () )
    }

    pub fn apply_textures( &self, gl : &gl::WebGl2RenderingContext )
    {
      if let Some( ref t ) = self.metallic_roughness_texture { t.apply( gl ); }
      if let Some( ref t ) = self.base_color_texture { t.apply( gl ); }
      if let Some( ref t ) = self.normal_texture { t.apply( gl ); }
      if let Some( ref t ) = self.occlusion_texture { t.apply( gl ); }
      if let Some( ref t ) = self.emissive_texture { t.apply( gl ); }
      if let Some( ref t ) = self.specular_texture { t.apply( gl ); }
      if let Some( ref t ) = self.specular_color_texture { t.apply( gl ); }
    }

    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
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

    /// #define directives to be inserted into the shader
    pub fn get_fragment_defines( &self ) -> String
    {
      let use_pbr = self.base_color_factor.is_some()
      | self.metallic_factor.is_some()
      | self.roughness_factor.is_some()
      | self.metallic_roughness_texture.is_some()
      | self.base_color_texture.is_some();

      let use_base_color_texture = self.base_color_texture.is_some();
      let use_metallic_roughness_texture = self.metallic_roughness_texture.is_some();

      let use_emission_texture = self.emissive_texture.is_some();

      let use_khr_materials_specular = self.specular_factor.is_some()
      | self.specular_color_factor.is_some()
      | self.specular_texture.is_some()
      | self.specular_color_texture.is_some();

      let use_specular_texture = self.specular_texture.is_some();
      let use_specular_color_texture = self.specular_color_texture.is_some();

      let use_normal_texture = self.normal_texture.is_some();
      let use_occlusion_texture = self.occlusion_texture.is_some();
      let use_alpha_cutoff = self.alpha_cutoff.is_some() && self.alpha_mode == AlphaMode::Mask;

      let mut defines = String::new();
      let add_texture = | defines : &mut String, name : &str, uv_name : &str, info : Option< &TextureInfo > |
      {
        defines.push_str( &format!( "#define {}\n", name ) );
        defines.push_str( &format!( "#define {} vUv_{}\n", uv_name, info.unwrap().uv_position ) );
      };

      if use_pbr { defines.push_str( "#define USE_PBR\n" ); }
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
      if use_emission_texture 
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

      defines
    }
  }

  impl Default for Material
  {
    fn default() -> Self
    {
      let id = uuid::Uuid::new_v4();

      let base_color_factor = Some( gl::F32x4::from( [ 1.0, 1.0, 1.0, 1.0 ] ) );
      let base_color_texture = Default::default();
      let metallic_factor = Default::default();
      let roughness_factor = Default::default();
      let metallic_roughness_texture = Default::default();

      let normal_scale = Default::default();
      let normal_texture = Default::default();

      let occlusion_strength = Default::default();
      let occlusion_texture = Default::default();

      let emissive_texture = Default::default();

      let specular_factor = Default::default();
      let specular_texture = Default::default();
      let specular_color_factor = Default::default();
      let specular_color_texture = Default::default();

      let alpha_mode = AlphaMode::default();
      let alpha_cutoff = Default::default();

      return Self
      {
        id,
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
        specular_factor,
        specular_texture,
        specular_color_factor,
        specular_color_texture,
        alpha_mode,
        alpha_cutoff
      };
    }
  }
}


crate::mod_interface!
{
  orphan use
  {
    AlphaMode,
    TextureInfo,
    Material
  };
}