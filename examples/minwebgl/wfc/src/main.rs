//! Generate with wfc-image and render tile map on quad.
//!
//! The wfc-image implements the Wave Function Collapse (WFC)
//! algorithm to generate new images based on a sample input image; it works
//! by analyzing the input to learn the local patterns (like tiles or
//! overlapping blocks) and the rules of which patterns can appear next
//! to which, then applies these learned constraints to probabilistically
//! "collapse" possibilities on a grid until a consistent, novel image is
//! generated that shares the structural and textural characteristics of
//! the source.
#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ]

use gl::GL;
use image::{ DynamicImage, ImageBuffer, Luma };
use minwebgl as gl;
use ndarray_cg::F32x4x4;
use web_sys::wasm_bindgen::prelude::*;
use minwebgl::dom::image_element_create;
use minwebgl::WebGlVertexArrayObject;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{ HtmlInputElement, HtmlButtonElement, FileReader, Event };
use wfc_algo::{Size, ForbidNothing};
use wfc_image::{ generate_image, wrap::WrapXY, retry::NumTimes };
use ndarray_cg::mat3x3h;

/// Tile map size. Length of square map side (a x a).
/// More than 256x256 is very slow.
/// This example can generate only static size square maps.
const SIZE : usize = 50;

/// The size of the patterns to be analyzed from the input tilemap image.
const PATTERN_SIZE : u32 = 3;

/// A struct to hold the application's state.
/// This replaces the global static variables.
struct ApplicationState 
{
  map : Option< Vec< Vec< u8 > > >,
  pattern_image : Option< DynamicImage >,
}

/// Set load callback for an image with `path` and hide it from the UI.
///
/// This function creates an HTML `<img>` element, appends it to the
/// document's body (initially hidden and positioned off-screen),
/// sets its ID, cross-origin, load callback, `src` attributes,
/// to trigger the browser's loading process.
///
/// # Arguments
///
/// * `path`: URL or origin-relative path of the image to load, used to construct
///   the image URL and set the element's ID.
/// * `on_load_callback`: A closure that will be invoked with a reference to
///   the loaded `HtmlImageElement` when the browser's `load` event fires for the image.
///
/// # Returns
///
/// Returns `Ok(web_sys::HtmlImageElement)` containing the created image element if
/// successful, or `Err(minwebgl::JsValue)`.
///
/// # Side effects
///
/// * An `<img>` element is created and appended to the document's `<body>`.
/// * The element's ID, styles (`visibility: hidden`, `position: absolute`, etc.), `crossorigin`, `onload` callback, and `src` attributes are set.
/// * The browser starts loading the image asynchronously.
fn image_load
(
  path : &str,
  on_load_callback : Box< dyn Fn( &web_sys::HtmlImageElement ) >,
) -> Result< web_sys::HtmlImageElement, minwebgl::JsValue >
{
  // Fix(BUG-338): both `image_element_create` and `set_id` used to be called with the hardcoded
  // literal "tileset.png" instead of `path`, ignoring the parameter entirely for two of its three
  // uses. `image_element_create("tileset.png")` resolves against the app root (no `static/`
  // prefix), so the element's initial `src` pointed at a URL that 404s -- a real, wasted network
  // request fired on every page load, immediately overwritten a few lines below by the correctly
  // computed `url`. Root cause: literal copy-pasted in place of the parameter that was meant to
  // drive it (the doc comment above already claimed `path` was "used to construct the image URL",
  // which was only true for the later `set_src` call).
  // Pitfall: a demo with a single call site can hide a parameter being silently ignored --
  // nothing here fails visibly unless a second caller passes a different `path`.
  let image = image_element_create( path )?;

  let window = web_sys::window()
  .ok_or_else( || JsValue::from_str( "Failed to get window" ) )?;
  let document = window.document()
  .ok_or_else( || JsValue::from_str( "Failed to get document" ) )?;
  let body = document.body()
  .ok_or_else( || JsValue::from_str( "Failed to get body" ) )?;
  let _ = body.append_child( &image );
  // The DOM id stays filename-only (not the full `path`) so it keeps matching the bare-filename
  // ids that `texture_array_prepare`'s `get_element_by_id` lookups already use elsewhere in this
  // file -- only the element-creation `src` bug above needed the full path.
  let id = path.rsplit( '/' ).next().unwrap_or( path );
  image.set_id( id );

  let style = image.style();
  let _ = style.set_property( "visibility", "hidden" );
  let _ = style.set_property( "position", "absolute" );
  let _ = style.set_property( "top", "0" );
  let _ = style.set_property( "width", "10px" );
  let _ = style.set_property( "height", "10px" );
  image.set_cross_origin( Some( "anonymous" ) );
  let img = image.clone();
  let on_load_callback : Closure< dyn Fn() > = Closure::new( move || on_load_callback( &img ) );
  image.set_onload
  (
    Some
    (
      on_load_callback
      .as_ref()
      .unchecked_ref()
    )
  );
  on_load_callback.forget();
  // Fix(BUG-109): joined `path` against `window.location().origin()` alone,
  // discarding the current page's own directory — resolved to the site root
  // instead of this example's own subpath when deployed under one.
  // Root cause: see `mingl::web::resolve_url`'s doc comment — origin never
  // carries a path; relative references must resolve against the document's
  // own directory.
  // Pitfall: don't hand-roll this join — reuse `gl::web::file::url_resolve`,
  // the same helper `gl::dom::image_element_create` now uses internally.
  let href = window.location().href()?;
  let url = gl::web::file::url_resolve( &href, path );
  image.set_src( &url );
  Ok( image )
}

