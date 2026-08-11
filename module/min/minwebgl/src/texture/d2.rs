use crate::{ web_sys, JsCast, WebglError, js_sys, wasm_bindgen, JsValue, JsFuture, canvas, context, buffer, AsBytes };
use crate::web_sys::
{
  window,
  HtmlImageElement,
  WebGlTexture,
  wasm_bindgen::closure::Closure
};

type GL = web_sys::WebGl2RenderingContext;

// WebGL2 format/filter/wrap parameter constants ( `RGBA`, `LINEAR`, `NEAREST`, `REPEAT`,
// `CLAMP_TO_EDGE`, etc. ) are small enum values fixed by the WebGL2 spec, far below
// `i32::MAX` -- the `texImage2D`/`texParameteri` family requires `i32` per their WebIDL
// `GLint`/`GLenum` signatures, so this narrow, single-purpose conversion point is safe by
// construction for every constant it is called with in this file.
fn param_as_i32( value : u32 ) -> i32
{
  value as i32
}

// Image/video/sprite-sheet pixel dimensions, offsets, and sprite counts handled here
// ultimately feed WebGL calls whose own WebIDL signatures take `GLsizei`/`GLint` ( 32-bit
// signed ) -- real browsers cap canvas/texture/video dimensions at a few tens of thousands
// of pixels ( e.g. `MAX_TEXTURE_SIZE`, a browser's own max canvas area ), so these values
// never approach `i32::MAX` in practice.
fn dim_as_i32( value : u32 ) -> i32
{
  value as i32
}

/// Uploads an image from a URL to a WebGL texture.
///
/// This function creates a new `WebGlTexture` and asynchronously loads an image from the provided URL into it.
/// It uses a `Closure` to handle the `onload` event of an `HtmlImageElement`, ensuring the texture is
/// uploaded only after the image has finished loading.
///
/// # Arguments
///
/// * `gl` - The WebGl2RenderingContext.
/// * `src` - A reference-counted string containing the URL of the image to load.
///
/// # Returns
///
/// A `WebGlTexture` object.
///
/// # Panics
/// Panics if the browser has no `window`/`document`, if an `<img>` element or WebGL texture
/// can't be created, or if the `img` element's `display` style property can't be set.
#[ inline ]
#[ must_use ]
pub fn upload_image_from_path( gl : &GL, src : &str, flip : bool ) -> WebGlTexture
{
  let window = window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );

  let texture = gl.create_texture().expect( "Failed to create a texture" );

  let img_element = document.create_element( "img" )
  .expect( "Can't create img" )
  .dyn_into::< HtmlImageElement >()
  .expect( "Can't convert to HtmlImageElement" );
  img_element.style().set_property( "display", "none" ).expect( "Can't set property" );
  let load_texture : Closure< dyn Fn() > = Closure::new
  (
    {
      let gl = gl.clone();
      let img = img_element.clone();
      let texture = texture.clone();
      move ||
      {
        if flip
        {
          crate::texture::d2::upload( &gl, Some( &texture ), &img );
        }
        else
        {
          crate::texture::d2::upload_no_flip( &gl, Some( &texture ), &img );
        }

        crate::texture::d2::filter_linear( &gl );
        img.remove();
      }
    }
  );

  img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
  img_element.set_src( src );
  load_texture.forget();

  texture
}

/// Uploads an image from HtmlImageElement to a 2D texture.
/// Image format and internal format are assumed to be RGBA unsigned bytes.
/// Flips the texture in Y direction.
///
/// Using HtmlImageElement is recommended, as it is the most natural
/// and the least expensive way to parse images on the web.
///
/// # Panics
/// Panics if the WebGL driver fails to upload the image data to the texture.
#[ inline ]
pub fn upload
(
  gl : &GL,
  texture : Option< &web_sys::WebGlTexture >,
  img : &web_sys::HtmlImageElement
)
{
  gl.bind_texture( GL::TEXTURE_2D, texture );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 1 );
  gl.tex_image_2d_with_u32_and_u32_and_html_image_element
  (
    GL::TEXTURE_2D,
    0,
    param_as_i32( GL::RGBA ),
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    img
  ).expect( "Failed to upload data to texture" );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );
}
/// Represents a sprite sheet containing multiple sprites arranged in rows and columns.
///
/// A sprite sheet is commonly used in 2D game development to manage and optimize
/// rendering of animations or multiple images by storing them in a single texture.
// `examples/minwebgl/sprite_animation/src/main.rs` constructs `SpriteSheet` via a struct
// literal from outside this crate ( `gl::texture::d2::SpriteSheet { sprites_in_row: 8, .. }` );
// `#[non_exhaustive]` would break that established external call-site contract, so the struct
// deliberately stays exhaustive ( `exhaustive_structs` is centrally allowed in the root manifest ).
pub struct SpriteSheet
{
  /// Number of sprites in each row of the sheet
  pub sprites_in_row : u32,

