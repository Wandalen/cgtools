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

  /// Runs the `gltf-json` structural validation ( index bounds, required fields, ... ) that
  /// `gltf::Gltf::from_slice` would run, minus its `extensionsRequired` rule.
  ///
  /// That rule checks against `gltf-json`'s compile-time list of extensions it has typed
  /// support for, which lacks extensions this loader reads by hand ( `KHR_materials_clearcoat`,
  /// `KHR_materials_anisotropy` ) and even `KHR_materials_specular`, so a valid asset requiring
  /// one of them could never load. [`required_extensions_check`] enforces the same glTF rule
  /// against `SUPPORTED_EXTENSIONS`, and `load` runs it first.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` if any other validation error is reported; each one is logged
  /// via `gl::browser::error!` with its JSON path.
  pub fn document_validate( gltf_file : &gltf::Gltf ) -> Result< (), gl::WebglError >
  {
    use gltf::json::validation::Validate;

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
    // The `gltf` crate has no typed accessors for KHR_materials_clearcoat / KHR_materials_anisotropy,
    // so their JSON is parsed manually via `extension_value`.
    let parse_ext_texture_info = | json : &Value | -> Option< TextureInfo >
    {
      let index = json.get( "index" )?.as_u64()? as usize;
      let uv_position = json.get( "texCoord" ).and_then( Value::as_u64 ).unwrap_or( 0 ) as u32;
      textures.get( index ).map( | t | TextureInfo { texture : t.clone(), uv_position } )
    };

    // KHR_materials_clearcoat
    if let Some( cc ) = gltf_m.extension_value( "KHR_materials_clearcoat" )
    {
      material.clearcoat_factor_set( Some( cc.get( "clearcoatFactor" ).and_then( Value::as_f64 ).unwrap_or( 0.0 ) as f32 ) );
      material.clearcoat_roughness_factor_set( Some( cc.get( "clearcoatRoughnessFactor" ).and_then( Value::as_f64 ).unwrap_or( 0.0 ) as f32 ) );

      if let Some( t ) = cc.get( "clearcoatTexture" )
      {
        material.clearcoat_texture_set( parse_ext_texture_info( t ) );
      }
      if let Some( t ) = cc.get( "clearcoatRoughnessTexture" )
      {
        material.clearcoat_roughness_texture_set( parse_ext_texture_info( t ) );
      }
      if let Some( t ) = cc.get( "clearcoatNormalTexture" )
      {
        material.clearcoat_normal_scale = t.get( "scale" ).and_then( Value::as_f64 ).unwrap_or( 1.0 ) as f32;
        material.clearcoat_normal_texture_set( parse_ext_texture_info( t ) );
      }
    }

    // KHR_materials_anisotropy
    if let Some( an ) = gltf_m.extension_value( "KHR_materials_anisotropy" )
    {
      material.anisotropy_strength_set( Some( an.get( "anisotropyStrength" ).and_then( Value::as_f64 ).unwrap_or( 0.0 ) as f32 ) );
      material.anisotropy_rotation = an.get( "anisotropyRotation" ).and_then( Value::as_f64 ).unwrap_or( 0.0 ) as f32;
      if let Some( t ) = an.get( "anisotropyTexture" )
      {
        material.anisotropy_texture_set( parse_ext_texture_info( t ) );
      }
    }
  }
}

crate::mod_interface!
{
  own use material_layer_extensions_apply;
  own use required_extensions_check;
  own use document_validate;
}