/// Handles the `change` event on the file input element.
fn on_input_change
(
  event : &Event,
  app_state : &Rc< RefCell< ApplicationState > >
)
{
  let Some( target ) = event.target()
  else
  { 
    gl::warn!( "Event target is not present" );
    return;
  };

  let input : HtmlInputElement = target
  .dyn_into()
  .unwrap();

  let Some( file_list ) = input.files()
  else
  { 
    gl::warn!( "Failed to get file list from input" );
    return;
  };
  let Some( file ) = file_list.get( 0 )
  else
  { 
    gl::warn!( "No file selected" );
    return;
  };

  let reader = FileReader::new().unwrap();
  let app_state_clone = Rc::clone( app_state );
  let onload_callback = Closure::< dyn FnMut( _ ) >::new
  (
    move | event : Event |
    {
      let reader = event.target()
      .and_then( | target | target.dyn_into::< FileReader >().ok() );

      if let Some( reader ) = reader
      {
        match reader.result()
        {
          Ok( js_val ) =>
          {
            if let Some( tmx_content ) = js_val.as_string()
            {
              let mut state = app_state_clone.borrow_mut();
              pattern_set( &tmx_content, &mut state );
              map_wfc_image_generate( &mut state );
              tile_map_render( &state );
            }
          },
          _ => gl::warn!( "Can't read input file" )
        }
      }
    }
  );

  reader.set_onload( Some( onload_callback.as_ref().unchecked_ref() ) );
  onload_callback.forget();

  let _ = reader.read_as_text( &file );
}

/// Initializes the file input element for uploading TMX files.
fn input_tilemap_init( app_state : &Rc< RefCell< ApplicationState > > ) -> Result< (), JsValue >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let file_input = document.get_element_by_id( "file-input" )
  .unwrap()
  .dyn_into::< HtmlInputElement >()
  .unwrap();

  let file_input_style = file_input.style();
  let _ = file_input_style.set_property( "position", "absolute" );
  let _ = file_input_style.set_property( "top", "15px" );
  let _ = file_input_style.set_property( "left", "15px" );

  let on_change_callback = Closure::< dyn FnMut( _ ) >::new
  (
    {
      let app_state = Rc::clone( app_state );
      move | e : Event | on_input_change( &e, &app_state )
    }
  );

  file_input.add_event_listener_with_callback( "change", on_change_callback.as_ref().unchecked_ref() )?;
  on_change_callback.forget();

  Ok( () )
}