  /// Width of each individual sprite
  pub sprite_width : u32,

  /// Height of each individual sprite
  pub sprite_height : u32,

  /// Total number of sprites to upload
  pub amount : u32,
}

/// Creates a 2D texture from HtmlImageElement.
/// Image format and internal format are assumed to be RGBA unsigned bytes.
/// Flips the texture in Y direction.
/// Returns created texture.
///
/// Using HtmlImageElement is recommended, as it is the most natural
/// and the least expensive way to parse images on the web.
///
/// # Panics
/// Panics if the WebGL driver fails to upload the image data to the texture.
#[ inline ]
#[ must_use ]
pub fn create_and_upload( gl : &GL, img : &web_sys::HtmlImageElement ) -> Option< web_sys::WebGlTexture >
{
  let texture = gl.create_texture()?;

  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 1 );
  gl.tex_image_2d_with_u32_and_u32_and_html_image_element
  (
    GL::TEXTURE_2D,
    0,
    param_as_i32( GL::RGBA ),
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    img
  ).expect( "Failed to upload data to texture" );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );

  Some( texture )
}

/// Uploads an image from HtmlImageElement to a 2D texture.
/// Image format and internal format are assumed to be RGBA unsigned bytes.
/// Does not flip the texture in Y direction.
/// Returns created texture.
///
/// # Panics
/// Panics if the WebGL driver fails to upload the image data to the texture.
#[ inline ]
pub fn upload_no_flip
(
  gl : &GL,
  texture : Option< &web_sys::WebGlTexture >,
  img : &web_sys::HtmlImageElement
)
{
  gl.bind_texture( GL::TEXTURE_2D, texture );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );
  gl.tex_image_2d_with_u32_and_u32_and_html_image_element
  (
    GL::TEXTURE_2D,
    0,
    param_as_i32( GL::RGBA ),
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    img
  ).expect( "Failed to upload data to texture" );
}

/// Creates a 2D texture from HtmlImageElement.
/// Image format and internal format are assumed to be RGBA unsigned bytes.
/// Does not flip the texture in Y direction.
/// Returns created texture.
///
/// # Panics
/// Panics if the WebGL driver fails to upload the image data to the texture.
#[ inline ]
#[ must_use ]
pub fn create_and_upload_no_flip( gl : &GL, img : &web_sys::HtmlImageElement ) -> Option< web_sys::WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );
  gl.tex_image_2d_with_u32_and_u32_and_html_image_element
  (
    GL::TEXTURE_2D,
    0,
    param_as_i32( GL::RGBA ),
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    img
  ).expect( "Failed to upload data to texture" );

  texture
}

/// Update the video texture for each frame in render loop
/// # Parameters
/// - `gl`: Reference to the WebGL rendering context
/// - `texture`: The WebGL texture to update
/// - `video_element`: The HTML video element to source the texture from
///
/// # Behavior
/// - Binds the texture to the current WebGL context
/// - Uploads the current video frame to the texture
///
/// # When it useful
/// - Playing video as a texture
/// - Updating video every frame
///
/// # Panics
/// Panics if the WebGL driver fails to upload the video frame to the texture.
#[ inline ]
pub fn update_video( gl : &GL, texture : &web_sys::WebGlTexture, video_element : &web_sys::HtmlVideoElement )
{
  gl.bind_texture( GL::TEXTURE_2D, Some( texture ) );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_video_element
  (
    GL::TEXTURE_2D,
    0,
    param_as_i32( GL::RGBA ),
    dim_as_i32( video_element.width() ),
    dim_as_i32( video_element.height() ),
    0,
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    video_element
  ).expect( "Failed to upload data to texture" );
}

