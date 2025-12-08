//! Renders outlines for 3D objects.
#![ doc( html_root_url = "https://docs.rs/narrow_outline/latest/narrow_outline/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders outlines for 3D objects" ) ]

#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::doc_overindented_list_items ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::map_flatten ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::needless_for_each ) ]
#![ allow( clippy::let_and_return ) ]
#![ allow( clippy::useless_conversion ) ]
#![ allow( clippy::manual_memcpy ) ]
#![ allow( clippy::needless_range_loop ) ]
#![ allow( clippy::match_wildcard_for_single_variants ) ]
#![ allow( clippy::single_match ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::missing_panics_doc ) ]
#![ allow( clippy::module_name_repetitions ) ]

use mingl::
{
  AsBytes,
  VectorDataType
};
use minwebgl as gl;
use gl::
{
  WebglError,
  drawbuffers::drawbuffers,
  GL,
  web_sys::
  {
    WebGl2RenderingContext,
    WebGlUniformLocation,
    WebGlTexture,
    WebGlFramebuffer,
    WebGlProgram,
    WebGlBuffer
  }
};
use std::rc::Rc;
use std::cell::RefCell;
use renderer::webgl::
{
  camera::Camera,
  loaders::gltf::{ load, GLTF },
  node::{ Node, Object3D },
  program::
  {
    NormalDepthOutlineObjectShader,
    NormalDepthOutlineShader,
    ProgramInfo
  },
  scene::Scene,
  AttributeInfo,
  IndexInfo,
  Geometry,
  Material,
  Mesh,
  Primitive
};
use ndarray_cg::
{
  mat3x3h::rot,
  F32x4,
  F32x3
};
use std::collections::HashMap;
use csgrs::CSG;
use rand::Rng;
use std::any::type_name_of_val;

const MAX_OBJECT_COUNT : usize = 1024;

/// Binds a texture to a texture unit and uploads its location to a uniform.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `texture` - The texture to bind.
/// * `location` - The uniform location in the shader for the sampler.
/// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
fn upload_texture
(
  gl : &gl::WebGl2RenderingContext,
  texture : &WebGlTexture,
  location : &WebGlUniformLocation,
  slot : u32,
)
{
  gl.active_texture( slot );
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
  gl.uniform1i( Some( location ), ( slot - GL::TEXTURE0 ) as i32 );
}

/// Creates a WebGL2 framebuffer and a color attachment texture.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `size` - The size of the framebuffer and its attachment ( width, height ).
/// * `color_attachment` - The index of the color attachment point ( e.g., 0 for `GL::COLOR_ATTACHMENT0` ).
///
/// # Returns
///
/// An `Option< ( WebGlFramebuffer, WebGlTexture ) >` containing the created framebuffer and
/// its color attachment texture, or `None` if creation fails.
fn create_framebuffer
(
  gl : &gl::WebGl2RenderingContext,
  ( width, height ) : ( i32, i32 )
)
-> Option< ( WebGlFramebuffer, Vec< WebGlTexture > ) >
{
  let color = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
  // Use tex_storage_2d for immutable texture storage ( WebGL2 )
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  let depthbuffer = gl.create_renderbuffer().unwrap();
  gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width, height );

  let depth = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &depth ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, GL::R32F, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  // Normal attachment
  // Using RGB8 for normals (each component x,y,z stored in R,G,B)
  let normal = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &normal ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGB8, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  let framebuffer = gl.create_framebuffer()?;
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &color ), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, Some( &normal ), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT2, GL::TEXTURE_2D, Some( &depth ), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

  drawbuffers( gl, &[ 0, 1, 2 ] );

  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  Some( ( framebuffer, vec![ color, depth, normal ] ) )
}

/// Binds a framebuffer for rendering and sets the viewport to its size.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `framebuffer` - The framebuffer to bind.
/// * `size` - The size of the framebuffer ( width, height ).
fn upload_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  framebuffer : &WebGlFramebuffer,
  size : ( i32, i32 )
)
{
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
  gl.viewport( 0, 0, size.0, size.1 );
}