/// Sets up a button with a click event listener.
fn button_generate_setup( id : &str, top : u32, app_state : &Rc< RefCell< ApplicationState > > )
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let button_element = document.get_element_by_id( id )
  .unwrap()
  .dyn_into::< HtmlButtonElement >()
  .unwrap();

  let button_style = button_element.style();
  let _ = button_style.set_property( "position", "absolute" );
  let _ = button_style.set_property( "top", format!( "{top}px" ).as_str() );
  let _ = button_style.set_property( "left", "15px" );

  let button_callback = Closure::< dyn FnMut( _ ) >::new
  (
    {
      let app_state = Rc::clone( app_state );
      move | _e : Event |
      {
        let mut state = app_state.borrow_mut();
        map_wfc_image_generate( &mut state );
        tile_map_render( &state );
      }
    }
  );

  let _ = button_element.add_event_listener_with_callback
  (
    "click",
    button_callback.as_ref().unchecked_ref()
  );

  button_callback.forget();
}

/// Initializes the application by setting up the browser environment and UI.
fn init()
{
  gl::browser::setup( gl::browser::Config::default() );

  let app_state = Rc::new
  (
    RefCell::new
    (
      ApplicationState 
      {
        map : None,
        pattern_image : None,
      }
    )
  );

  let _ = input_tilemap_init( &app_state );
  button_generate_setup( "generate-wfc-image", 50, &app_state );

  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window.document()
  .expect( "Should have a document" );
  let body_style = document.body()
  .unwrap()
  .style();
  let _ = body_style.set_property( "margin", "0" );
  let _ = body_style.set_property( "padding", "0" );
  let _ = body_style.set_property( "overflow", "hidden" );
  let _ = body_style.set_property( "height", "100%" );

  let app_state_for_load = Rc::clone( &app_state );
  let load = move | _img : &web_sys::HtmlImageElement |
  {
    gl::spawn_local( default_pattern_load( Rc::clone( &app_state_for_load ) ) );
  };

  let _ = image_load( "static/tileset.png", Box::new( load ) );
}

/// Prepares the vertex attributes for rendering a quad.
fn vertex_attributes_prepare() -> WebGlVertexArrayObject
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let position_data: [ f32; 12 ] =
  [
    -1., -1., -1., 1., 1., 1.,
    -1., -1., 1., -1., 1., 1.
  ];

  let uv_data: [ f32; 12 ] =
  [
    0., 1., 0., 0., 1., 0.,
    0., 1., 1., 1., 1., 0.
  ];

  let position_slot = 0;
  let position_buffer = gl::buffer::create( &gl )
  .unwrap();
  gl::buffer::upload( &gl, &position_buffer, &position_data, GL::STATIC_DRAW );

  let uv_slot = 1;
  let uv_buffer = gl::buffer::create( &gl )
  .unwrap();
  gl::buffer::upload( &gl, &uv_buffer, &uv_data, GL::STATIC_DRAW );

  let vao = gl::vao::create( &gl )
  .unwrap();
  gl.bind_vertex_array( Some( &vao ) );
  let position_attr = mingl::VertexAttribute::new( position_slot, mingl::VectorDataType::new( mingl::DataType::F32, 2, 1 ), 0 );
  let uv_attr = mingl::VertexAttribute::new( uv_slot, mingl::VectorDataType::new( mingl::DataType::F32, 2, 1 ), 0 );
  gl::BufferDescriptor::from_vector( position_attr.vector )
  .stride( 2 )
  .offset( position_attr.offset )
  .attribute_pointer( &gl, position_attr.location, &position_buffer )
  .unwrap();
  gl::BufferDescriptor::from_vector( uv_attr.vector )
  .stride( 2 )
  .offset( uv_attr.offset )
  .attribute_pointer( &gl, uv_attr.location, &uv_buffer )
  .unwrap();
  gl.bind_vertex_array( None );

  vao
}

