//! The glTF loader's buffer stage : the raw bytes of every buffer a document names,
//! from its GLB binary chunk, from a fetched uri, or zeroed for a meshopt fallback
//! that `meshopt::views_decode` fills afterwards.

mod private
{
  use minwebgl as gl;
  use gl::JsCast;
  use crate::webgl::loaders::gltf_uri::asset_uri_resolve;

  /// Most meshopt fallback bytes [`buffers_load`] allocates for one document, all its fallback
  /// buffers together. A fallback holds no data of its own, so its size is only the
  /// `byteLength` the JSON declares : without a limit a few bytes of JSON could commit
  /// gigabytes, in one buffer or spread over many, and copying such buffers into wasm memory
  /// for skins or animations would abort the module. Decoded web geometry is far below this.
  const FALLBACK_BYTES_MAX : usize = 1 << 30;

  /// Checks, before anything is fetched or allocated, that the meshopt fallback buffers of
  /// `gltf_file` declare at most [`FALLBACK_BYTES_MAX`] bytes together. Their sizes are known
  /// from the JSON alone, so a document over the limit is refused before it downloads its
  /// other buffers.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when the fallback buffers declare more than
  /// [`FALLBACK_BYTES_MAX`] bytes together.
  fn fallback_bytes_check( gltf_file : &gltf::Gltf ) -> Result< (), gl::WebglError >
  {
    let mut total = 0_usize;
    for buffer in gltf_file.buffers().filter( crate::webgl::loaders::meshopt::buffer_is_fallback )
    {
      total = total.saturating_add( buffer.length() );
      if total > FALLBACK_BYTES_MAX
      {
        gl::browser::error!
        (
          "glTF meshopt fallback buffers declare {total} bytes up to buffer {}, more than the {FALLBACK_BYTES_MAX} this loader allocates for one document",
          buffer.index()
        );
        return Err( gl::WebglError::Other( "glTF meshopt fallback buffers are too large" ) );
      }
    }
    Ok( () )
  }