/// Uploads raw byte data to a WebGL buffer.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `buffer` - The `WebGlBuffer` to upload data to.
/// * `target` - The target buffer type ( e.g., `GL::ARRAY_BUFFER` ).
/// * `offset` - The offset in bytes within the buffer to start uploading data.
/// * `data` - The `Vec<u8>` containing the data to upload.
pub fn upload_buffer_data
(
  gl : &gl::WebGl2RenderingContext,
  buffer : &WebGlBuffer,
  target : u32,
  offset : u32,
  data : Vec< u8 >
)
{
  let data = data.into_iter()
  .collect::< Vec< _ > >();

  gl.bind_buffer( target, Some( buffer ) );
  gl.buffer_data_with_js_u8_array_and_src_offset_and_length
  (
    target,
    &gl::js_sys::Uint8Array::from( data.as_bytes() ),
    gl::STATIC_DRAW,
    offset,
    data.len() as u32
  );
}

/// Simplifies new buffer initialization
pub fn add_buffer
(
  gl : &gl::WebGl2RenderingContext,
  gltf : &mut GLTF,
  buffer_data : Vec< u8 >
) -> Result< WebGlBuffer, gl::WebglError >
{
  let buffer = gl.create_buffer().ok_or( gl::WebglError::FailedToAllocateResource( "Buffer" ) )?;
  upload_buffer_data( gl, &buffer, GL::ARRAY_BUFFER, 0, buffer_data );
  gltf.gl_buffers.push( buffer.clone() );
  Ok( buffer )
}

/// Adds additional attributes and their data into [`GLTF`] and
/// returns object_id data for updating data for per object attributes
pub fn add_attributes
(
  gl : &gl::WebGl2RenderingContext,
  gltf : &mut GLTF,
) -> Result< Vec< i32 >, gl::WebglError >
{
  let mut object_id_data : Vec< i32 > = vec![];

  let mut object_id = 1;
  let mut object_vertex_count = 0;
  for mesh in &gltf.meshes
  {
    for primitive in &mesh.borrow().primitives
    {
      let primitive = primitive.borrow();
      let geometry = primitive.geometry.borrow();
      let vertex_count = geometry.vertex_count as usize;
      object_vertex_count += vertex_count;
    }

    object_id_data.extend( vec![ object_id; object_vertex_count ] );

    object_id += 1;
  }

  let object_id_bytes = object_id_data.iter().map( | i | i.to_be_bytes() ).flatten().collect::< Vec< _ > >();
  let object_id_buffer = add_buffer( gl, gltf, object_id_bytes )?;

  let object_id_info = make_buffer_attribute_info(
    &object_id_buffer,
    0,
    1,
    2,
    false,
    VectorDataType::new( mingl::DataType::F32, 1, 1 )
  )
  .unwrap();

  for mesh in &gltf.meshes
  {
    for primitive in &mesh.borrow().primitives
    {
      let primitive = primitive.borrow();
      let mut geometry = primitive.geometry.borrow_mut();
      let _ = geometry.add_attribute( gl, "object_ids", object_id_info.clone(), false );
    }
  }

  Ok( object_id_data )
}

/// Creates an `AttributeInfo` struct for a given WebGL buffer.
///
/// # Arguments
///
/// * `buffer` - The WebGL buffer containing the attribute data.
/// * `offset` - The offset in bytes to the first component of the first generic vertex attribute.
/// * `stride` - The byte offset between consecutive generic vertex attributes.
/// * `slot` - The attribute location ( slot ) in the shader program.
/// * `normalized` - Whether integer data values should be normalized.
/// * `vector` - The `VectorDataType` describing the data type and component count.
///
/// # Returns
///
/// A `Result<AttributeInfo, WebglError>` containing the attribute info or an error if the type is not supported.
fn make_buffer_attribute_info
(
  buffer : &web_sys::WebGlBuffer,
  offset : i32,
  stride : i32,
  slot : u32,
  normalized : bool,
  vector: gl::VectorDataType
) -> Result< AttributeInfo, gl::WebglError >
{
  let descriptor = match vector.scalar
  {
      gl::DataType::U8 => gl::BufferDescriptor::new::< [ u8; 1 ] >(),
      gl::DataType::I8 => gl::BufferDescriptor::new::< [ i8; 1 ] >(),
      gl::DataType::U16 => gl::BufferDescriptor::new::< [ u16; 1 ] >(),
      gl::DataType::I16 => gl::BufferDescriptor::new::< [ i16; 1 ] >(),
      gl::DataType::U32 => gl::BufferDescriptor::new::< [ u32; 1 ] >(),
      gl::DataType::F32 => gl::BufferDescriptor::new::< [ f32; 1 ] >(),
      _ => return Err( gl::WebglError::NotSupportedForType( type_name_of_val( &vector.scalar ) ) )
  };

  let descriptor = descriptor
  .offset( offset )
  .normalized( normalized )
  .stride( stride )
  .vector( vector );

  Ok
  (
    AttributeInfo
    {
      slot,
      buffer : buffer.clone(),
      descriptor,
      bounding_box : Default::default()
    }
  )
}

