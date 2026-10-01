//! glTF extension support of the loader: which extensions an asset may require
//! (`required_extensions_check`), structural validation that tolerates the extensions read
//! by hand (`document_validate`), and reading the material extensions the `gltf` crate has
//! no typed accessor (or Cargo feature) for, `KHR_materials_clearcoat` and
//! `KHR_materials_anisotropy`, from `gltf::Material::extension_value`.

mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use gltf::json::Value;
  use minwebgl as gl;
  use crate::webgl::{ Texture, TextureInfo, material::PbrMaterial };

  /// glTF extensions this loader actually implements support for. Kept in sync
  /// with the `gltf`-crate extension Cargo features this crate enables in
  /// `Cargo.toml` ( `[dependencies.gltf].features` ) -- an extension whose
  /// Cargo feature isn't turned on has no typed accessor exposed by the `gltf`
  /// crate at all, so this loader could not act on it even if it were listed
  /// here. Cross-checked against this file's own code, not just the feature
  /// list: `KHR_lights_punctual` is read in [`light_list_get`] / [`light_get`];
  /// `KHR_materials_specular` is read in `materials_create`'s `gltf_m.specular()`
  /// branch. `KHR_materials_clearcoat` / `KHR_materials_anisotropy` have no typed
  /// accessor or Cargo feature in the `gltf` crate; `materials_create` reads their
  /// JSON through `extension_value`.
  const SUPPORTED_EXTENSIONS : &[ &str ] =
  &[
    "KHR_lights_punctual",
    "KHR_materials_specular",
    "KHR_materials_clearcoat",
    "KHR_materials_anisotropy",
  ];

  /// Validates a parsed glTF document's `extensionsRequired` against
  /// `SUPPORTED_EXTENSIONS`, per glTF 2.0's "Specifying Extensions" : a
  /// conformant client MUST refuse to load an asset that requires an extension
  /// it doesn't support, rather than silently proceeding and producing
  /// incomplete/incorrect output ( e.g. silently ignoring
  /// `KHR_materials_transmission` or `KHR_draco_mesh_compression` content ).
  /// Pure check over the parsed document -- no GL calls -- so it can run
  /// immediately after parsing, before any buffer/image/GL work begins, and be
  /// unit-tested without a live `WebGl2RenderingContext`.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` if any entry in `extensionsRequired` is not in
  /// `SUPPORTED_EXTENSIONS` -- the offending extension name and the full
  /// supported list are logged via `gl::browser::error!` before returning,
  /// since `WebglError::Other` itself only carries a static summary.
  pub fn required_extensions_check( gltf_file : &gltf::Gltf ) -> Result< (), gl::WebglError >
  {
    for required in gltf_file.extensions_required()
    {
      if !SUPPORTED_EXTENSIONS.contains( &required )
      {
        gl::browser::error!
        (
          "glTF asset requires unsupported extension '{required}' ( loader supports: {SUPPORTED_EXTENSIONS:?} )"
        );
        return Err( gl::WebglError::Other( "glTF asset requires an unsupported extension" ) );
      }
    }

    Ok( () )
  }

  /// Validates a document parsed with `gltf::Gltf::from_slice_without_validation`: this
  /// loader's `extensionsRequired` rule ( [`required_extensions_check`] ), then the `gltf-json`
  /// structural validation ( index bounds, required fields, ... ) that `gltf::Gltf::from_slice`
  /// would run, minus its own `extensionsRequired` rule.
  ///
  /// That upstream rule checks against `gltf-json`'s compile-time list of extensions it has typed
  /// support for, which lacks extensions this loader reads by hand ( `KHR_materials_clearcoat`,
  /// `KHR_materials_anisotropy` ) and even `KHR_materials_specular`, so a valid asset requiring
  /// one of them could never load. This function replaces it with the same glTF rule against
  /// `SUPPORTED_EXTENSIONS`, so it is complete on its own.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` if the asset requires an unsupported extension or any other
  /// validation error is reported; each one is logged via `gl::browser::error!`, with its JSON
  /// path for structural errors.
  pub fn document_validate( gltf_file : &gltf::Gltf ) -> Result< (), gl::WebglError >
  {
    use gltf::json::validation::Validate;

    required_extensions_check( gltf_file )?;

    let root = gltf_file.document.as_json();
    let mut errors = Vec::new();
    root.validate( root, gltf::json::Path::new, &mut | path, error |
    {
      let path = path();
      if !path.as_str().starts_with( "extensionsRequired" )
      {
        errors.push( ( path, error ) );
      }
    });

    if errors.is_empty()
    {
      return Ok( () );
    }
    for ( path, error ) in &errors
    {
      gl::browser::error!( "invalid glTF at '{path}': {error:?}" );
    }
    Err( gl::WebglError::Other( "glTF document failed validation" ) )
  }

  /// A texture reference inside a material extension object: an index into the asset's
  /// `textures` array and the UV set it samples (`texCoord`, glTF default 0).
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub struct ExtensionTextureRef
  {
    /// Index into the asset's `textures` array.
    pub index : usize,
    /// UV set the texture is sampled with.
    pub tex_coord : u32,
  }

  /// `KHR_materials_clearcoat` values as read from JSON, with the extension's defaults filled in.
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub struct ClearcoatParams
  {
    /// `clearcoatFactor` (default 0).
    pub factor : f32,
    /// `clearcoatRoughnessFactor` (default 0).
    pub roughness_factor : f32,
    /// `clearcoatTexture`.
    pub texture : Option< ExtensionTextureRef >,
    /// `clearcoatRoughnessTexture`.
    pub roughness_texture : Option< ExtensionTextureRef >,
    /// `clearcoatNormalTexture`.
    pub normal_texture : Option< ExtensionTextureRef >,
    /// `clearcoatNormalTexture.scale` (default 1).
    pub normal_scale : f32,
  }

  /// `KHR_materials_anisotropy` values as read from JSON, with the extension's defaults filled in.
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub struct AnisotropyParams
  {
    /// `anisotropyStrength` (default 0).
    pub strength : f32,
    /// `anisotropyRotation` in radians (default 0).
    pub rotation : f32,
    /// `anisotropyTexture`.
    pub texture : Option< ExtensionTextureRef >,
  }

  fn number_get( json : &Value, key : &str, default : f32 ) -> f32
  {
    json.get( key ).and_then( Value::as_f64 ).map_or( default, | v | v as f32 )
  }

  /// Reads a glTF `textureInfo` object; `None` when it has no integer `index`. A `texCoord`
  /// that isn't a non-negative integer reads as the default UV set 0. Both cases are logged.
  fn texture_ref_parse( json : &Value ) -> Option< ExtensionTextureRef >
  {
    let Some( index ) = json.get( "index" ).and_then( Value::as_u64 ).and_then( | v | usize::try_from( v ).ok() )
    else
    {
      gl::warn!( "glTF material extension textureInfo {json} has no valid index, ignoring it" );
      return None;
    };
    let tex_coord = match json.get( "texCoord" )
    {
      None => 0,
      Some( v ) => v.as_u64().and_then( | v | u32::try_from( v ).ok() ).unwrap_or_else( ||
      {
        gl::warn!( "glTF material extension texture {index} has an invalid texCoord {v}, using UV set 0" );
        0
      }),
    };
    Some( ExtensionTextureRef { index, tex_coord } )
  }

  /// Reads a `KHR_materials_clearcoat` extension object.
  #[ must_use ]
  pub fn clearcoat_parse( json : &Value ) -> ClearcoatParams
  {
    let normal = json.get( "clearcoatNormalTexture" );
    ClearcoatParams
    {
      factor : number_get( json, "clearcoatFactor", 0.0 ),
      roughness_factor : number_get( json, "clearcoatRoughnessFactor", 0.0 ),
      texture : json.get( "clearcoatTexture" ).and_then( texture_ref_parse ),
      roughness_texture : json.get( "clearcoatRoughnessTexture" ).and_then( texture_ref_parse ),
      normal_texture : normal.and_then( texture_ref_parse ),
      normal_scale : normal.map_or( 1.0, | n | number_get( n, "scale", 1.0 ) ),
    }
  }

  /// Reads a `KHR_materials_anisotropy` extension object.
  #[ must_use ]
  pub fn anisotropy_parse( json : &Value ) -> AnisotropyParams
  {
    AnisotropyParams
    {
      strength : number_get( json, "anisotropyStrength", 0.0 ),
      rotation : number_get( json, "anisotropyRotation", 0.0 ),
      texture : json.get( "anisotropyTexture" ).and_then( texture_ref_parse ),
    }
  }

  /// Applies `gltf_m`'s `KHR_materials_clearcoat` / `KHR_materials_anisotropy` extension
  /// objects, if present, to `material`. Texture indices resolve against `textures` (the
  /// asset's `textures` array); an index outside it leaves that texture unset.
  pub fn material_layer_extensions_apply
  (
    gltf_m : &gltf::Material< '_ >,
    textures : &[ Rc< RefCell< Texture > > ],
    material : &mut PbrMaterial,
  )
  {
    let texture_info = | r : Option< ExtensionTextureRef > | -> Option< TextureInfo >
    {
      let r = r?;
      let Some( texture ) = textures.get( r.index )
      else
      {
        gl::warn!( "glTF material extension names texture {} of {}, leaving it unset", r.index, textures.len() );
        return None;
      };
      Some( TextureInfo { texture : texture.clone(), uv_position : r.tex_coord } )
    };

    if let Some( json ) = gltf_m.extension_value( "KHR_materials_clearcoat" )
    {
      let cc = clearcoat_parse( json );
      material.clearcoat_factor_set( Some( cc.factor ) );
      material.clearcoat_roughness_factor_set( Some( cc.roughness_factor ) );
      material.clearcoat_texture_set( texture_info( cc.texture ) );
      material.clearcoat_roughness_texture_set( texture_info( cc.roughness_texture ) );
      material.clearcoat_normal_texture_set( texture_info( cc.normal_texture ) );
      material.clearcoat_normal_scale = cc.normal_scale;
    }

    if let Some( json ) = gltf_m.extension_value( "KHR_materials_anisotropy" )
    {
      let an = anisotropy_parse( json );
      material.anisotropy_strength_set( Some( an.strength ) );
      material.anisotropy_rotation = an.rotation;
      material.anisotropy_texture_set( texture_info( an.texture ) );
    }
  }
}

crate::mod_interface!
{
  own use material_layer_extensions_apply;
  own use clearcoat_parse;
  own use anisotropy_parse;
  own use ClearcoatParams;
  own use AnisotropyParams;
  own use ExtensionTextureRef;
  own use required_extensions_check;
  own use document_validate;
}
