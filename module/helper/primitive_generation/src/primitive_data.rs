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
  use minwebgl as gl;
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
    AttributeInfo, Geometry, IndexInfo, Material, Mesh, Node, Object3D, Primitive, Scene, loaders::gltf::GLTF, material::PbrMaterial
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
    pub fn node_transform_set( &self, node : &RefCell< Node > )
    {
      let t = self.translation;
      let r = self.rotation;
      let s = self.scale;
      let mut node_mut = node.borrow_mut();
      node_mut.translation_set( t );
      let q = gl::Quat::from_euler_xyz( r );
      node_mut.rotation_set( q );
      node_mut.scale_set( s );
      node_mut.local_matrix_update();
    }
  }

  /// Mesh attribute data containing vertex positions and triangle indices.
  #[ derive( Debug ) ]
  pub struct AttributesData
  {
    /// Vertex positions in 3D space.
    pub positions : Vec< [ f32; 3 ] >,
    /// Triangle indices referencing the positions array.
    pub indices : Vec< u32 >,
    // Fix(BUG-217)
    // Root cause: `primitives_data_to_gltf` never uploaded any per-vertex
    // normal, yet `PbrMaterial`'s vertex shader unconditionally computes
    // `normalize( normalMatrix * normal )` from a `layout(location=1)`
    // attribute -- an unbound attribute reads WebGL's default `(0,0,0)`,
    // and `normalize` of the zero vector is NaN (`0 * inversesqrt(0)` =
    // `0 * Inf`), corrupting every downstream lighting calculation for
    // every primitive this crate generates.
    // Pitfall: a shader that unconditionally reads and normalizes a vertex
    // attribute gives no signal (no error, no panic) when the attribute
    // was never bound -- the defect only shows up as NaN in the final
    // shaded output, far from its actual cause.
    /// Per-vertex surface normals, parallel to `positions`.
    pub normals : Vec< [ f32; 3 ] >
  }

  /// Complete primitive data including geometry attributes, color, and transform.
  #[ derive( Clone ) ]
  pub struct PrimitiveData
  {
    /// Optional name of this primitive data.
    pub name : Option< Box< str > >,
    /// Parent of this primitive data.
    pub parent : Option< usize >,
    /// Shared mesh attribute data.
    pub attributes : Option< Rc< RefCell< AttributesData > > >,
    /// RGBA color values.
    pub color : F32x4,
    /// 3D transformation to apply to the primitive.
    pub transform : Transform
  }

  /// Creates an `AttributeInfo` object using one function call for a WebGL buffer.
  ///
  /// `attribute` carries the cross-backend location/vector-shape/offset description;
  /// `stride`/`normalized` are WebGL-only concerns `mingl::VertexAttribute` doesn't model,
  /// so they're layered on top of the bridged `BufferDescriptor` directly.
  #[ must_use ]
  pub fn buffer_attribute_info_make
  (
    buffer : &web_sys::WebGlBuffer,
    attribute : mingl::VertexAttribute,
    stride : i32,
    normalized : bool
  ) -> AttributeInfo
  {
    let descriptor = gl::BufferDescriptor::from_vector( attribute.vector )
    .offset( attribute.offset )
    .stride( stride )
    .normalized( normalized );

    AttributeInfo
    {
      slot : attribute.location,
      buffer : buffer.clone(),
      descriptor,
      bounding_box : mingl::geometry::BoundingBox::default()
    }
  }

  /// Validates that the `parent` links across `primitives_data` form an acyclic graph.
  ///
  /// Each primitive's `parent` chain is walked toward the root. An out-of-bounds parent
  /// index is treated as "no parent", matching `primitives_data_to_gltf`'s own wiring
  /// fallback ( an invalid index roots that node at the scene instead of erroring ). A
  /// self-reference or any longer cycle is rejected.
  ///
  /// # Errors
  ///
  /// Returns `Err` naming the primitive index that closes a cycle when the parent graph
  /// is not acyclic.
  // Fix(BUG-499)
  // Root cause: the parent/child wiring loop in `primitives_data_to_gltf` linked
  // `Rc<RefCell<Node>>` parent/child pointers directly from `PrimitiveData::parent`
  // indices with no acyclic check -- a self-referencing or cyclic parent index (e.g.
  // primitive 2's parent is primitive 2, or 0 -> 1 -> 0) built a `Node` graph containing
  // a reference cycle with no error surfaced, silently producing a broken/unbounded scene
  // graph (and, since `Node` uses `Rc<RefCell<_>>` parent/child links, a genuine memory
  // leak from the reference cycle that nothing would ever detect at runtime).
  // Pitfall: index-based parent links look like plain data until they're wired into
  // `Rc`/`RefCell` graph pointers -- validating the indices *before* wiring is the only
  // point where a cycle is cheap to detect and cheap to test (a plain `&[PrimitiveData]`
  // slice, no GL context or live `Node` graph needed); once wired, finding the same cycle
  // requires walking live `Rc` pointers instead.
  #[ must_use = "ignoring a cyclic parent graph error leaves callers wiring a `Node` graph with a leaked `Rc` reference cycle" ]
  pub fn primitives_parent_graph_validate( primitives_data : &[ PrimitiveData ] ) -> Result< (), String >
  {
    for start in 0..primitives_data.len()
    {
      let mut visited = std::collections::HashSet::new();
      visited.insert( start );
      let mut current = start;

      while let Some( parent_index ) = primitives_data[ current ].parent
      {
        if parent_index >= primitives_data.len()
        {
          // Out-of-bounds parent index is treated as "no parent", matching
          // `primitives_data_to_gltf`'s own wiring fallback.
          break;
        }

        if !visited.insert( parent_index )
        {
          return Err( format!(
            "primitive {start} has a cyclic parent chain that revisits primitive {parent_index}"
          ) );
        }

        current = parent_index;
      }
    }

    Ok( () )
  }

  /// Converts a collection of primitive data into a GLTF scene for WebGL rendering.
  ///
  /// # Panics
  ///
  /// Panics if the WebGL context fails to create a buffer ( e.g. a lost context ), or if
  /// `primitives_data`'s `parent` links form a cycle or self-reference ( see
  /// [`primitives_parent_graph_validate`] ).
  #[ expect( clippy::too_many_lines, reason = "GLTF assembly is inherently a flat sequence of buffer/mesh/node/scene construction steps; splitting it would scatter tightly-coupled local state ( buffers, meshes, nodes ) across artificial helper functions" ) ]
  #[ must_use ]
  pub fn primitives_data_to_gltf
  (
    gl : &WebGl2RenderingContext,
    primitives_data : &[ PrimitiveData ]
  ) -> GLTF
  {
    if let Err( cycle ) = primitives_parent_graph_validate( primitives_data )
    {
      panic!( "primitives_data_to_gltf: {cycle}" );
    }

    let mut scenes = vec![];
    let mut nodes = vec![];
    let mut gl_buffers = vec![];
    let mut meshes = vec![];

    let material : Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( PbrMaterial::new( gl ) ) ) );
    let materials = vec![ material.clone() ];

    scenes.push( Rc::new( RefCell::new( Scene::new() ) ) );

    let position_buffer = gl.create_buffer().unwrap();
    let normal_buffer = gl.create_buffer().unwrap();

    gl_buffers.push( position_buffer.clone() );
    gl_buffers.push( normal_buffer.clone() );

    // Fix(BUG-217): wire a "normal" attribute at slot 1, matching
    // `main.vert`'s `layout( location = 1 ) in vec3 normal;` -- see
    // `AttributesData::normals`'s own doc comment for the full root cause.
    let attribute_infos =
    [
      (
        "positions",
        buffer_attribute_info_make(
          &position_buffer,
          mingl::VertexAttribute::new( 0, VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 ),
          3,
          false
        )
      ),
      (
        "normal",
        buffer_attribute_info_make(
          &normal_buffer,
          mingl::VertexAttribute::new( 1, VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 ),
          3,
          false
        )
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
    let mut normals = vec![];

    // Create nodes for all primitives, even those without attributes (parent nodes)
    for primitive in primitives_data
    {
      let node = Rc::new( RefCell::new( Node::new() ) );

      // Assign name from the primitive record
      if let Some( name ) = &primitive.name
      {
        node.borrow_mut().name_set( name.clone() );
      }

      // Only create geometry/mesh if attributes exist
      if let Some( attributes ) = &primitive.attributes
      {
        let last_positions_count = positions.len() as u32;
        positions.extend( attributes.borrow().positions.clone() );
        normals.extend( attributes.borrow().normals.clone() );
        let primitive_indices = attributes.borrow().indices.iter()
        .map( | i | i + last_positions_count )
        .collect::< Vec< _ > >();
        let offset = indices.len() as u32 * 4;
        indices.extend( primitive_indices );

        index_info.offset = offset;
        index_info.count = attributes.borrow().indices.len() as u32;

        let Ok( mut geometry ) = Geometry::new( gl ) else
        {
          panic!( "Can't create new Geometry struct" );
        };

        for ( name, info ) in &attribute_infos
        {
          geometry.attribute_add( gl, *name, info.clone() ).unwrap();
        }

        geometry.index_add( gl, index_info.clone() ).unwrap();
        geometry.vertex_count = attributes.borrow().positions.len() as u32;

        let primitive = Primitive
        {
          geometry : Rc::new( RefCell::new( geometry ) ),
          material : material.clone()
        };

        let mesh = Rc::new( RefCell::new( Mesh::new() ) );
        mesh.borrow_mut().primitive_add( Rc::new( RefCell::new( primitive ) ) );

        node.borrow_mut().object = Object3D::Mesh( mesh.clone() );
        meshes.push( mesh );
      }

      // Set transform for all nodes (with or without geometry)
      primitive.transform.node_transform_set( &node );

      nodes.push( node.clone() );
    }

    // Set up parent-child relationships
    for ( i, node ) in nodes.iter().enumerate()
    {
      let primitive = &primitives_data[ i ];

      if let Some( parent_index ) = primitive.parent
      {
        // Get parent node and add this node as its child
        if let Some( parent_node ) = nodes.get( parent_index )
        {
          parent_node.borrow_mut().child_add( node.clone() );
          node.borrow_mut().parent_set( Some( parent_node.clone() ) );
        }
        else
        {
          // Parent index is out of bounds, add to scene root
          scenes[ 0 ].borrow_mut().children.push( node.clone() );
        }
      }
      else
      {
        // No parent specified, add as top-level child of scene
        scenes[ 0 ].borrow_mut().children.push( node.clone() );
      }
    }

    gl::buffer::upload( gl, &position_buffer, &positions, GL::STATIC_DRAW );
    gl::buffer::upload( gl, &normal_buffer, &normals, GL::STATIC_DRAW );
    gl::index::upload( gl, &index_buffer, &indices, GL::STATIC_DRAW );

    GLTF
    {
      scenes,
      nodes,
      gl_buffers,
      images : Rc::new( RefCell::new( vec![] ) ),
      textures : vec![],
      materials,
      meshes,
      animations : vec![],
      lights : vec![]
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
    primitives_parent_graph_validate,
    buffer_attribute_info_make
  };
}