/// Creates a Model-View-Projection (MVP) matrix for the scene.
fn mvp_create() -> F32x4x4
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;
  let aspect_ratio = width / height;

  let perspective_matrix = mat3x3h::perspective_rh_gl
  (
    70.0f32.to_radians(),
    aspect_ratio,
    0.1,
    1000.0
  );

  let t = [ 0.0, 0.0, 0.0 ];
  let translate = mat3x3h::translation( t );

  let s = [ 1.95 / 3.0, 1.95 / 3.0, 1.95 / 3.0 ];
  let scale = mat3x3h::scale( s );

  let eye = [ 0.0, 0.0, 1.0 ];
  let up = [ 0.0, 1.0, 0.0 ];
  let center = [ 0., 0., 0. ];
  let view_matrix = mat3x3h::look_at_rh( eye, center, up );

  perspective_matrix * view_matrix * translate * scale
}

/// Binds an RGBA texture from an image `id` to a specified `texture_id` slot.
fn texture_array_prepare( id : &str, texture_id : u32 ) -> Option< web_sys::WebGlTexture >
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window.document()
  .expect( "Should have a document" );
  let img = document.get_element_by_id( id )?;
  let img = img.dyn_into::< web_sys::HtmlImageElement >()
  .unwrap();

  let width = img.natural_width();
  let layers = img.natural_height() / width;
  let height = img.natural_height() / layers;

  let texture_array = gl.create_texture();
  gl.active_texture( texture_id );
  gl.bind_texture( GL::TEXTURE_2D_ARRAY, texture_array.as_ref() );

  let _ = gl.tex_image_3d_with_html_image_element
  (
    GL::TEXTURE_2D_ARRAY,
    0,
    GL::RGBA as i32,
    width as i32,
    height as i32,
    layers as i32,
    0,
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    &img,
  );

  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_MIN_FILTER,
    GL::NEAREST as i32,
  );
  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_MAG_FILTER,
    GL::NEAREST as i32,
  );

  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_WRAP_S,
    GL::CLAMP_TO_EDGE as i32,
  );
  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_WRAP_T,
    GL::CLAMP_TO_EDGE as i32,
  );

  texture_array
}

/// Binds an R8UI texture from `data` with `size` to a specified `texture_id` slot.
fn texture1u_prepare
(
  data : &[ u8 ],
  size : ( i32, i32 ),
  texture_id : u32,
)
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let texture = gl.create_texture();
  gl.active_texture( texture_id );
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
  (
    GL::TEXTURE_2D,
    0,
    GL::R8UI as i32,
    size.0,
    size.1,
    0,
    GL::RED_INTEGER,
    GL::UNSIGNED_BYTE,
    Some( data ),
  )
  .expect( "Can't load an image" );
  gl.pixel_storei( GL::UNPACK_ALIGNMENT, 1 );

  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
}

/// Renders the tile map on the quad.
fn tile_map_render(app_state : &ApplicationState)
{
  let Some( ref map ) = app_state.map
  else
  {
    return;
  };
  if map.is_empty() || map[ 0 ].is_empty()
  {
    return;
  }

  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src )
  .compile_and_link( &gl )
  .unwrap();
  gl.use_program( Some( &program ) );

  let mvp = mvp_create();
  let mvp_location = gl.get_uniform_location( &program, "mvp" );

  gl::uniform::matrix_upload( &gl, mvp_location, mvp.raw_slice(), false )
  .unwrap();

  let vao = vertex_attributes_prepare();
  gl.bind_vertex_array( Some( &vao ) );
  texture_array_prepare( "tileset.png", GL::TEXTURE0 );

  let size = ( map[ 0 ].len() as i32, map.len() as i32 );
  let data = map.iter()
  .flatten()
  .copied()
  .collect::< Vec< u8 > >();

  texture1u_prepare( &data, size, GL::TEXTURE1 );

  let tiles_location = gl.get_uniform_location( &program, "tiles_sampler" );
  let map_sampler_location = gl.get_uniform_location( &program, "map_sampler" );

  gl.uniform1i( tiles_location.as_ref(), 0 );
  gl.uniform1i( map_sampler_location.as_ref(), 1 );

  let texel_size = [ 1.0 / size.0 as f32, 1.0 / size.1 as f32 ];
  let texel_size_location = gl.get_uniform_location( &program, "texel_size" );
  let _ = gl::uniform::upload( &gl, texel_size_location, texel_size.as_slice() );

  gl.draw_arrays( GL::TRIANGLES, 0, 3 * 2 );
  gl.bind_vertex_array( None );
}