/// Adds a single CSG primitive's geometry data to the provided vectors.
///
/// # Arguments
///
/// * `primitive` - The CSG primitive to process.
/// * `positions` - A mutable vector to accumulate vertex position data.
/// * `normals` - A mutable vector to accumulate vertex normal data.
/// * `object_ids` - A mutable vector to accumulate object ID data for each vertex.
/// * `indices` - A mutable vector to accumulate index data.
/// * `vertex_offset` - A mutable reference to the current vertex offset, which is updated.
pub fn add_primitive
(
  primitive : CSG< () >,
  positions: &mut Vec< [ f32; 3 ] >,
  normals: &mut Vec< [ f32; 3 ] >,
  object_ids: &mut Vec< f32 >,
  indices: &mut Vec< u32 >,
  vertex_offset: &mut u32,
)
{
  let mut last_object_id = *object_ids.last().unwrap_or( &0.0 );

  let mesh = primitive.to_trimesh().unwrap();

  let primitive_positions = mesh.vertices()
  .iter()
  .map( | p | [ p.coords.x as f32, p.coords.y as f32, p.coords.z as f32 ] )
  .collect::< Vec< _ > >();
  positions.extend( primitive_positions.clone() );

  let primitive_indices = mesh.indices()
  .iter()
  .flatten()
  .map( | i | i + *vertex_offset )
  .collect::< Vec< _ > >();
  indices.extend( primitive_indices.clone() );

  let vertices_count = mesh.vertices().len();

  // Calculating normals for primitives using this article: https://iquilezles.org/articles/normals/
  let mut primitive_normals = vec![ [ 0.0; 3 ]; vertices_count ];
  primitive_indices.chunks( 3 )
  .for_each
  (
    | ids |
    {
      let t = ( 0..3 ).map( | i | F32x3::from( positions[ ids[ i ] as usize ] ) )
      .collect::< Vec< _ > >();
      let e1 = t[ 0 ] - t[ 1 ];
      let e2 = t[ 2 ] - t[ 1 ];
      let c = ndarray_cg::vector::cross( &e1, &e2 );
      ( 0..3 ).for_each
      (
        | i |
        {
          primitive_normals[ ( ids[ i ] - *vertex_offset ) as usize ] = [ c[ 0 ], c[ 1 ], c[ 2 ] ];
        }
      );
    }
  );

  primitive_normals.iter_mut()
  .for_each(
    | n |
    {
      *n = *F32x3::from_array( *n ).normalize();
    }
  );

  normals.extend( primitive_normals );

  last_object_id += 1.0;
  object_ids.extend( vec![ last_object_id as f32; vertices_count ] );

  *vertex_offset += vertices_count as u32;
}