/// Creates a 2D texture from HtmlImageElement.
/// Get pixel data from the HtmlImageElement using the 2d context of temporary canvas and load it into the texture array element by element.
///
/// # Parameters
/// - `gl`: Reference to the WebGL rendering context
/// - `img`: The HTML image element containing the sprite sheet
///
/// # Returns
/// A `Result` containing the created WebGL texture or a `WebglError`
///
/// # Behavior
/// - Creates a WebGL texture array
/// - Loads image data using a temporary canvas
/// - Splits sprite sheet into individual sprite textures
/// - Configures texture parameters and mipmapping
///
/// # When it useful
/// - Loading sprites
/// - Working with texture arrays
///
/// # Errors
/// Returns `WebglError::FailedToAllocateResource` if the WebGL context fails to allocate the
/// texture, and propagates any `WebglError` from creating the temporary canvas or its 2D context.
///
/// # Panics
/// Panics if the image fails to load, if the temporary canvas's style properties can't be
/// removed, if drawing the image to the temporary canvas fails, or if reading back its pixel
/// data fails.
// `get_image_data` below is `#[cfg(web_sys_unstable_apis)]`-gated at two argument-type
// signatures inside web-sys itself (see BUG-053); `web_sys_unstable_apis` is a raw `--cfg`
// flag, not a Cargo feature, declared via `check-cfg` in the root manifest's
// `[workspace.lints.rust]` so referencing it is not `unexpected_cfgs`.
#[ inline ]
pub async fn upload_sprite( gl : &GL, image_element : &web_sys::HtmlImageElement, sprite_sheet : &SpriteSheet ) -> Result< web_sys::WebGlTexture, WebglError >
{
  let load_promise = js_sys::Promise::new
  (
    &mut | resolve, reject |
    {
      let on_load = wasm_bindgen::prelude::Closure::once_into_js
      (
        move || { resolve.call0( &JsValue::NULL ).unwrap() }
      );

      let on_error = wasm_bindgen::prelude::Closure::once_into_js
      (
        move || { reject.call1( &JsValue::NULL, &JsValue::from_str( "Failed to load image" ) ).unwrap() }
      );

      image_element.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
      image_element.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );
    }
  );

  JsFuture::from( load_promise ).await.unwrap();

  let texture = gl.create_texture().ok_or( WebglError::FailedToAllocateResource( "Sprite texture" ) )?;
  gl.bind_texture( GL::TEXTURE_2D_ARRAY, Some( &texture ) );

  let ( img_width, img_height ) = ( image_element.width(), image_element.height() );

  let image_data =
  {
    let tmp_canvas = canvas::make()?;
    // Remove global canvas properties.
    tmp_canvas.style().remove_property( "width" ).unwrap();
    tmp_canvas.style().remove_property( "height" ).unwrap();
    // Set custom properties.
    tmp_canvas.set_width( img_width );
    tmp_canvas.set_height( img_height );

    // Get 2d context of the temp canvas.
    let ctx = context::from_canvas_2d( &tmp_canvas )?;

    // Draw image to temp canvas.
    ctx.draw_image_with_html_image_element( image_element, 0.0, 0.0 ).unwrap();

    // Get pixel array of the image.
    // Fix(BUG-053): `web_sys::CanvasRenderingContext2d::get_image_data` has two mutually
    // exclusive signatures gated by the `web_sys_unstable_apis` cfg — `f64` args when it's off,
    // `i32` args when it's on — and this workspace's `.cargo/config.toml` sets it on via
    // `[build] rustflags`, EXCEPT that setting is entirely replaced (not merged) whenever a
    // caller sets the `RUSTFLAGS` env var directly, e.g. this project's own Level 1 command
    // `RUSTFLAGS="-D warnings" cargo nextest run --all-features`. A single literal-typed call
    // site can only ever match one of the two, so it must branch on the same cfg web-sys does.
    // Root cause: this exact line has flip-flopped between bare `f64` and `i32` casts across at
    // least 8 distinct commits in git history — each prior "fix" only matched whichever
    // RUSTFLAGS state the fixer happened to build under, immediately breaking the other.
    // Pitfall: `cargo check -p minwebgl` (no RUSTFLAGS override) and
    // `RUSTFLAGS="-D warnings" cargo nextest run --all-features` (this project's own Level 1
    // command) resolve to opposite overloads of the SAME function in the SAME workspace —
    // never assume one invocation style's success implies the other's.
    #[ cfg( web_sys_unstable_apis ) ]
    let data = ctx.get_image_data( 0, 0, dim_as_i32( img_width ), dim_as_i32( img_height ) ).unwrap().data().to_vec();
    #[ cfg( not( web_sys_unstable_apis ) ) ]
    let data = ctx.get_image_data( 0.0, 0.0, f64::from( img_width ), f64::from( img_height ) ).unwrap().data().to_vec();

    tmp_canvas.remove();

    data
  };

  // Allocate memory for the 3D texture.
  gl.tex_storage_3d
  (
    GL::TEXTURE_2D_ARRAY,
    8,
    GL::RGBA8,
    dim_as_i32( sprite_sheet.sprite_width ),
    dim_as_i32( sprite_sheet.sprite_height ),
    dim_as_i32( sprite_sheet.amount )
  );

  // Create a Pixel Buffer Object (PBO) and copy the image data into it.
  let pbo = buffer::create( gl )?;
  gl.bind_buffer( GL::PIXEL_UNPACK_BUFFER, Some( &pbo ) );
  gl.buffer_data_with_js_u8_array
  (
    GL::PIXEL_UNPACK_BUFFER,
    &js_sys::Uint8Array::from( image_data.as_bytes() ),
    GL::STATIC_DRAW
  );

  // Set the pixel store parameters for 3D texture uploads.
  gl.pixel_storei( GL::UNPACK_ROW_LENGTH, dim_as_i32( img_width ) );
  gl.pixel_storei( GL::UNPACK_IMAGE_HEIGHT, dim_as_i32( img_height ) );

  for i in 0..sprite_sheet.amount
  {
    // Calculate the row and column coordinates for the current sprite based on the total number of sprites and their size.
    let col = i % sprite_sheet.sprites_in_row * sprite_sheet.sprite_width;
    let row = i / sprite_sheet.sprites_in_row * sprite_sheet.sprite_height;

    // Set the correct position of the sprite in the PBO.
    gl.pixel_storei( GL::UNPACK_SKIP_PIXELS, dim_as_i32( col ) );
    gl.pixel_storei( GL::UNPACK_SKIP_ROWS, dim_as_i32( row ) );

    // Copy the current sprite data from PBO to a 3D texture.
    gl.tex_sub_image_3d_with_i32(
      GL::TEXTURE_2D_ARRAY,
      0,
      0,
      0,
      dim_as_i32( i ),
      dim_as_i32( sprite_sheet.sprite_width ),
      dim_as_i32( sprite_sheet.sprite_height ),
      1,
      GL::RGBA,
      GL::UNSIGNED_BYTE,
      0
    ).unwrap();
  }

  gl.tex_parameteri( GL::TEXTURE_2D_ARRAY, GL::TEXTURE_MIN_FILTER, param_as_i32( GL::NEAREST ) );
  gl.tex_parameteri( GL::TEXTURE_2D_ARRAY, GL::TEXTURE_MAG_FILTER, param_as_i32( GL::NEAREST ) );

  gl.generate_mipmap( GL::TEXTURE_2D_ARRAY );
  gl.tex_parameteri( GL::TEXTURE_2D_ARRAY, GL::TEXTURE_BASE_LEVEL, 0 );

  Ok( texture )
}