/// Parses and sets the reference pattern for generating the tilemap from the content of a TMX file.
///
/// # Fix(BUG-468)
/// This used to be a chain of 9 raw `.unwrap()` calls over the XML/attribute/CSV parse, so any
/// TMX that didn't match the exact shape this demo's own bundled `island_pattern.tmx` happens to
/// have -- non-CSV layer encoding (e.g. Tiled's own default base64+zlib export), a missing
/// `<layer>`/`<data>` element, a non-numeric attribute -- panicked the whole wasm module instead
/// of failing gracefully, reachable simply by using the demo's own advertised "load your own
/// pattern" file input. Every parse step now returns early with a `gl::warn!` message (mirroring
/// `default_pattern_load`'s own graceful-failure idiom below) and leaves `app_state.pattern_image`
/// unchanged on failure, so a bad upload never corrupts or clears an already-working pattern.
/// Root cause: the original code assumed every uploaded TMX matches the one bundled sample file's
/// exact shape, with no validation that a user-supplied file actually does.
/// Pitfall: a "load your own file" input is untrusted input by definition -- code reachable from it
/// must never `.unwrap()` on the file's structure, no matter how reasonable the expected shape is.
fn pattern_set( tmx_content : &str, app_state : &mut ApplicationState )
{
  let Ok( elem ) = tmx_content.parse::< xml::Element >()
  else
  {
    gl::warn!( "Failed to load pattern: file is not valid XML" );
    return;
  };

  let Some( layer ) = elem.get_child( "layer", None )
  else
  {
    gl::warn!( "Failed to load pattern: TMX file has no <layer> element" );
    return;
  };

  let Some( width ) = layer.attributes.get( &( "width".to_string(), None ) )
  .and_then( | v | v.parse::< u32 >().ok() )
  else
  {
    gl::warn!( "Failed to load pattern: <layer> is missing a valid \"width\" attribute" );
    return;
  };
  let Some( height ) = layer.attributes.get( &( "height".to_string(), None ) )
  .and_then( | v | v.parse::< u32 >().ok() )
  else
  {
    gl::warn!( "Failed to load pattern: <layer> is missing a valid \"height\" attribute" );
    return;
  };

  let Some( data ) = layer.get_children( "data", None )
  .find( | ch | ch.attributes.get( &( "encoding".to_string(), None ) ) == Some( &"csv".to_string() ) )
  else
  {
    gl::warn!( "Failed to load pattern: no CSV-encoded <data> layer found -- re-export from Tiled with Layer Format set to \"CSV\"" );
    return;
  };

  let mut pattern_raw = Vec::new();
  for tile in data.content_str().split( ',' )
  {
    let tile = tile.trim();
    if tile.is_empty()
    {
      // Trailing comma/newline artifacts from Tiled's own CSV export.
      continue;
    }

    let Ok( gid ) = tile.parse::< u32 >()
    else
    {
      gl::warn!( "Failed to load pattern: tile value {tile:?} is not a valid non-negative integer" );
      return;
    };

    // Fix(BUG-469): Tiled's CSV GIDs are 1-based -- GID 0 means "empty cell"
    // and GID 1 is the *first* tileset tile (local index 0). The previous
    // `saturating_sub( 1 )` mapped both GID 0 and GID 1 to the same encoded
    // pixel value 0, so an empty cell and the first tileset tile were
    // silently indistinguishable in the generated pattern image. `u8::MAX`
    // is reserved as the "empty" sentinel instead, kept outside the valid
    // tile-index range (0..=254) `wfc_image::generate_image` learns from.
    // Root cause: `saturating_sub` was chosen only to avoid a GID-0 underflow
    // panic, without considering that it also needed a distinct *encoding*
    // for "empty" rather than collapsing onto the same value as GID 1.
    // Pitfall: a 1-based id space with a reserved zero value can't be turned
    // into a 0-based index via a plain `- 1` (or `saturating_sub( 1 )`) --
    // the reserved value needs its own explicit branch.
    let value = if gid == 0
    {
      u8::MAX
    }
    else
    {
      // Tiled encodes horizontal/vertical/diagonal flip flags in a real
      // GID's high bits, which this demo's tileset format doesn't support;
      // such a GID (or any GID beyond the 255-tile range this `u8`-indexed
      // pattern format supports) is rejected rather than silently
      // misinterpreted as an unrelated tile.
      let Ok( index ) = u8::try_from( gid - 1 )
      else
      {
        gl::warn!( "Failed to load pattern: tile GID {gid} is out of the supported range (max 255 tiles, flipped tiles are not supported)" );
        return;
      };
      index
    };
    pattern_raw.push( value );
  }

  let Some( pattern_buf ) : Option< ImageBuffer< Luma< u8 >, Vec< u8 > > > =
  ImageBuffer::from_vec( width, height, pattern_raw )
  else
  {
    gl::warn!( "Failed to load pattern: tile count does not match width x height" );
    return;
  };
  let pattern_img = DynamicImage::ImageLuma8( pattern_buf );

  app_state.pattern_image = Some( pattern_img );
}