/// Generates a vector of CSG primitives and random transformations for each.
///
/// # Returns
///
/// A `Vec<(CSG<()>, [f32; 9])>` where each tuple contains a primitive and an array of
/// 9 floats representing its translation, rotation, and scale.
fn get_primitives_and_transform() -> Vec< ( CSG< () >, [ f32; 9 ] ) >
{
  let meshes: Vec< CSG< () > > = vec![
    {
      // Cone is constructed using frustum with one radius near zero.
      // Parameters: radius1, radius2, height, segments
      CSG::frustum( 1.0, 0.001, 2.0, 32, None )
    },
    {
      // Torus is constructed by revolving a 2D circle.
      // A circle with minor_radius is translated by major_radius along X, then revolved.
      let minor_radius = 0.5;
      let major_radius = 1.5;
      let segments = 32; // Segments for the circle cross-section
      let revolve_segments = 64; // Segments for the revolution

      let circle_2d = CSG::circle( minor_radius, segments, None );
      // Translate the circle away from the origin to define the major radius.
      // The `rotate_extrude` revolves around the Y-axis.
      circle_2d
      .translate_vector( [ major_radius, 0.0, 0.0 ].into() )
      .rotate_extrude( 360.0, revolve_segments )
    },
    {
      // Direct cylinder primitive.
      // Parameters: radius, height, segments
      CSG::cylinder( 1.0, 2.0, 32, None )
    },
    {
      // Direct sphere primitive.
      // Parameters: radius, segments, stacks
      CSG::sphere( 1.0, 32, 16, None )
    },
    {
      // Direct cube/cuboid primitive.
      // Parameters: width, length, height
      CSG::cube( 1.0, None )
    },
    {
      // Capsule3d is constructed by unioning a cylinder with two hemispheres (spheres).
      let radius = 0.5;
      let height = 1.0;
      let segments = 32;
      let stacks = 16;

      let cylinder = CSG::cylinder( radius, height, segments, None );
      let top_sphere = CSG::sphere( radius, segments, stacks, None )
      .translate_vector( [ 0.0, 0.0, height ].into() );
      let bottom_sphere = CSG::sphere( radius, segments, stacks, None );

      cylinder.union( &top_sphere )
      .union( &bottom_sphere )
    }
  ];

  let mut position = F32x4::new( 2.0, 0.0, 1.0, 1.0 );

  // Generate random transformation parameters for each mesh.
  let mut rng = rand::rng();
  let count = meshes.len();
  let rot_matrix = rot( 0.0f32, ( 360.0f32 / count as f32 ).to_radians(), 0.0f32 );
  let primitives = ( 0..count )
  .into_iter()
  .map(
    | i |
    {
      // t[ 0 - 2 ]: translation ( x, y, z )
      // t[ 3 - 5 ]: rotation ( Euler XYZ, in degrees )
      // t[ 6 - 8 ]: scale ( x, y, z )
      let mut t = [ 0.0; 9 ];

      position = rot_matrix * position;

      for j in 0..3
      {
        t[ j ] = position.0[ j ];
      }

      for j in 3..6
      {
        t[ j ] = ( rng.random_range( 0.0..360.0 ) as f32 ).to_radians();
      }

      for j in 6..9
      {
        t[ j ] = rng.random_range( 0.35..0.6 );
      }

      ( meshes[ i ].clone(), t )
    }
  )
  .collect::< Vec<( CSG< () >, [ f32; 9 ] ) > >();

  primitives
}