/// Set the default parameters for the texture
/// Sets MAG and MIN filters to LINEAR
/// Set wrap mode for S, R, T dimensions to REPEAT
#[ inline ]
pub fn default_parameters( gl : &GL )
{
  filter_linear( gl );
  wrap_repeat( gl );
}

/// Set the magnification and minification filters to LINEAR
#[ inline ]
pub fn filter_linear( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, param_as_i32( GL::LINEAR ) );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, param_as_i32( GL::LINEAR ) );
}

/// Set the magnification and minification filters to NEAREST
#[ inline ]
pub fn filter_nearest( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, param_as_i32( GL::NEAREST ) );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, param_as_i32( GL::NEAREST ) );
}

/// Set the wrap mode for S, T and R dimensions to REPEAT
#[ inline ]
pub fn wrap_repeat( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, param_as_i32( GL::REPEAT ) );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, param_as_i32( GL::REPEAT ) );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_R, param_as_i32( GL::REPEAT ) );
}

/// Set the wrap mode for S, T and R dimensions to CLAMP_TO_EDGE
#[ inline ]
pub fn wrap_clamp( gl : &GL )
{
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, param_as_i32( GL::CLAMP_TO_EDGE ) );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, param_as_i32( GL::CLAMP_TO_EDGE ) );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_R, param_as_i32( GL::CLAMP_TO_EDGE ) );
}
