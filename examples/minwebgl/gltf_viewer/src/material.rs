use minwebgl as gl;
use gl::GL;

use crate::{program::ProgramInfo, texture::Texture};


#[ derive( Clone ) ]
pub struct Material< 'a >
{
  pub id : uuid::Uuid,
  fs_defines : String,
  base_color_factor : gl::F32x4,
  base_color_texture : Option< &'a Texture >,
  metallic_factor : f32,
  roughness_factor : f32,
  metallic_roughness_texture : Option< &'a Texture >,
  normal_scale : f32,
  normal_texture : Option< &'a Texture >,
  occlusion_strength : f32,
  occlusion_texture : Option< &'a Texture >,
  emissive_factor : gl::F32x3,
  emissive_texture : Option< &'a Texture >,
  alpha_mode : gltf::material::AlphaMode,
  alpha_cutoff : Option< f32 >,
  double_sided : bool
}

impl< 'a > Material< 'a > 
{
  pub fn new< 'b >
  ( 
    m : gltf::Material< 'b >,
    textures : &'a [ Texture ]
  ) -> Self
  {
    let id = uuid::Uuid::new_v4();
    let mut fs_defines = String::new();
    fs_defines.push_str( "#define USE_PBR\n" );

    let mut add_texture = | t_info : Option< gltf::texture::Info< 'b > >, texture_define, uv_define |
    {
      if let Some( info ) = t_info
      {
        fs_defines.push_str( &format!( "#define {}\n", texture_define ) );
        fs_defines.push_str( &format!( "#define {} vUv_{}\n", uv_define, info.tex_coord() ) );
        Some( &textures[ info.texture().index() ] )
      }
      else 
      {
        None
      }
    };

    let pbr = m.pbr_metallic_roughness();
    let base_color_factor = gl::F32x4::from( pbr.base_color_factor() );
    let base_color_texture = add_texture( pbr.base_color_texture(), "USE_BASE_COLOR_TEXTURE", "vBaseColorUv" );
    let metallic_factor = pbr.metallic_factor();
    let roughness_factor = pbr.roughness_factor();
    let metallic_roughness_texture = add_texture( pbr.metallic_roughness_texture(), "USE_MR_TEXTURE", "vMRUv" );
    let emissive_factor = gl::F32x3::from( m.emissive_factor() );
    let emissive_texture =  add_texture( m.emissive_texture(), "USE_EMISSION_TEXTURE", "vEmissionUv" );
    let mut normal_scale = 1.0;
    let normal_texture =  if let Some( info ) = m.normal_texture()
    {
      fs_defines.push_str( &format!( "#define {}\n", "USE_NORMAL_TEXTURE" ) );
      fs_defines.push_str( &format!( "#define {} vUv_{}\n", "vNormalUv", info.tex_coord() ) );
      normal_scale = info.scale();
      Some( &textures[ info.texture().index() ] )
    }
    else
    {
      None
    };
    let mut occlusion_strength = 1.0;
    let occlusion_texture =  if let Some( info ) = m.occlusion_texture()
    {
      fs_defines.push_str( &format!( "#define {}\n", "USE_OCCLUSION_TEXTURE" ) );
      fs_defines.push_str( &format!( "#define {} vUv_{}\n", "vOcclusionUv", info.tex_coord() ) );
      occlusion_strength = info.strength();
      Some( &textures[ info.texture().index() ] )
    }
    else
    {
      None
    };
    let alpha_mode = m.alpha_mode();
    let alpha_cutoff = m.alpha_cutoff();
    let double_sided = m.double_sided();


    return Self
    {
      id,
      fs_defines,
      base_color_factor,
      base_color_texture,
      metallic_factor,
      roughness_factor,
      metallic_roughness_texture,
      normal_scale,
      normal_texture,
      occlusion_strength,
      occlusion_texture,
      emissive_factor,
      emissive_texture,
      alpha_mode,
      alpha_cutoff,
      double_sided
    };
  }

