//! Render tile map on quad.

mod wfc;

use gl::GL;
use minwebgl as gl;
use ndarray_cg::{ mat::DescriptorOrderColumnMajor, F32x4x4 };
use web_sys::wasm_bindgen::prelude::*;
use minwebgl::dom::create_image_element;
use wfc::{ generate, Relations };
use std::sync::Mutex;
use web_sys::console;

/// Tile map size. Length of square map side (a x a). 
/// More than 256x256 is very slow.
/// This example can generate only static size square maps
const SIZE : usize = 32;

/// Desciption what neighbours can have current tile.
/// You can imagine this relations array in such way:
/// 
/// `
///   [
///     0: [ 0, 1 ], // <tile_type>: [ <posible_neighbour_tile_types>... ]  
///     1: [ 1, 2 ],
///     2: [ 2, 3 ],
///     3: [ 3, 4 ],
///     4: [ 4, 5 ],
///     5: [ 5 ]
///   ]
/// `
/// 
/// Then relations array used by WFC algorithm for choosing neighbours of every map cell.
/// 
/// If tile type 0 has posible_neighbours 0, 1 then this is valid generation:
/// `
///   *, 1, *,
///   1, *0*, 0, // *0* has neighbours 1, 1, 0, 0 and this is valid case 
///   *, 0, *,   
/// `
/// 
/// And this is invalid generation that case WFC have to avoid: 
/// `
///   *, 2, *,
///   3, *0*, 1, // *0*  has neighbours 3, 2, which isn't valid, 
///   *, 0, *,   // because 0 has such available neighbours array: [ 0, 1 ]
/// `
const RELATIONS : &str = "
  [
    [ 0, 1 ],
    [ 1, 2 ],
    [ 2, 3 ],
    [ 3, 4 ],
    [ 4, 5 ],
    [ 5 ]
  ]
";

/// Storage for generated tile map
const MAP : Mutex< Option< Vec< Vec< u8 > > > > = Mutex::new( None );

/// Set load callback for image with [`path`] location and hide it from UI
fn load_image
(
  path : &str,
  on_load_callback : Box< dyn Fn( &web_sys::HtmlImageElement ) >,
) -> Result< web_sys::HtmlImageElement, minwebgl::JsValue >
{
  let image = create_image_element( "tileset.png" )?;

  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window.document()
  .expect( "Should have a document" );
  let body = document.body()
  .unwrap();
  let _ = body.append_child( &image );
  image.set_id( &format!( "{path}" ) );

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
  let origin = window.location()
  .origin()
  .expect( "Should have an origin" );
  let url = format!( "{origin}/static/{path}" );
  image.set_src( &url );
  Ok( image )
}

fn init()
{
  gl::browser::setup( Default::default() );

  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window.document()
  .expect( "Should have a document" );
  let body_style = document.body()
  .unwrap()
  .style();
  let _ = body_style.set_property( "margin", "0" );

  let load = move | _img : &web_sys::HtmlImageElement |
  {
    render_tile_map();
  };

  let _ = load_image( "tileset.png", Box::new( load ) );
}

fn prepare_vertex_attributes() -> WebGlVertexArrayObject
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

  vao
}

fn create_mvp() -> ndarray_cg::Mat< 4, 4, f32, DescriptorOrderColumnMajor >
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;
  let aspect_ratio = width / height;

  let perspective_matrix = ndarray_cg::d2::mat3x3h::perspective_rh_gl(
    70.0f32.to_radians(),
    aspect_ratio,
    0.1,
    1000.0
  );

  // qqq : use helpers
  // aaa : for now ndarray_cg crate don't have analog for creating 3x3 transformations, it has only 2x2 
  let t = ( 0.0, 0.0, 0.0 );
  let translate = F32x4x4::from_column_major
  (
    [
      1.0, 0.0, 0.0, t.0,
      0.0, 1.0, 0.0, t.1,
      0.0, 0.0, 1.0, t.2,
      0.0, 0.0, 0.0, 1.0,
    ]
  );

  let s = ( 2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0 );
  let scale = F32x4x4::from_column_major
  (
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
  let view_matrix = ndarray_cg::d2::mat3x3h::loot_at_rh( eye, center, up );

  perspective_matrix * view_matrix * translate * scale
}

/// Binds RGBA texture from image [`id`] to slot [`texture_id`]. 
/// Used for binding tile set to shader.
fn prepare_texture_array( id : &str, texture_id : u32 ) -> Option< web_sys::WebGlTexture >
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

/// Binds R8UI texture [`data`] with [`size`] to slot [`texture_id`]. 
/// Used for binding tile map to shader.
fn prepare_texture1u
(
  data: &[ u8 ],
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

/// Used for rendering tile map. Called only once when images are loaded.
/// This function prepare shaders, buffer data, bind textures, buffers to 
/// current GL context and then draws binded buffers
fn render_tile_map()
{
  let gl = gl::context::reretrieve_or_make()
  .unwrap();

  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src )
  .compile_and_link( &gl )
  .unwrap();
  gl.use_program( Some( &program ) );

  let mvp = create_mvp();
  let mvp_location = gl.get_uniform_location( &program, "mvp" );

  gl::uniform::matrix_upload( &gl, mvp_location, mvp.raw_slice(), false )
  .unwrap();

  let vao = prepare_vertex_attributes();
  gl.bind_vertex_array( Some( &vao ) );
  prepare_texture_array( "tileset.png", GL::TEXTURE0 );

  let Some( ref map ) = *MAP.lock().unwrap()
  else
  {
    panic!( "Map is empty" );
  };
  if map.is_empty() || map[ 0 ].is_empty()
  {
    panic!("Map is empty");
  }
  let size = ( map[ 0 ].len() as i32, map.len() as i32 );
  let data = map.iter()
  .cloned()
  .flatten()
  .collect::< Vec< u8 > >();

  prepare_texture1u( &data, size, GL::TEXTURE1 );

  let tiles_location = gl.get_uniform_location( &program, "tiles_sampler" );
  let map_location = gl.get_uniform_location( &program, "map_sampler" );

  // When more than 1 texture is used. You need set binding slot for every texture.
  gl.uniform1i( tiles_location.as_ref(), 0 );
  gl.uniform1i( map_location.as_ref(), 1 );

  let texel_size = [ 1.0 / size.0 as f32, 1.0 / size.1 as f32 ];
  let texel_size_location = gl.get_uniform_location( &program, "texel_size" );
  let _ = gl::uniform::upload( &gl, texel_size_location, texel_size.as_slice() );

  gl.draw_arrays( GL::TRIANGLES, 0, 3 * 2 );
  gl.bind_vertex_array( None );
}

fn generate_map() -> Result< (), String >
{
  let relations: Relations = match serde_json::from_str( RELATIONS )
  {
    Ok( relations ) => relations,
    Err( err ) => return Err( format!( "{err}" ) )
  };
  let map = generate( ( SIZE, SIZE ), relations, 0.01 )?;
  *MAP.lock().unwrap() = Some( map );
  Ok( () )
}

fn run()
{
  if let Err( err ) = generate_map()
  {
    console::log_1( &JsValue::from( format!( "{err}" ) ) );
  };
  init();
  render_tile_map();
}

fn main()
{
  run()
}