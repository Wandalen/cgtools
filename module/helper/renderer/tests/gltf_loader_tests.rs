//! Verifies the glTF loader's asset-URI resolution ( `renderer::webgl::loaders::gltf::asset_uri_resolve` ) —
//! the pure URI-resolution logic extracted from the browser-bound glTF `load` path
//! ( the rest of the loader needs a live GL context and `fetch` ). Origin-side rules
//! live in mingl's `is_self_contained_url`, which this helper delegates to. Relocated
//! from inline `src/webgl/loaders/gltf.rs` per the all-tests-in-tests/ convention;
//! the helper is exported at its module path for exactly this purpose.

use renderer::webgl::loaders::gltf::asset_uri_resolve;

#[ test ]
fn joins_relative_uri_with_folder()
{
  assert_eq!
  (
    asset_uri_resolve( "models", "scene/buffer.bin" ),
    "models/scene/buffer.bin"
  );
}

#[ test ]
fn passes_blob_uri_through()
{
  assert_eq!
  (
    asset_uri_resolve( "models", "blob:https://app.example.com/uuid-1234" ),
    "blob:https://app.example.com/uuid-1234"
  );
}

#[ test ]
fn passes_data_uri_through()
{
  assert_eq!
  (
    asset_uri_resolve( "models", "data:application/octet-stream;base64,Z2xURg==" ),
    "data:application/octet-stream;base64,Z2xURg=="
  );
}

#[ test ]
fn passes_absolute_url_through()
{
  assert_eq!
  (
    asset_uri_resolve( "models", "https://cdn.example.com/textures/t.png" ),
    "https://cdn.example.com/textures/t.png"
  );
}

#[ test ]
fn passes_origin_absolute_path_through()
{
  assert_eq!
  (
    asset_uri_resolve( "models", "/textures/t.png" ),
    "/textures/t.png"
  );
}

#[ test ]
fn empty_folder_yields_origin_absolute_uri()
{
  // Documents the benign empty-folder behavior: origin-absolute and
  // origin-relative forms collapse to the same URL once `url_resolve`
  // joins them against the window origin.
  assert_eq!
  (
    asset_uri_resolve( "", "buffer.bin" ),
    "/buffer.bin"
  );
}
