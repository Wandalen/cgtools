//! This module provides a set of tools for working with 3D primitives
//! and transforming them into a GLTF scene graph for rendering. It
//! defines essential data structures:
//!
//! - `Transform` for manipulating an object's position, rotation,
//! and scale.
//!
//! - `AttributesData` and `PrimitiveData` for holding vertex and index
//! data along with other primitive properties.
//!
//! The core functionality is encapsulated in `primitives_data_to_gltf`,
//! which takes a collection of `PrimitiveData` and constructs a complete
//! `GLTF` object, including all necessary WebGL buffers, geometries
//! and a scene hierarchy.
mod private
{
  use minwebgl::
  {
    self as gl,
    BufferDescriptor
  };
  use gl::
  {
    GL,
    F32x4,
    F32x3,
    WebGl2RenderingContext,
    VectorDataType
  };
  use std::cell::RefCell;
  use std::rc::Rc;
  use renderer::webgl::
  {
    Node,
    Object3D,
    Scene,
    Material,
    Mesh,
    Primitive,
    loaders::gltf::GLTF,
    Geometry,
    IndexInfo,
    AttributeInfo
  };

  /// 3D transformation data including translation, rotation, and scale components.
  #[ derive( Debug, Clone ) ]
  pub struct Transform
  {
    /// Position offset in 3D space.
    pub translation : F32x3,
    /// Rotation angles in radians for each axis.
    pub rotation : F32x3,
    /// Scale factors for each axis.
    pub scale : F32x3
  }

  impl Default for Transform
  {
    /// Returns a new `Transform` with default values: translation and rotation are zero, and scale is one.
    fn default() -> Self
    {
      Self
      {
        translation : [ 0.0; 3 ].into(),
        rotation : [ 0.0; 3 ].into(),
        scale : [ 1.0; 3 ].into()
      }
    }
  }

  impl Transform
  {
    /// Set new local matrix of `Node`.
    pub fn set_node_transform( &self, node : Rc< RefCell< Node > > )
    {
      let t = self.translation;
      let r = self.rotation;
      let s = self.scale;
      let mut node_mut = node.borrow_mut();
      node_mut.set_translation( t );
      let q = gl::Quat::from_euler_xyz( r );
      node_mut.set_rotation( q );
      node_mut.set_scale( s );
      node_mut.update_local_matrix();
    }
  }

  /// Mesh attribute data containing vertex positions and triangle indices.
  #[ derive( Debug ) ]
  pub struct AttributesData
  {
    /// Vertex positions in 3D space.
    pub positions : Vec< [ f32; 3 ] >,
    /// Triangle indices referencing the positions array.
    pub indices : Vec< u32 >
  }

  /// Complete primitive data including geometry attributes, color, and transform.
  #[ derive( Clone ) ]
  pub struct PrimitiveData
  {
    /// Shared mesh attribute data.
    pub attributes : Rc< RefCell< AttributesData > >,
    /// RGBA color values.
    pub color : F32x4,
    /// 3D transformation to apply to the primitive.
    pub transform : Transform
  }

  /// Creates an `AttributeInfo` object using one function call for a WebGL buffer.
  pub fn make_buffer_attribute_info
  (
    buffer : &web_sys::WebGlBuffer,
    descriptor : gl::BufferDescriptor,
    offset : i32,
    stride : i32,
    slot : u32,
    normalized : bool,
    vector: gl::VectorDataType
  ) -> Result< AttributeInfo, gl::WebglError >
  {
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

  /// Converts a collection of primitive data into a GLTF scene for WebGL rendering.
  pub fn primitives_data_to_gltf
  (
    gl : &WebGl2RenderingContext,
    primitives_data : Vec< PrimitiveData >
  ) -> GLTF
  {
    let mut scenes = vec![];
    let mut nodes = vec![];
    let mut gl_buffers = vec![];
    let mut meshes = vec![];

    let material = Rc::new( RefCell::new( Material::default() ) );
    let materials = vec![ material.clone() ];

    scenes.push( Rc::new( RefCell::new( Scene::new() ) ) );

    let position_buffer = gl.create_buffer().unwrap();

    gl_buffers.push( position_buffer.clone() );

    let attribute_infos =
    [
      (
        "positions",
        make_buffer_attribute_info(
          &position_buffer,
          BufferDescriptor::new::< [ f32; 3 ] >(),
          0,
          3,
          0,
          false,
          VectorDataType::new( mingl::DataType::F32, 3, 1 )
        ).unwrap()
      ),
    ];

    let index_buffer = gl.create_buffer().unwrap();
    gl_buffers.push( index_buffer.clone() );

    let mut index_info = IndexInfo
    {
      buffer : index_buffer.clone(),
      count : 0,
      offset : 0,
      data_type : GL::UNSIGNED_INT
    };

    let mut positions = vec![];
    let mut indices = vec![];

    for primitive_data in primitives_data
    {
      let last_positions_count = positions.len() as u32;
      positions.extend( primitive_data.attributes.borrow().positions.clone() );
      let primitive_indices = primitive_data.attributes.borrow().indices.iter()
      .map( | i | i + last_positions_count )
      .collect::< Vec< _ > >();
      let offset = indices.len() as u32 * 4;
      indices.extend( primitive_indices );

      index_info.offset = offset;
      index_info.count = primitive_data.attributes.borrow().indices.len() as u32;

      let Ok( mut geometry ) = Geometry::new( gl ) else
      {
        panic!( "Can't create new Geometry struct" );
      };

      for ( name, info ) in &attribute_infos
      {
        geometry.add_attribute( gl, *name, info.clone(), false ).unwrap();
      }

      geometry.add_index( gl, index_info.clone() ).unwrap();
      geometry.vertex_count = primitive_data.attributes.borrow().positions.len() as u32;

      let primitive = Primitive
      {
        geometry : Rc::new( RefCell::new( geometry ) ),
        material : material.clone()
      };

      let mesh = Rc::new( RefCell::new( Mesh::new() ) );
      mesh.borrow_mut().add_primitive( Rc::new( RefCell::new( primitive ) ) );

      let node = Rc::new( RefCell::new( Node::new() ) );
      node.borrow_mut().object = Object3D::Mesh( mesh.clone() );
      primitive_data.transform.set_node_transform( node.clone() );

      nodes.push( node.clone() );
      meshes.push( mesh );
      scenes[ 0 ].borrow_mut().children.push( node );
    }

    gl::buffer::upload( &gl, &position_buffer, &positions, GL::STATIC_DRAW );
    gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

    GLTF
    {
      scenes,
      nodes,
      gl_buffers,
      images : Rc::new( RefCell::new( vec![] ) ),
      textures : vec![],
      materials,
      meshes,
      animations : vec![]
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Transform,
    PrimitiveData,
    AttributesData,
    primitives_data_to_gltf,
    make_buffer_attribute_info
  };
}
