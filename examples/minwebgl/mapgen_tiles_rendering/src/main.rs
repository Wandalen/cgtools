//! Render tile map on quad.

use gl::GL;
use minwebgl as gl;
use ndarray_cg::{ mat::DescriptorOrderColumnMajor, F32x4x4 };
use web_sys::wasm_bindgen::prelude::*;

const LAYERS : i32 = 6;
// Tile map raw data for texture with integer color channels
const DATA : [ u8; 256 ] =
[
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 2, 1, 0,
  0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 1, 2, 2, 1,
  0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 1, 0, 1, 1, 0,
  0, 0, 1, 1, 1, 1, 2, 2, 3, 3, 2, 2, 1, 0, 0, 0,
  0, 1, 2, 2, 1, 1, 2, 3, 4, 4, 3, 3, 2, 1, 0, 0,
  1, 2, 3, 3, 2, 2, 2, 3, 4, 4, 4, 3, 2, 1, 0, 0,
  1, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 3, 2, 1, 0, 0,
  1, 2, 3, 4, 4, 4, 4, 4, 5, 5, 4, 3, 3, 2, 1, 0,
  1, 2, 3, 4, 4, 4, 4, 5, 5, 5, 4, 4, 3, 2, 1, 0,
  1, 2, 3, 3, 4, 4, 1, 1, 5, 5, 4, 4, 3, 2, 1, 0,
  0, 1, 2, 3, 3, 1, 1, 4, 4, 4, 4, 3, 2, 1, 1, 0,
  0, 0, 1, 2, 1, 1, 3, 3, 3, 3, 3, 3, 2, 1, 0, 0,
  0, 0, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0,
  0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn load_callback_set()
{
  let load = move | _img: &web_sys::HtmlImageElement |
  {
    update();
  };

  let _ = image_load( "static/tileset.png", Box::new( load ) );
}

fn image_load
(
  path : &str,
  on_load_callback : Box< dyn Fn( &web_sys::HtmlImageElement ) >,
) -> Result< web_sys::HtmlImageElement, minwebgl::JsValue >
{
  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window
  .document()
  .expect( "Should have a document" );
  // Created directly rather than via `image_element_create` — this function
  // sets its own `src` below ( it needs `id`/style/`cross_origin` attached
  // before the load fires ), so a first `image_element_create` call here
  // would only have its `src` immediately overwritten.
  let image = document
  .create_element( "img" )?
  .dyn_into::< web_sys::HtmlImageElement >()?;
  let body = document
  .body()
  .unwrap();
  let _ = body.append_child( &image );
  image.set_id( path );
  let _ = image.style()
  .set_property( "visibility", "hidden" );
  let _ = image.style()
  .set_property( "position", "absolute" );
  let _ = image.style()
  .set_property( "top", "0" );
  let _ = image.style()
  .set_property( "width", "10px" );
  let _ = image.style()
  .set_property( "height", "10px" );
  image.set_cross_origin( Some( "anonymous" ) );
  let img = image.clone();
  let on_load_callback: Closure< dyn Fn() > = Closure::new( move || on_load_callback( &img ) );
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

fn init()
{
  gl::browser::setup( gl::browser::Config::default() );

  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window.document()
  .expect( "Should have a document" );
  let body_style = document
  .body()
  .unwrap()
  .style();
  let _ = body_style.set_property( "margin", "0" );

  load_callback_set();
}

fn vertex_attributes_prepare()
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
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
    .stride( 2 )
    .offset( 0 )
    .attribute_pointer( &gl, position_slot, &position_buffer )
    .unwrap();
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
    .stride( 2 )
    .offset( 0 )
    .attribute_pointer( &gl, uv_slot, &uv_buffer )
    .unwrap();
  gl.bind_vertex_array( None );
  gl.bind_vertex_array( Some( &vao ) );
}

fn mvp_create() -> ndarray_cg::Mat< 4, 4, f32, DescriptorOrderColumnMajor >
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;
  let aspect_ratio = width / height;

  let perspective_matrix = ndarray_cg::d2::mat3x3h::perspective_rh_gl
  (
    70.0f32.to_radians(),
    aspect_ratio,
    0.1,
    1000.0
  );

  let t = ( 0.0, 0.0, 0.0 );
  let translate = F32x4x4::from_column_major(
    [
      1.0, 0.0, 0.0, t.0,
      0.0, 1.0, 0.0, t.1,
      0.0, 0.0, 1.0, t.2,
      0.0, 0.0, 0.0, 1.0,
    ]
  );

  let s = ( 2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0 );
  let scale = F32x4x4::from_column_major(
    [
      s.0, 0.0, 0.0, 0.0,
      0.0, s.1, 0.0, 0.0,
      0.0, 0.0, s.2, 0.0,
      0.0, 0.0, 0.0, 1.0,
    ]
  );

  let eye = [ 0.0, 0.0, 1.0 ];
  let up = [ 0.0, 1.0, 0.0 ];
  let center = [ 0., 0., 0. ];
  let view_matrix = ndarray_cg::d2::mat3x3h::look_at_rh( eye, center, up );

  perspective_matrix * view_matrix * translate * scale
}