/// Fetches the bundled default TMX pattern, sets it as the reference pattern,
/// and generates the first tile map so the demo works without requiring an upload.
/// Called from `tileset.png`'s load callback so the texture is guaranteed ready
/// by the time `tile_map_render` needs it.
async fn default_pattern_load( app_state : Rc< RefCell< ApplicationState > > )
{
  let Ok( bytes ) = gl::file::load( "static/island_pattern.tmx" ).await
  else
  {
    gl::warn!( "Failed to load default pattern" );
    return;
  };

  let Ok( tmx_content ) = String::from_utf8( bytes )
  else
  {
    gl::warn!( "Default pattern is not valid UTF-8" );
    return;
  };

  let mut state = app_state.borrow_mut();
  if state.pattern_image.is_some()
  {
    // A user upload already set the pattern before this fetch resolved — the
    // default must never clobber an explicit choice.
    return;
  }
  pattern_set( &tmx_content, &mut state );
  map_wfc_image_generate( &mut state );
  tile_map_render( &state );
}

/// Generates a new tile map using the WFC algorithm with the loaded pattern image.
fn map_wfc_image_generate( app_state : &mut ApplicationState )
{
  let Some( ref pattern_img ) = app_state.pattern_image
  else
  {
    return;
  };

  let Ok( map_img ) = generate_image
  (
    pattern_img,
    std::num::NonZero::new( PATTERN_SIZE ).unwrap(),
    Size::try_new( SIZE as u32, SIZE as u32 ).unwrap(),
    &wfc_algo::orientation::ALL,
    WrapXY,
    ForbidNothing,
    NumTimes( 1 )
  )
  else
  {
    return;
  };

  let map_raw : Vec<u8> = map_img.to_luma8().into_raw();
  let map = map_raw.chunks( SIZE )
  .map( <[u8]>::to_vec )
  .collect::< Vec< Vec< _ > > >();

  app_state.map = Some( map );
}

/// Runs the main application logic.
fn app_run()
{
  init();
}

/// The main entry point of the Rust program.
fn main()
{
  app_run();
}

