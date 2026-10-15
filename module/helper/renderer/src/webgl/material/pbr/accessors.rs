//! `PbrMaterial`'s property getters and setters. They live in a child layer of `pbr` to keep
//! `pbr.rs` to the material's state, defines and `Material` impl; the fields they touch are
//! `pub( super )` in `pbr`, so they stay invisible outside it.

mod private
{
  use crate::webgl::material::{ AlphaMode, PbrMaterial, TextureInfo };
  use minwebgl as gl;
  use rustc_hash::FxHashMap;

  impl PbrMaterial
  {
    /// Enables or disables Image-Based Lighting (IBL) for this material.
    /// If the value changes, the shader program will be marked for recompilation.
    pub fn need_use_ibl_set( &mut self, value : bool )
    {
      if value != self.need_use_ibl
      {
        self.needs_recompile.set( true );
      }
      self.need_use_ibl = value;
    }

    /// Returns whether Image-Based Lighting (IBL) is enabled for this material.
    pub fn need_use_ibl( &self ) -> bool
    {
      self.need_use_ibl
    }

    /// Sets the base color texture.
    pub fn base_color_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.base_color_texture = value;
      self.defines_update();
    }

    /// Returns the base color texture.
    pub fn base_color_texture( &self ) -> Option< &TextureInfo >
    {
      self.base_color_texture.as_ref()
    }

