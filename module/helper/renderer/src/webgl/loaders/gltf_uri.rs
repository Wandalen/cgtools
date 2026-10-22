//! Resolving the uris a glTF document names for its buffers and images. Shared by
//! `gltf` ( images ) and `gltf_buffers` ( buffers ), so neither imports the other for it.

mod private
{
  use minwebgl as gl;

  /// Resolves a glTF asset `uri` (buffer or image) against the model's `folder_path`.
  ///
  /// URIs that already carry their own location are returned unchanged, because
  /// prefixing `folder_path` would corrupt them:
  /// * absolute / protocol-relative URLs (`http://`, `https://`, `//`),
  /// * self-contained URIs (`blob:`, `data:`),
  /// * origin-absolute paths (leading `/`).
  ///
  /// Everything else is treated as folder-relative and joined with a single `/`.
  ///
  /// When `folder_path` is empty (the glTF was loaded from a bare filename, so it
  /// sits at the origin root) a folder-relative `uri` resolves to `"/{uri}"`. This
  /// is intentional and harmless: `url_resolve` joins both `"/buffer.bin"` and
  /// `"buffer.bin"` against the origin to the same `"{origin}/buffer.bin"`. A glTF
  /// served from a subdirectory must be loaded with that directory in `gltf_path`
  /// (e.g. `"assets/scene.gltf"`), otherwise the glTF fetch itself fails first.
  #[ must_use ]
  pub fn asset_uri_resolve( folder_path : &str, uri : &str ) -> String
  {
    // `gl::file::load` already resolves self-contained URLs and origin-absolute
    // paths against the window origin; only genuinely folder-relative URIs need
    // the model's folder prefix folded in.
    if gl::file::is_self_contained_url( uri ) || uri.starts_with( '/' )
    {
      uri.to_string()
    }
    else
    {
      format!( "{folder_path}/{uri}" )
    }
  }
}

crate::mod_interface!
{
  own use asset_uri_resolve;
}