// `pattern_set` is a private fn with no `[lib]` target to reach it from
// `tests/` -- per this workspace's rulebook.md "Test placement" rule, its
// tests live inline instead. Kept as the last item in the file
// (clippy::items_after_test_module).
#[ cfg( test ) ]
mod tests
{
  use super::*;

  fn app_state_empty() -> ApplicationState
  {
    ApplicationState { map : None, pattern_image : None }
  }

  // BUG-468 task/bug/completed/468_wfc_pattern_set_unwrap_chain_panics_on_upload.md --
  // reproducer for `pattern_set` panicking instead of failing gracefully.
  // test_kind: bug_reproducer(BUG-468)
  #[ test ]
  fn pattern_set_rejects_non_csv_encoding_without_panicking()
  {
    // Tiled's own default export encoding (base64+zlib) -- not CSV, and this
    // demo only supports CSV, so this must be rejected gracefully.
    let tmx = r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" orientation="orthogonal" width="2" height="1">
 <layer id="1" name="Layer 1" width="2" height="1">
  <data encoding="base64">AAAAAA==</data>
 </layer>
</map>"#;

    let mut state = app_state_empty();
    pattern_set( tmx, &mut state ); // must not panic

    assert!
    (
      state.pattern_image.is_none(),
      "a non-CSV-encoded TMX must be rejected, not silently accepted"
    );
  }

  // BUG-468 -- malformed XML entirely.
  // test_kind: bug_reproducer(BUG-468)
  #[ test ]
  fn pattern_set_rejects_malformed_xml_without_panicking()
  {
    let mut state = app_state_empty();
    pattern_set( "not xml at all <<<", &mut state ); // must not panic
    assert!( state.pattern_image.is_none() );
  }

  // BUG-468 -- a tile GID beyond the supported `u8`-index range (e.g. a
  // flipped-tile GID, which sets high bits far beyond any realistic tile
  // count) must be rejected gracefully rather than panicking the parse.
  // test_kind: bug_reproducer(BUG-468)
  #[ test ]
  fn pattern_set_rejects_out_of_range_gid_without_panicking()
  {
    let tmx = r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" orientation="orthogonal" width="2" height="1">
 <layer id="1" name="Layer 1" width="2" height="1">
  <data encoding="csv">1,2147483649</data>
 </layer>
</map>"#;

    let mut state = app_state_empty();
    pattern_set( tmx, &mut state ); // must not panic
    assert!( state.pattern_image.is_none() );
  }

  // BUG-469 task/bug/completed/469_wfc_pattern_gid_zero_one_collision.md --
  // reproducer for GID 0 ( empty cell ) and GID 1 ( first tileset tile )
  // silently encoding to the same pixel value.
  // test_kind: bug_reproducer(BUG-469)
  #[ test ]
  fn pattern_set_distinguishes_empty_cell_from_first_tile()
  {
    let tmx = r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" orientation="orthogonal" width="2" height="1">
 <layer id="1" name="Layer 1" width="2" height="1">
  <data encoding="csv">0,1</data>
 </layer>
</map>"#;

    let mut state = app_state_empty();
    pattern_set( tmx, &mut state );

    let DynamicImage::ImageLuma8( image ) = state.pattern_image.expect( "a valid TMX must populate a pattern" )
    else
    {
      panic!( "pattern_set always builds an ImageLuma8" );
    };

    let empty_cell_pixel = image.get_pixel( 0, 0 ).0[ 0 ];
    let first_tile_pixel = image.get_pixel( 1, 0 ).0[ 0 ];

    assert_ne!
    (
      empty_cell_pixel, first_tile_pixel,
      "GID 0 (empty) and GID 1 (first tile) must not collide onto the same encoded pixel value"
    );
    assert_eq!( empty_cell_pixel, u8::MAX, "empty cells must encode to the reserved sentinel value" );
    assert_eq!( first_tile_pixel, 0, "the first tileset tile (GID 1) must encode to index 0" );
  }
}