  /// Allocates the zeroed buffer a meshopt fallback's views decode into.
  /// [`fallback_bytes_check`] has already bounded its length.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when the browser refuses the allocation.
  /// `Uint8Array::new_with_length` would throw that `RangeError` straight through the wasm
  /// frames, so the array is built through `Reflect.construct`, which hands the exception back
  /// as a value.
  fn fallback_buffer_allocate( gltf_buffer : &gltf::Buffer< '_ > ) -> Result< gl::js_sys::Uint8Array, gl::WebglError >
  {
    use gl::js_sys::{ Array, Reflect, Uint8Array };

    // At most `FALLBACK_BYTES_MAX`, which fits a `u32`.
    let length_u32 = u32::try_from( gltf_buffer.length() ).unwrap_or( u32::MAX );
    let constructor = Uint8Array::new_with_length( 0 ).constructor();
    Reflect::construct( &constructor, &Array::of1( &length_u32.into() ) )
    .map( JsCast::unchecked_into )
    .map_err( | e |
    {
      gl::browser::error!( "glTF meshopt fallback buffer {}: allocation failed: {e:?}", gltf_buffer.index() );
      gl::WebglError::Other( "glTF meshopt fallback buffer allocation failed" )
    } )
  }

  /// Collects the raw byte payload of every glTF buffer, indexed like the
  /// document's `buffers` : the embedded GLB binary chunk, each URI-addressed
  /// buffer fetched relative to `folder_path`, and a zeroed placeholder for each
  /// meshopt fallback buffer, which [`crate::webgl::loaders::meshopt::views_decode`]
  /// then fills.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when a buffer has no data (a uri-less buffer other
  /// than the GLB chunk), a fetch fails, or a fallback buffer cannot be allocated.
  pub async fn buffers_load
  (
    gltf_file : &mut gltf::Gltf,
    folder_path : &str
  )
  -> Result< Vec< gl::js_sys::Uint8Array >, gl::WebglError >
  {
    fallback_bytes_check( gltf_file )?;

    let mut buffers : Vec< gl::js_sys::Uint8Array > = Vec::new();
    let mut blob = gltf_file.blob.take();

    for gltf_buffer in gltf_file.buffers()
    {
      if crate::webgl::loaders::meshopt::buffer_is_fallback( &gltf_buffer )
      {
        buffers.push( fallback_buffer_allocate( &gltf_buffer )? );
        continue;
      }

      let bytes = match gltf_buffer.source()
      {
        gltf::buffer::Source::Bin => glb_chunk_take( &gltf_buffer, &mut blob )?,
        gltf::buffer::Source::Uri( uri ) => uri_buffer_fetch( folder_path, uri ).await?,
      };
      buffers.push( bytes );
    }

    Ok( buffers )
  }

  /// Whether the loader needs a copy of the buffers in wasm memory. Only skins, morph
  /// targets and animations are read on the CPU, animations only with the `animation`
  /// feature, which builds their reader. Everything else goes straight from the JS buffers to
  /// the GPU, so the copy is skipped when nothing needs it : it costs as much as the decoded
  /// model again, and the wasm heap never shrinks afterwards.
  #[ must_use ]
  pub fn cpu_buffers_needed( gltf_file : &gltf::Gltf ) -> bool
  {
    gltf_file.skins().next().is_some()
    || ( cfg!( feature = "animation" ) && gltf_file.animations().next().is_some() )
    || gltf_file.meshes().any( | m | m.primitives().any( | p | p.morph_targets().next().is_some() ) )
  }

  /// The GLB binary chunk, as the data of the uri-less `gltf_buffer`. GLB stores its
  /// binary chunk as `buffers[ 0 ]`, the only buffer allowed to omit its uri. Handing
  /// the chunk to a later uri-less buffer would hide a malformed file behind misplaced
  /// data. Takes the chunk out of `blob`, so a second claim on it fails.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when `gltf_buffer` is not buffer 0, or the file has
  /// no binary chunk.
  fn glb_chunk_take( gltf_buffer : &gltf::Buffer< '_ >, blob : &mut Option< Vec< u8 > > ) -> Result< gl::js_sys::Uint8Array, gl::WebglError >
  {
    if gltf_buffer.index() != 0
    {
      gl::browser::error!( "glTF buffer {} has no uri; only buffer 0 may omit it", gltf_buffer.index() );
      return Err( gl::WebglError::Other( "glTF buffer other than buffer 0 has no uri" ) );
    }
    let blob = blob.take().ok_or_else( ||
    {
      gl::browser::error!( "glTF buffer 0 has no uri and the file has no GLB binary chunk" );
      gl::WebglError::Other( "glTF buffer 0 has no uri and no GLB binary chunk" )
    } )?;
    gl::debug!( "The glTF binary payload is present: {}", blob.len() );
    Ok( blob.as_slice().into() )
  }

  /// Fetches a uri-addressed glTF buffer, resolved against `folder_path` as
  /// [`asset_uri_resolve`] does.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when the fetch fails.
  async fn uri_buffer_fetch( folder_path : &str, uri : &str ) -> Result< gl::js_sys::Uint8Array, gl::WebglError >
  {
    let path = asset_uri_resolve( folder_path, uri );
    let buffer = gl::file::load( &path ).await
    .map_err( | e |
    {
      gl::browser::error!( "Failed to load glTF buffer '{path}': {e:?}" );
      gl::WebglError::Other( "Failed to load a buffer" )
    } )?;

    gl::debug!
    (
      "Buffer path: {}\n
      \tBuffer length: {}",
      path,
      buffer.len()
    );

    Ok( buffer.as_slice().into() )
  }
}

crate::mod_interface!
{
}

// Both run only inside `gltf::load`.
pub( crate ) use private::{ buffers_load, cpu_buffers_needed };

/// `cpu_buffers_needed`, reachable from `tests/` under `test_internals`. Without the feature
/// this module does not exist.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
pub mod internal
{
  pub use super::private::cpu_buffers_needed;
}