    /// Sets the metallic roughness texture.
    pub fn metallic_roughness_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.metallic_roughness_texture = value;
      self.defines_update();
    }

    /// Returns the metallic roughness texture.
    pub fn metallic_roughness_texture( &self ) -> Option< &TextureInfo >
    {
      self.metallic_roughness_texture.as_ref()
    }

    /// Sets the normal texture.
    pub fn normal_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.normal_texture = value;
      self.defines_update();
    }

    /// Returns the normal texture.
    pub fn normal_texture( &self ) -> Option< &TextureInfo >
    {
      self.normal_texture.as_ref()
    }

    /// Sets the occlusion texture.
    pub fn occlusion_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.occlusion_texture = value;
      self.defines_update();
    }

    /// Returns the occlusion texture.
    pub fn occlusion_texture( &self ) -> Option< &TextureInfo >
    {
      self.occlusion_texture.as_ref()
    }

    /// Sets the emissive texture.
    pub fn emissive_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.emissive_texture = value;
      self.defines_update();
    }

    /// Returns the emissive texture.
    pub fn emissive_texture( &self ) -> Option< &TextureInfo >
    {
      self.emissive_texture.as_ref()
    }

    /// Sets the specular texture.
    pub fn specular_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.specular_texture = value;
      self.defines_update();
    }

    /// Returns the specular texture.
    pub fn specular_texture( &self ) -> Option< &TextureInfo >
    {
      self.specular_texture.as_ref()
    }

    /// Sets the specular color texture.
    pub fn specular_color_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.specular_color_texture = value;
      self.defines_update();
    }

    /// Returns the specular color texture.
    pub fn specular_color_texture( &self ) -> Option< &TextureInfo >
    {
      self.specular_color_texture.as_ref()
    }

    /// Sets the light map texture.
    pub fn light_map_set( &mut self, value : Option< TextureInfo > )
    {
      self.light_map = value;
      self.defines_update();
    }

    /// Returns the light map texture.
    pub fn light_map( &self ) -> Option< &TextureInfo >
    {
      self.light_map.as_ref()
    }

    /// Sets the clearcoat factor.
    pub fn clearcoat_factor_set( &mut self, value : Option< f32 > )
    {
      self.clearcoat_factor = value;
      self.defines_update();
    }

    /// Returns the clearcoat factor.
    pub fn clearcoat_factor( &self ) -> Option< f32 >
    {
      self.clearcoat_factor
    }

    /// Sets the clearcoat texture.
    pub fn clearcoat_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.clearcoat_texture = value;
      self.defines_update();
    }

    /// Returns the clearcoat texture.
    pub fn clearcoat_texture( &self ) -> Option< &TextureInfo >
    {
      self.clearcoat_texture.as_ref()
    }

    /// Sets the clearcoat roughness factor.
    pub fn clearcoat_roughness_factor_set( &mut self, value : Option< f32 > )
    {
      self.clearcoat_roughness_factor = value;
      self.defines_update();
    }

    /// Returns the clearcoat roughness factor.
    pub fn clearcoat_roughness_factor( &self ) -> Option< f32 >
    {
      self.clearcoat_roughness_factor
    }

    /// Sets the clearcoat roughness texture.
    pub fn clearcoat_roughness_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.clearcoat_roughness_texture = value;
      self.defines_update();
    }

    /// Returns the clearcoat roughness texture.
    pub fn clearcoat_roughness_texture( &self ) -> Option< &TextureInfo >
    {
      self.clearcoat_roughness_texture.as_ref()
    }

    /// Sets the clearcoat normal texture.
    pub fn clearcoat_normal_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.clearcoat_normal_texture = value;
      self.defines_update();
    }

    /// Returns the clearcoat normal texture.
    pub fn clearcoat_normal_texture( &self ) -> Option< &TextureInfo >
    {
      self.clearcoat_normal_texture.as_ref()
    }

    /// Sets the anisotropy strength.
    pub fn anisotropy_strength_set( &mut self, value : Option< f32 > )
    {
      self.anisotropy_strength = value;
      self.defines_update();
    }

    /// Returns the anisotropy strength.
    pub fn anisotropy_strength( &self ) -> Option< f32 >
    {
      self.anisotropy_strength
    }

    /// Sets the anisotropy texture.
    pub fn anisotropy_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.anisotropy_texture = value;
      self.defines_update();
    }

    /// Returns the anisotropy texture.
    pub fn anisotropy_texture( &self ) -> Option< &TextureInfo >
    {
      self.anisotropy_texture.as_ref()
    }

    /// Sets the alpha mode.
    pub fn alpha_mode_set( &mut self, value : AlphaMode )
    {
      self.alpha_mode = value;
      self.defines_update();
    }

    /// Returns the alpha mode.
    pub fn alpha_mode( &self ) -> AlphaMode
    {
      self.alpha_mode
    }

    /// Sets the specular factor.
    pub fn specular_factor_set( &mut self, value : Option< f32 > )
    {
      self.specular_factor = value;
      self.defines_update();
    }

    /// Returns the specular factor.
    pub fn specular_factor( &self ) -> Option< f32 >
    {
      self.specular_factor
    }

    /// Sets the specular color factor.
    pub fn specular_color_factor_set( &mut self, value : Option< gl::F32x3 > )
    {
      self.specular_color_factor = value;
      self.defines_update();
    }

    /// Returns the specular color factor.
    pub fn specular_color_factor( &self ) -> Option< gl::F32x3 >
    {
      self.specular_color_factor
    }

    /// Added the specified name and value is #define directive to the material
    pub fn vertex_define_add< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      self.vertex_defines.insert( name.into(), value.into() );
      self.defines_update();
    }

    /// Added the specified name and value is #define directive to the material
    pub fn fragment_define_add< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      self.fragment_defines.insert( name.into(), value.into() );
      self.defines_update();
    }

    /// Added the specified name and value is #define directive to the material
    pub fn define_add< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      let name = name.into();
      let value = value.into();
      self.vertex_defines.insert( name.clone(), value.clone() );
      self.fragment_defines.insert( name, value );
      self.defines_update();
    }

    /// Returns an immutable reference to the local vertex defines map
    pub fn vertex_defines( &self ) -> &FxHashMap< Box< str >, String >
    {
      &self.vertex_defines
    }

    /// Returns an immutable reference to the local fragment defines map
    pub fn fragment_defines( &self ) -> &FxHashMap< Box< str >, String >
    {
      &self.fragment_defines
    }
  }
}

crate::mod_interface!
{
}
