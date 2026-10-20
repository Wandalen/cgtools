//! Fixtures shared by the glTF loader's browser suites ( `gltf_meshopt_browser_test.rs`,
//! `gltf_buffers_browser_test.rs`, `gltf_quantization_browser_test.rs` ) : a live context, data
//! uris, a one-triangle document, GLB packing and read-back of what `load` uploaded.

#![ cfg( target_arch = "wasm32" ) ]
#![ allow( dead_code, reason = "each suite that includes this module uses only some of its fixtures" ) ]

use minwebgl as gl;
use gl::GL;
use renderer::webgl::loaders::gltf;

pub fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap()
}

pub fn document() -> gl::web_sys::Document
{
  gl::web_sys::window().unwrap().document().unwrap()
}

/// One triangle's `POSITION` data : three `f32` VEC3 elements, 12 bytes each.
pub fn triangle_bytes() -> Vec< u8 >
{
  [ 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0_f32 ].iter().flat_map( | f | f.to_le_bytes() ).collect()
}

/// The reason `gltf::load` refuses `uri` : the static message of its `WebglError::Other`,
/// so a test can tell its intended rejection from any other failure.
pub async fn load_refusal( uri : &str ) -> &'static str
{
  match gltf::load( &document(), uri, &gl_init() ).await
  {
    Ok( _ ) => panic!( "the file must be refused" ),
    Err( gl::WebglError::Other( reason ) ) => reason,
    Err( other ) => panic!( "refused for an unexpected reason: {other:?}" ),
  }
}

pub fn data_uri( mime : &str, bytes : &[ u8 ] ) -> String
{
  let latin1 : String = bytes.iter().map( | &b | char::from( b ) ).collect();
  format!( "data:{mime};base64,{}", gl::web_sys::window().unwrap().btoa( &latin1 ).unwrap() )
}

/// A one-triangle scene whose `POSITION` accessor reads bufferView 0. `buffers` and `view` are
/// spliced in as JSON; `used` lists extensions for `extensionsUsed` / `extensionsRequired`.
pub fn triangle_gltf( used : &[ &str ], buffers : &str, view : &str ) -> String
{
  let extensions = if used.is_empty()
  {
    String::new()
  }
  else
  {
    let names = used.iter().map( | name | format!( "\"{name}\"" ) ).collect::< Vec< _ > >().join( ", " );
    format!( r#""extensionsUsed" : [ {names} ], "extensionsRequired" : [ {names} ],"# )
  };
  format!
  (
    r#"{{
      "asset" : {{ "version" : "2.0" }},
      {extensions}
      "buffers" : [ {buffers} ],
      "bufferViews" : [ {view} ],
      "accessors" : [ {{ "bufferView" : 0, "componentType" : 5126, "count" : 3, "type" : "VEC3", "min" : [ 0, 0, 0 ], "max" : [ 1, 1, 0 ] }} ],
      "materials" : [ {{}} ],
      "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0 }}, "material" : 0 }} ] }} ],
      "nodes" : [ {{ "mesh" : 0 }} ],
      "scenes" : [ {{ "nodes" : [ 0 ] }} ],
      "scene" : 0
    }}"#
  )
}

/// Packs `json` and `bin` into a GLB container : a 12-byte header, then the JSON chunk padded
/// with spaces and the BIN chunk padded with zeros, each behind its length and type.
pub fn glb( json : &str, bin : &[ u8 ] ) -> Vec< u8 >
{
  let as_u32 = | n : usize | u32::try_from( n ).unwrap().to_le_bytes();
  let mut json = json.as_bytes().to_vec();
  json.resize( json.len().next_multiple_of( 4 ), b' ' );
  let mut bin = bin.to_vec();
  bin.resize( bin.len().next_multiple_of( 4 ), 0 );

  let mut out = Vec::new();
  out.extend_from_slice( b"glTF" );
  out.extend_from_slice( &2_u32.to_le_bytes() );
  out.extend_from_slice( &as_u32( 12 + 8 + json.len() + 8 + bin.len() ) );
  out.extend_from_slice( &as_u32( json.len() ) );
  out.extend_from_slice( b"JSON" );
  out.extend( json );
  out.extend_from_slice( &as_u32( bin.len() ) );
  out.extend_from_slice( b"BIN\0" );
  out.extend( bin );
  out
}

/// Reads back the GPU buffer `load` uploaded for bufferView `view`.
pub fn view_bytes_read( gl : &GL, loaded : &gltf::GLTF, view : usize, length : usize ) -> Vec< u8 >
{
  let mut bytes = vec![ 0u8; length ];
  gl.bind_buffer( gl::COPY_READ_BUFFER, Some( &loaded.gl_buffers[ view ] ) );
  gl.get_buffer_sub_data_with_i32_and_u8_array( gl::COPY_READ_BUFFER, 0, &mut bytes );
  gl.bind_buffer( gl::COPY_READ_BUFFER, None );
  bytes
}