fn texture_array_prepare( id: &str, layers: i32, texture_id: u32 ) -> Option< web_sys::WebGlTexture >
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
  // Texture array is image with height: 1 tile height * tile count
  let height = img.natural_height() / layers as u32;

  let texture_array = gl.create_texture();
  // Don't forget to activate the texture before binding and
  // setting texture data and parameters
  gl.active_texture( texture_id );
  gl.bind_texture( GL::TEXTURE_2D_ARRAY, texture_array.as_ref() );

  let _ = gl.tex_image_3d_with_html_image_element
  (
    GL::TEXTURE_2D_ARRAY,
    0,
    GL::RGBA as i32,
    width as i32,
    height as i32,
    layers,
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

fn texture1u_prepare
(
  data: &[u8],
  size: ( i32, i32 ),
  texture_id: u32,
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
    // Texture from raw data must have format with integer channels
    // Data range here is 0..255
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

fn update()
{
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

  vertex_attributes_prepare();
  // Pre-existing defect found during BUG-109 live-reverification, distinct root
  // cause: `image_load` ( see its call site in `load_callback_set` ) sets the
  // created `<img>`'s DOM `id` to the full path it was given, `"static/tileset.png"`
  // — but this lookup was passing the bare filename, `"tileset.png"`, which never
  // matches. `get_element_by_id` returned `None`, and the leading `?` in
  // `texture_array_prepare` silently skipped texture creation entirely before any
  // GL call ran, leaving the tile map's texture unit unbound ( black canvas ).
  texture_array_prepare( "static/tileset.png", LAYERS, GL::TEXTURE0 );

  let size = ( 16, 16 );
  texture1u_prepare( &DATA, size, GL::TEXTURE1 );

  let tiles_location = gl.get_uniform_location( &program, "tiles_sampler" );
  let map_sampler_location = gl.get_uniform_location( &program, "map_sampler" );

  // When more than 1 texture is used. You need set binding slot for every texture.
  gl.uniform1i( tiles_location.as_ref(), 0 );
  gl.uniform1i( map_sampler_location.as_ref(), 1 );

  let texel_size = [ 1.0 / size.0 as f32, 1.0 / size.1 as f32 ];
  let texel_size_location = gl.get_uniform_location( &program, "texel_size" );
  let _ = gl::uniform::upload( &gl, texel_size_location, texel_size.as_slice() );

  gl.draw_arrays( GL::TRIANGLES, 0, 3 * 2 );
  gl.bind_vertex_array( None );
}

fn app_run()
{
  init();
  update();
}

fn main()
{
  app_run();
}