  pub fn apply
  ( 
    &self, 
    gl : &gl::WebGl2RenderingContext, 
    program_info : &ProgramInfo
  ) -> Result< (), gl::WebglError >
  {
    let locations = program_info.get_locations();

    // Assign a texture unit for each type of texture
    gl.uniform1i( locations.get( "metallicRoughnessTexture" ).unwrap().clone().as_ref() , 0 );
    gl.uniform1i( locations.get( "baseColorTexture" ).unwrap().clone().as_ref() , 1 );
    gl.uniform1i( locations.get( "normalTexture" ).unwrap().clone().as_ref() , 2 );
    gl.uniform1i( locations.get( "occlusionTexture" ).unwrap().clone().as_ref() , 3 );
    gl.uniform1i( locations.get( "emissiveTexture" ).unwrap().clone().as_ref() , 4 );

    gl::uniform::upload( gl, locations.get( "baseColorFactor" ).unwrap().clone(), &self.base_color_factor.to_array() )?;
    gl::uniform::upload( gl, locations.get( "metallicFactor" ).unwrap().clone(), &self.metallic_factor )?;
    gl::uniform::upload( gl, locations.get( "roughnessFactor" ).unwrap().clone(), &self.roughness_factor )?;
    gl::uniform::upload( gl, locations.get( "normalScale" ).unwrap().clone(), &self.normal_scale )?;
    gl::uniform::upload( gl, locations.get( "occlusionStrength" ).unwrap().clone(), &self.occlusion_strength )?;

    self.apply_textures( gl );

    Ok( () )
  }

  pub fn apply_textures( &self, gl : &gl::WebGl2RenderingContext )
  {
    if let Some( t ) = self.metallic_roughness_texture { t.apply( gl ); }
    if let Some( t ) = self.base_color_texture { t.apply( gl ); }
    if let Some( t ) = self.normal_texture { t.apply( gl ); }
    if let Some( t ) = self.occlusion_texture { t.apply( gl ); }
    if let Some( t ) = self.emissive_texture { t.apply( gl ); }
  }

  pub fn bind_textures( &self, gl : &gl::WebGl2RenderingContext )
  {
   
    if let Some( t ) = self.metallic_roughness_texture 
    {  
      gl.active_texture( gl::TEXTURE0 + 0 ); 
      t.bind( gl ); 
    }

    if let Some( t ) = self.base_color_texture 
    { 
      gl.active_texture( gl::TEXTURE0 + 1 );
      t.bind( gl ); 
    }

    if let Some( t ) = self.normal_texture 
    { 
      gl.active_texture( gl::TEXTURE0 + 2 );
      t.bind( gl ); 
    }

    if let Some( t ) = self.occlusion_texture
    { 
      gl.active_texture( gl::TEXTURE0 + 3 );
      t.bind( gl ); 
    }

    if let Some( t ) = self.emissive_texture
    { 
      gl.active_texture( gl::TEXTURE0 + 4 );
      t.bind( gl ); 
    }
  }

  pub fn get_fragment_defines( &self ) -> &str
  {
    &self.fs_defines
  }
}

impl< 'a > Default for Material< 'a >
{
  fn default() -> Self 
  {
    let id = uuid::Uuid::default();
    let fs_defines = String::new();

    let base_color_factor = gl::F32x4::from( [ 1.0, 1.0, 1.0, 1.0 ] );
    let base_color_texture = None;
    let metallic_factor = 1.0;
    let roughness_factor = 1.0;
    let metallic_roughness_texture = None;

    let normal_scale = 1.0;
    let normal_texture = None;

    let occlusion_strength = 1.0;
    let occlusion_texture = None;

    let emissive_factor = gl::F32x3::from( [ 0.0, 0.0, 0.0 ] );
    let emissive_texture = None;

    let alpha_mode = gltf::material::AlphaMode::Opaque;
    let alpha_cutoff = Some( 0.5 );
    let double_sided = false;

    return Self
    {
      id,
      fs_defines,
      base_color_factor,
      base_color_texture,
      metallic_factor,
      roughness_factor,
      metallic_roughness_texture,
      normal_scale,
      normal_texture,
      occlusion_strength,
      occlusion_texture,
      emissive_factor,
      emissive_texture,
      alpha_mode,
      alpha_cutoff,
      double_sided
    };    
  }
}