//! The glTF loader's buffer stage : the raw bytes of every buffer a document names,
//! from its GLB binary chunk, from a fetched uri, or zeroed for a meshopt fallback
//! that `meshopt::views_decode` fills afterwards.

mod private
{
  use minwebgl as gl;
  use gl::JsCast;
  use crate::webgl::loaders::gltf::asset_uri_resolve;

  /// Largest meshopt fallback buffer [`buffers_load`] allocates. A fallback holds no data of its
  /// own, so its size is only the `byteLength` the JSON declares : without a limit a few
  /// bytes of JSON could commit gigabytes, and copying such a buffer into wasm memory for
  /// skins or animations would abort the module. Decoded web geometry is far below this.
  const FALLBACK_BUFFER_MAX : usize = 1 << 30;

  /// Allocates the zeroed buffer a meshopt fallback's views decode into.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when the declared length exceeds [`FALLBACK_BUFFER_MAX`]
  /// or the browser refuses the allocation. `Uint8Array::new_with_length` would throw that
  /// `RangeError` straight through the wasm frames, so the array is built through
  /// `Reflect.construct`, which hands the exception back as a value.
  fn fallback_buffer_allocate( gltf_buffer : &gltf::Buffer< '_ > ) -> Result< gl::js_sys::Uint8Array, gl::WebglError >
  {
    use gl::js_sys::{ Array, Reflect, Uint8Array };

    let length = gltf_buffer.length();
    let Some( length ) = u32::try_from( length ).ok().filter( | _ | length <= FALLBACK_BUFFER_MAX ) else
    {
      gl::browser::error!
      (
        "glTF meshopt fallback buffer {} declares {length} bytes, more than the {FALLBACK_BUFFER_MAX} this loader allocates",
        gltf_buffer.index()
      );
      return Err( gl::WebglError::Other( "glTF meshopt fallback buffer is too large" ) );
    };

    let constructor = Uint8Array::new_with_length( 0 ).constructor();
    Reflect::construct( &constructor, &Array::of1( &length.into() ) )
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
    let mut buffers : Vec< gl::js_sys::Uint8Array > = Vec::new();
    let mut blob = gltf_file.blob.take();

    for gltf_buffer in gltf_file.buffers()
    {
      if crate::webgl::loaders::meshopt::buffer_is_fallback( &gltf_buffer )
      {
        buffers.push( fallback_buffer_allocate( &gltf_buffer )? );
        continue;
      }

      match gltf_buffer.source()
      {
        gltf::buffer::Source::Bin =>
        {
          // GLB stores its binary chunk as `buffers[ 0 ]`, the only buffer allowed to omit
          // its uri. Handing the chunk to a later uri-less buffer would hide a malformed file
          // behind misplaced data.
          if gltf_buffer.index() != 0
          {
            gl::browser::error!( "glTF buffer {} has no uri; only buffer 0 may omit it", gltf_buffer.index() );
            return Err( gl::WebglError::Other( "glTF buffer has no data" ) );
          }
          let blob = blob.take().ok_or_else( ||
          {
            gl::browser::error!( "glTF buffer 0 has no uri and the file has no GLB binary chunk" );
            gl::WebglError::Other( "glTF buffer has no data" )
          } )?;
          gl::debug!( "The gltf binary payload is present: {}", blob.len() );
          buffers.push( blob.as_slice().into() );
        },
        gltf::buffer::Source::Uri( uri ) =>
        {
          let path = asset_uri_resolve( folder_path, uri );
          let buffer = gl::file::load( &path ).await
          .map_err( | e |
          {
            gl::browser::error!( "Failed to load gltf buffer '{path}': {e:?}" );
            gl::WebglError::Other( "Failed to load a buffer" )
          } )?;

          gl::debug!
          (
            "Buffer path: {}\n
            \tBuffer length: {}",
            path,
            buffer.len()
          );

          buffers.push( buffer.as_slice().into() );
        }
      }
    }

    Ok( buffers )
  }
}

crate::mod_interface!
{
  own use buffers_load;
}
