//! WebGL adapter texture uploads.
//!
//! Extracted from `webgl.rs` to keep that file under the per-source-file size
//! budget. Uploads an `ImageAsset`'s pixels into a texture: synchronously from
//! CPU bytes, or asynchronously through an `HtmlImageElement`.

mod private
{
  use std::rc::Rc;
  use core::cell::RefCell;
  use web_sys::HtmlImageElement;
  use wasm_bindgen::prelude::*;
  use minwebgl as gl;
  use super::super::webgl_helpers::{ GpuResources, texture_filter_apply, texture_wrap_apply };
  use crate::backend::RenderError;
  use crate::types::{ ResourceId, MipmapMode, asset };

  /// Uploads CPU-resident bitmap bytes as a new texture. Gray8 and
  /// GrayAlpha8 are expanded to RGBA8 on the CPU before upload because:
  ///
  ///   1. WebGL1's LUMINANCE / LUMINANCE_ALPHA replicated the stored
  ///      channels across RGB on sample. On WebGL2 they are legacy
  ///      unsized formats backed by R8 / RG8 and sample as
  ///      (L, 0, 0, 1) / (L, 0, 0, A) — grayscale images render red.
  ///
  ///   2. The obvious native GL ES 3.0 fix — R8 / RG8 + TEXTURE_SWIZZLE_*
  ///      — is explicitly *removed* from WebGL2 (spec §6.19):
  ///      TEXTURE_SWIZZLE_R/G/B/A are not valid `texParameteri` names
  ///      and produce INVALID_ENUM.
  ///
  /// CPU expansion costs 4× memory for Gray8 / 2× for GrayAlpha8 at
  /// upload time, which is acceptable for the grayscale images typical
  /// in tilemap content (masks, icons, height fields) and is portable
  /// across WebGL2 implementations without special GL state.
  ///
  /// # Errors
  /// Returns `RenderError::BackendError` if the texture can't be created or the
  /// upload fails.
  pub fn bitmap_texture_upload
  (
    gl : &gl::GL,
    bytes : &[ u8 ],
    width : u32,
    height : u32,
    format : crate::assets::PixelFormat,
    id : ResourceId< asset::Image >,
  ) -> Result< web_sys::WebGlTexture, RenderError >
  {
    let tex = gl.create_texture()
    .ok_or_else( || RenderError::BackendError( "failed to create texture".into() ) )?;

    gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );

    let ( gl_fmt, unpack_alignment, bytes_owned ) : ( u32, i32, Option< Vec< u8 > > ) = match format
    {
      crate::assets::PixelFormat::Rgba8 => ( gl::RGBA, 4, None ),
      // RGB rows are 3*width bytes — may not be 4-aligned, so relax the
      // UNPACK stride to match. Restored below.
      crate::assets::PixelFormat::Rgb8  => ( gl::RGB, 1, None ),
      crate::assets::PixelFormat::Gray8 =>
      {
        let mut rgba = Vec::with_capacity( bytes.len() * 4 );
        for &l in bytes
        {
          rgba.extend_from_slice( &[ l, l, l, 0xFF ] );
        }
        ( gl::RGBA, 4, Some( rgba ) )
      }
      crate::assets::PixelFormat::GrayAlpha8 =>
      {
        let mut rgba = Vec::with_capacity( bytes.len() * 2 );
        for pair in bytes.chunks_exact( 2 )
        {
          let ( l, a ) = ( pair[ 0 ], pair[ 1 ] );
          rgba.extend_from_slice( &[ l, l, l, a ] );
        }
        ( gl::RGBA, 4, Some( rgba ) )
      }
    };

    // Relax UNPACK_ALIGNMENT only when the per-row byte count may not be
    // a multiple of 4 (RGB8 at odd widths). Default 4 is correct for
    // RGBA8 and for the CPU-expanded grayscale paths above.
    if unpack_alignment != 4 { gl.pixel_storei( gl::UNPACK_ALIGNMENT, unpack_alignment ); }