/// Converts a collection of CSG primitives into a GLTF object with WebGL resources.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
///
/// # Returns
///
/// A `GLTF` struct containing scenes, nodes, buffers, and other resources derived from the CSG primitives.
fn primitives_csgrs_gltf
(
  gl : &gl::WebGl2RenderingContext,
) -> GLTF
{
  let mut gltf = GLTF
  {
    scenes : vec![],
    nodes : vec![],
    gl_buffers : vec![],
    images : Rc::new( RefCell::new( vec![] ) ),
    textures : vec![],
    materials : vec![],
    meshes : vec![],
    animations : vec![],
  };

  gltf.scenes.push( Rc::new( RefCell::new( Scene::new() ) ) );

  let mut positions : Vec< [ f32; 3 ] > = vec![];
  let mut normals : Vec< [ f32; 3 ] > = vec![];
  let mut object_ids : Vec< f32 > = vec![];
  let mut indices : Vec< u32 > = vec![];
  let mut vertex_offset : u32 = 0;

  let position_buffer = gl.create_buffer().unwrap();
  let normal_buffer = gl.create_buffer().unwrap();
  let object_id_buffer = gl.create_buffer().unwrap();

  for buffer in
  [
    position_buffer.clone(),
    normal_buffer.clone(),
    object_id_buffer.clone()
  ]
  {
    gltf.gl_buffers.push( buffer );
  }

  let primitives = get_primitives_and_transform();

  let material = Rc::new( RefCell::new( Material::default() ) );
  gltf.materials.push( material.clone() );

  let attribute_infos =
  [
    (
      "positions",
      make_buffer_attribute_info
      (
        &position_buffer,
        0,
        3,
        0,
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap()
    ),
    (
      "normals",
      make_buffer_attribute_info
      (
        &normal_buffer,
        0,
        3,
        1,
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap()
    ),
    (
      "object_ids",
      make_buffer_attribute_info
      (
        &object_id_buffer,
        0,
        1,
        2,
        false,
        VectorDataType::new( mingl::DataType::F32, 1, 1 )
      ).unwrap()
    ),
  ];

  let index_buffer = gl.create_buffer().unwrap();
  gltf.gl_buffers.push( index_buffer.clone() );

  let mut index_info = IndexInfo
  {
    buffer : index_buffer.clone(),
    count : 0,
    offset : 0,
    data_type : GL::UNSIGNED_INT
  };

  for ( primitive, t ) in primitives
  {
    let last_indices_count = indices.len() as u32;
    let last_vertex_offset = vertex_offset;

    add_primitive
    (
      primitive,
      &mut positions,
      &mut normals,
      &mut object_ids,
      &mut indices,
      &mut vertex_offset
    );

    index_info.offset = last_indices_count * 4;
    index_info.count = indices.len() as u32 - last_indices_count;

    let Ok( mut geometry ) = Geometry::new( gl ) else
    {
      continue;
    };

    for ( name, info ) in &attribute_infos
    {
      geometry.add_attribute( gl, *name, info.clone(), false ).unwrap();
    }

    geometry.add_index( gl, index_info.clone() ).unwrap();
    geometry.vertex_count = vertex_offset - last_vertex_offset;

    let primitive = Primitive
    {
      geometry : Rc::new( RefCell::new( geometry ) ),
      material : material.clone()
    };

    let mesh = Rc::new( RefCell::new( Mesh::new() ) );
    mesh.borrow_mut().add_primitive( Rc::new( RefCell::new( primitive ) ) );

    let node = Rc::new( RefCell::new( Node::new() ) );
    {
      let mut node_mut = node.borrow_mut();
      node_mut.object = Object3D::Mesh( mesh );

      node_mut.set_translation( [ t[ 0 ], t[ 1 ], t[ 2 ] ] );
      let q = gl::QuatF32::from_euler_xyz( [ t[ 3 ], t[ 4 ], t[ 5 ] ] );
      node_mut.set_rotation( q );
      node_mut.set_scale( [ t[ 6 ], t[ 7 ], t[ 8 ] ] );
      node_mut.update_local_matrix();
    }
    gltf.nodes.push( node.clone() );
    gltf.scenes[ 0 ].borrow_mut().children.push( node );
  }

  gl::buffer::upload( &gl, &position_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &normal_buffer, &normals, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &object_id_buffer, &object_ids, GL::STATIC_DRAW );
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

  gltf
}

/// A collection of shader programs used for rendering.
struct Programs
{
  /// The shader program for the initial object rendering pass.
  object : ProgramInfo< NormalDepthOutlineObjectShader >,
  /// The shader program for the final outline pass.
  outline : ProgramInfo< NormalDepthOutlineShader >,
  /// The raw WebGL program for the outline shader.
  outline_program : WebGlProgram
}

impl Programs
{
  /// Creates a new `Programs` instance, compiling and linking the necessary shaders.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  ///
  /// # Returns
  ///
  /// A `Programs` struct with the compiled and linked shader programs.
  fn new( gl : &gl::WebGl2RenderingContext ) -> Self
  {
    // --- Load and Compile Shaders ---

    let object_vs_src = include_str!( "../resources/shaders/object.vert" );
    let object_fs_src = include_str!( "../resources/shaders/object.frag" );
    let fullscreen_vs_src = include_str!( "../resources/shaders/fullscreen.vert" );
    let outline_fs_src = include_str!( "../resources/shaders/outline.frag" );

    // Compile and link shader programs and store them
    let object_program = gl::ProgramFromSources::new( object_vs_src, object_fs_src ).compile_and_link( gl ).unwrap();
    let outline_program = gl::ProgramFromSources::new( fullscreen_vs_src, outline_fs_src ).compile_and_link( gl ).unwrap();

    let object = ProgramInfo::< NormalDepthOutlineObjectShader >::new( gl, object_program );
    let outline = ProgramInfo::< NormalDepthOutlineShader >::new( gl, outline_program.clone() );

    Self
    {
      object,
      outline,
      outline_program
    }
  }
}

/// Manages WebGL resources and rendering passes.
struct Renderer
{
  /// The WebGL2 rendering context.
  gl : WebGl2RenderingContext,
  /// The compiled and linked shader programs.
  programs : Programs,
  /// A hash map to store WebGL buffers by name.
  buffers : HashMap< String, WebGlBuffer >,
  /// A hash map to store WebGL textures by name.
  textures : HashMap< String, WebGlTexture >,
  /// A hash map to store WebGL framebuffers by name.
  framebuffers : HashMap< String, WebGlFramebuffer >,
  /// The current viewport size ( width, height ).
  viewport : ( i32, i32 ),
  /// The main camera for the scene.
  camera : Camera,
  /// A vector of random colors for each object, used in the outline pass.
  object_colors : Vec< f32 >
}

impl Renderer
{
  /// Creates a new Renderer instance, initializes WebGL, loads resources,
  /// and prepares the scene for rendering.
  async fn new() -> Self
  {
    gl::browser::setup( Default::default() );
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas( &canvas ).unwrap();

    // --- Initialization ---

    let viewport = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );

    let eye = F32x3::from_array( [ 0.0, 1.4, 2.5 ] ) * 1.5;
    let up = F32x3::Y;

    let aspect_ratio = viewport.0 as f32 / viewport.1 as f32;
    let fov =  70.0f32.to_radians();
    let near = 0.1;
    let far = 1000.0;

    let camera = Camera::new(
      eye,
      up,
      F32x3::new( 0.0, 0.4, 0.0 ),
      aspect_ratio,
      fov,
      near,
      far
    );

    camera.bind_controls( &canvas );

    let programs = Programs::new( &gl );

    // Create and store renderer instance
    let mut renderer = Self
    {
      gl,
      programs,
      buffers : HashMap::new(),
      textures : HashMap::new(),
      framebuffers : HashMap::new(),
      viewport,
      camera,
      object_colors : vec![]
    };

    let gl = &renderer.gl;

    // --- Create Framebuffers and Textures ---

    // Framebuffer for rendering the initial object silhouette
    let ( object_fb, t ) = create_framebuffer( gl, viewport ).unwrap();
    let object_fb_color = t[ 0 ].clone();
    let object_fb_depth = t[ 1 ].clone();
    let object_fb_norm = t[ 2 ].clone();

    // Store the color attachment textures
    renderer.textures.insert( "object_fb_color".to_string(), object_fb_color );
    renderer.textures.insert( "object_fb_depth".to_string(), object_fb_depth );
    renderer.textures.insert( "object_fb_norm".to_string(), object_fb_norm );

    // Store the framebuffers
    renderer.framebuffers.insert( "object_fb".to_string(), object_fb );

    let mut object_colors = vec![ [ 0.0; 4 ]; MAX_OBJECT_COUNT ];
    let mut rng = rand::rng();

    let range = 0.2..1.0;
    ( 0..MAX_OBJECT_COUNT ).for_each
    (
      | i |
      {
        object_colors[ i ] = F32x4::from(
          [
            rng.random_range( range.clone() ),
            rng.random_range( range.clone() ),
            rng.random_range( range.clone() ),
            1.0
          ]
        )
        .0;
      }
    );

    let object_color_buffer = gl::buffer::create( &gl ).unwrap();
    renderer.buffers.insert( "object_color_buffer".to_string(), object_color_buffer.clone() );
    let u_object_colors_loc = gl.get_uniform_block_index( &renderer.programs.outline_program, "ObjectColorBlock" );
    gl.uniform_block_binding( &renderer.programs.outline_program, u_object_colors_loc, 0 );
    gl.bind_buffer_base( GL::UNIFORM_BUFFER, 0, Some( &object_color_buffer ) );
    gl.bind_buffer( GL::UNIFORM_BUFFER, Some( &object_color_buffer ) );
    gl.buffer_data_with_i32( GL::UNIFORM_BUFFER, MAX_OBJECT_COUNT as i32 * 16, GL::STATIC_DRAW );
    renderer.object_colors = object_colors.into_iter().flatten().collect::< Vec< _ > >();
    gl::ubo::upload( &gl, &object_color_buffer, 0, &renderer.object_colors[ .. ], GL::STATIC_DRAW );

    renderer
  }

  /// Executes all rendering passes for a single frame.
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for animation ).
  fn render( &self, scenes : Vec< Rc< RefCell< Scene > > >, _t : f64 )
  {
    // 1. Object Rendering Pass: Render the object silhouette to a texture
    let _ = self.object_pass( scenes );
    self.outline_pass();
  }

  /// Renders the 3D object silhouette to the `object_fb`.
  ///
  /// Sets up the model-view-projection matrices and draws the loaded mesh.
  /// The fragment shader for this pass simply outputs white.
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for rotating the camera/view ).
  fn object_pass( &self, scenes : Vec< Rc< RefCell< Scene > > > ) -> Result<(), WebglError >
  {
    let gl = &self.gl;

    let object_fb = self.framebuffers.get( "object_fb" ).unwrap();

    let locations = self.programs.object.get_locations();

    let u_projection_loc = locations.get( "u_projection" ).unwrap().clone().unwrap();
    let u_view_loc = locations.get( "u_view" ).unwrap().clone().unwrap();
    let u_model_loc = locations.get( "u_model" ).unwrap().clone().unwrap();
    let u_normal_matrix_loc = locations.get( "u_normal_matrix" ).unwrap().clone().unwrap();
    let u_near_loc = locations.get( "near" ).unwrap().clone().unwrap();
    let u_far_loc = locations.get( "far" ).unwrap().clone().unwrap();

    upload_framebuffer( gl, object_fb, self.viewport );
    //gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    gl.clear_color( 0.0, 0.0, 0.0, 0.0 );
    gl.clear_depth( 1.0 );
    gl.clear( GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT );
    gl.enable( GL::DEPTH_TEST );

    // Define a closure to handle the drawing of each node in the scene.
    let mut draw_node =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      // If the node contains a mesh...
      if let Object3D::Mesh( ref mesh ) = node.borrow().object
      {
        // Iterate over each primitive in the mesh.
        for primitive_rc in mesh.borrow().primitives.iter()
        {
          let primitive = primitive_rc.borrow();

          self.programs.object.bind( gl );

          gl::uniform::upload( gl, Some( u_near_loc.clone() ), &[ 0.1 ] ).unwrap();
          gl::uniform::upload( gl, Some( u_far_loc.clone() ), &[ 1000.0 ] ).unwrap();

          gl::uniform::matrix_upload( gl, Some( u_projection_loc.clone() ), &self.camera.get_projection_matrix().to_array(), true ).unwrap();
          gl::uniform::matrix_upload( gl, Some( u_view_loc.clone() ), &self.camera.get_view_matrix().to_array(), true ).unwrap();
          gl::uniform::matrix_upload( gl, Some( u_model_loc.clone() ), &node.borrow().get_world_matrix().to_array(), true ).unwrap();
          let normal_matrix = self.camera.get_view_matrix() * node.borrow().get_world_matrix();
          gl::uniform::matrix_upload( gl, Some( u_normal_matrix_loc.clone() ), &normal_matrix.to_array(), true ).unwrap();

          primitive.bind( gl );
          primitive.draw( gl );
        }
      }

      Ok( () )
    };

    // Traverse the scene and draw all opaque objects.
    for scene in scenes
    {
      scene.borrow().traverse( &mut draw_node )?;
    }

    Ok( () )
  }

  /// Performs the final outline pass.
  ///
  /// Reads the original object silhouette texture and the final JFA result texture
  /// to draw the final scene with object color, outline color, or background color.
  /// Renders to the default framebuffer ( screen ).
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for animating outline thickness ).
  /// * `num_passes` - The total number of JFA step passes performed. Used to determine
  ///                 which of the ping-pong textures ( `jfa_step_fb_color_0` or `jfa_step_fb_color_1` )
  ///                 holds the final JFA result.
  fn outline_pass( &self )
  {
    let gl = &self.gl;

    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();
    let object_fb_depth = self.textures.get( "object_fb_depth" ).unwrap();
    let object_fb_norm = self.textures.get( "object_fb_norm" ).unwrap();

    self.programs.outline.bind( gl );
    let locations = self.programs.outline.get_locations();

    let u_color_texture_loc = locations.get( "u_color_texture" ).unwrap().clone().unwrap();
    let u_depth_texture_loc = locations.get( "u_depth_texture" ).unwrap().clone().unwrap();
    let u_norm_texture_loc = locations.get( "u_norm_texture" ).unwrap().clone().unwrap();
    //let u_projection_loc = locations.get( "u_projection" ).unwrap().clone().unwrap();
    let u_resolution_loc = locations.get( "u_resolution" ).unwrap().clone().unwrap();
    let u_outline_thickness_loc = locations.get( "u_outline_thickness" ).unwrap().clone().unwrap();
    let u_background_color_loc = locations.get( "u_background_color" ).unwrap().clone().unwrap();

    let outline_thickness = [ 1.0 as f32 ]; //[ ( 2.0 * ( t / 1000.0 ).sin().abs() ) as f32 ]; // Example animation
    let background_color = [ 0.0, 0.0, 0.0, 1.0 ];

    // Bind the default framebuffer ( render to canvas )
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    gl.clear_color( background_color[ 0 ], background_color[ 1 ], background_color[ 2 ], background_color[ 3 ] );

    upload_texture( gl, object_fb_color, &u_color_texture_loc, GL::TEXTURE0 );
    upload_texture( gl, object_fb_depth, &u_depth_texture_loc, GL::TEXTURE1 );
    upload_texture( gl, object_fb_norm, &u_norm_texture_loc, GL::TEXTURE2 );
    //gl::uniform::matrix_upload( gl, Some( u_projection_loc.clone() ), &self.camera.get_projection_matrix().to_array()[ .. ], true ).unwrap();
    gl::uniform::upload( gl, Some( u_resolution_loc.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_thickness_loc.clone() ), &outline_thickness ).unwrap();
    gl::uniform::upload( gl, Some( u_background_color_loc.clone() ), &background_color ).unwrap();

    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }
}

/// Sets up the application and runs the main rendering loop.
///
/// Initializes the renderer and defines the update/draw function that is called
/// by the `gl::exec_loop::run`.
///
/// # Returns
///
/// A `Result` indicating success or a WebGL error.
async fn run() -> Result< (), gl::WebglError >
{
  let renderer = Renderer::new().await;

  let _ = renderer.gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to enable EXT_color_buffer_float extension" );

  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let gltf_path = "bike.glb";
  let mut gltf = load( &document, gltf_path, &renderer.gl ).await?;
  let _ = add_attributes( &renderer.gl, &mut gltf );

  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let primitive_gltf = primitives_csgrs_gltf( &renderer.gl );
  let primitive_scenes = primitive_gltf.scenes.clone();
  primitive_scenes[ 0 ].borrow_mut().update_world_matrix();

  let s = vec![ scenes[ 0 ].clone(), primitive_scenes[ 0 ].clone() ];

  let update_and_draw =
  {
    move | t : f64 |
    {
      renderer.render( s.clone(), t );
      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// The main entry point of the application.
///
/// Spawns the asynchronous `run` function using `gl::spawn_local` which is
/// suitable for WebAssembly targets in a browser environment.
fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