    // Fix(BUG-210): `image_upload_from_path` below uploads through
    // `minwebgl::texture::d2::upload`, which sets `UNPACK_FLIP_Y_WEBGL=1` --
    // an invariant `sprite.vert`/`sprite_batch.vert` rely on directly
    // ( "uploaded with UNPACK_FLIP_Y_WEBGL=1 so uv.y=1 samples image row 0" ).
    // This sync Bitmap path left the flag at its GL default of 0, so a
    // Bitmap-sourced image rendered upside-down through sprite commands --
    // documented but never fixed in this crate's own readme.md ( "WebGL
    // texture upload Y-flip asymmetry" ). Root cause: the two upload paths
    // set unrelated GL state for the same shader-side convention.
    gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );

    let upload_bytes : &[ u8 ] = bytes_owned.as_deref().unwrap_or( bytes );

    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    (
      gl::TEXTURE_2D, 0, gl_fmt as i32,
      width as i32, height as i32, 0,
      gl_fmt, gl::UNSIGNED_BYTE, Some( upload_bytes ),
    )
    .map_err( | e | RenderError::BackendError
    (
      format!( "tex_image_2d failed for image {id:?}: {e:?}" )
    ))?;

    // Restore defaults so later uploads aren't surprised by residual state.
    if unpack_alignment != 4 { gl.pixel_storei( gl::UNPACK_ALIGNMENT, 4 ); }
    gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

    Ok( tex )
  }

  /// Like `gl::texture::d2::image_upload_from_path`, but updates
  /// `GpuTexture.width` / `height` cells once the image loads. `src` is the
  /// URL to load `asset` from ( its path, or a `blob:` URL for encoded bytes );
  /// the id, sampler settings and premultiplied flag come from `asset`.
  ///
  /// # Panics
  /// Panics if there is no `window` / `document`, or if the texture or the
  /// `img` element can't be created.
  pub fn image_upload_from_path
  (
    gl : &gl::GL,
    src : &str,
    asset : &crate::assets::ImageAsset,
    resources : &Rc< RefCell< GpuResources > >,
    generation : u32,
  ) -> web_sys::WebGlTexture
  {
    let ( id, filter, mipmap, wrap, premultiplied ) = ( asset.id, asset.filter, asset.mipmap, asset.wrap, asset.premultiplied );
    let document = web_sys::window().expect( "no window" ).document().expect( "no document" );

    let texture = gl.create_texture().expect( "failed to create texture" );

    let img : HtmlImageElement = document.create_element( "img" )
      .expect( "can't create img" )
      .dyn_into()
      .expect( "not an HtmlImageElement" );
    img.style().set_property( "display", "none" ).expect( "can't hide img" );

    // The browser fires `load` XOR `error` exactly once for a given `set_src`
    // call — there is no retry path here — so FnOnce handlers are the right
    // shape. `Closure::once_into_js` takes ownership of the Rust closure,
    // returns a JsValue that we hand to the img element, and arranges for the
    // captured state (notably the `Rc<RefCell<GpuResources>>` clone in
    // `on_load`) to be freed after the single invocation, or via finalizer if
    // the event never fires and the JS function is GC'd. This is what lets a
    // `WebGlBackend` drop actually release its GPU resources.
    let src_for_load = src.to_owned();
    let on_load = Closure::once_into_js(
    {
      let gl = gl.clone();
      let img = img.clone();
      let texture = texture.clone();
      let resources = Rc::clone( resources );
      move ||
      {
        // `revoke_object_url` is only meaningful for `blob:` URLs created by
        // `minwebgl::blob_create` (`ImageSource::Encoded`); a real
        // `ImageSource::Path` string passed through this same shared closure
        // must never be revoked. Done unconditionally, before the staleness
        // check below: the browser has already decoded the image into `img`
        // by the time `load` fires, so the URL is safe to release regardless
        // of whether this generation is still current — deferring it behind
        // the early return would leak the URL whenever `assets_load` reruns
        // before an `Encoded` image finishes loading.
        if src_for_load.starts_with( "blob:" )
        {
          web_sys::Url::revoke_object_url( &src_for_load ).unwrap();
        }

        // Bail out if `assets_load` ran again before the image finished loading —
        // this closure belongs to a previous cycle and must not touch the fresh
        // texture that now occupies this id.
        if resources.borrow().generation != generation
        {
          img.remove();
          return;
        }

        // A premultiplied image's bytes go up as stored. The browser's default
        // colour conversion ( an ICC profile or `gAMA` tag to sRGB ) treats them
        // as straight colour and lifts an edge texel's colour above its alpha,
        // which the `ONE` source factor composites as a bright halo.
        if premultiplied
        {
          gl.pixel_storei( gl::UNPACK_COLORSPACE_CONVERSION_WEBGL, gl::NONE as i32 );
        }
        gl::texture::d2::upload( &gl, Some( &texture ), &img );
        if premultiplied
        {
          gl.pixel_storei( gl::UNPACK_COLORSPACE_CONVERSION_WEBGL, gl::BROWSER_DEFAULT_WEBGL as i32 );
        }

        // Bind and apply all sampler state now that level 0 is populated. Binding
        // explicitly because upload() may leave a different texture bound, and
        // tex_parameteri / generate_mipmap act on whatever is bound to TEXTURE_2D.
        // Applying filter here (not only at texture creation) ensures the correct
        // mag/min filters are installed on the texture object regardless of any
        // intervening bind changes — belt-and-suspenders for the async path.
        gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
        texture_filter_apply( &gl, &filter, &mipmap );
        texture_wrap_apply( &gl, wrap );
        if !matches!( mipmap, MipmapMode::Off )
        {
          gl.generate_mipmap( gl::TEXTURE_2D );
        }

        if let Some( gpu_tex ) = resources.borrow().texture( id )
        {
          gpu_tex.width.set( img.natural_width() );
          gpu_tex.height.set( img.natural_height() );
        }

        img.remove();
      }
    });

    let src_for_err = src.to_owned();
    let on_error = Closure::once_into_js(
    {
      let img = img.clone();
      move ||
      {
        web_sys::console::error_1
        (
          &format!( "tilemap_renderer: failed to load image from path {src_for_err:?}" ).into()
        );
        // Remove the element so the other (never-fired) handler becomes unreachable
        // and can be GC'd, rather than sitting on a detached img for the lifetime
        // of the document.
        img.remove();

        // See the matching guard in `on_load` above — only revoke URLs this
        // function itself created via a Blob.
        if src_for_err.starts_with( "blob:" )
        {
          web_sys::Url::revoke_object_url( &src_for_err ).unwrap();
        }
      }
    });

    img.set_onload( Some( on_load.unchecked_ref() ) );
    img.set_onerror( Some( on_error.unchecked_ref() ) );
    img.set_src( src );
    // `on_load` / `on_error` are `JsValue`s produced by `Closure::once_into_js`.
    // The img element now holds JS-side references to both functions via its
    // `onload` / `onerror` properties, so dropping the local JsValue bindings
    // here does not free the functions. When either event fires, its Rust
    // closure is dropped (releasing its captures, including the cloned Rc for
    // `on_load`); the other handler — plus the img itself — becomes GC-eligible
    // once the fired handler calls `img.remove()` above.

    texture
  }
}

mod_interface::mod_interface!
{
  own use bitmap_texture_upload;
  own use image_upload_from_path;
}
