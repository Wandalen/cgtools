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
    IndexInfo
  };
  use primitive_generation::primitive_data::
  {
    Transform,
    AttributesData,
    make_buffer_attribute_info
  };
  use std::ops::Range;

  /// Defines the dynamic behavior of a primitive, including animation, repetition, and color.
  #[ derive( Debug, Clone ) ]
  pub struct Behavior
  {
    pub animated_transform : Option< interpoli::Transform >,
    pub repeater : Option< interpoli::Repeater >,
    pub brush : interpoli::Brush,
    pub frames : Range< f64 >,
  }

  impl Default for Behavior
  {
    fn default() -> Self
    {
      Self
      {
        animated_transform : Default::default(),
        repeater : Default::default(),
        brush : interpoli::Brush::Fixed( peniko::Brush::default() ),
        frames : 0.0..0.0
      }
    }
  }

  /// A struct that holds all the data required to represent a graphical primitive.
  #[ derive( Debug, Clone ) ]
  pub struct PrimitiveData
  {
    pub name : Option< Box< str > >,
    pub attributes : Option< Rc< RefCell< AttributesData > > >,
    pub parent : Option< usize >,
    pub transform : Transform,
    pub behavior : Behavior
  }

  impl PrimitiveData
  {
    /// Creates a new `PrimitiveData` instance.
    ///
    /// # Arguments
    ///
    /// * `attributes` - An `Option` containing a reference-counted, mutable `AttributesData`.
    ///
    /// # Returns
    ///
    /// A new `PrimitiveData` with default values for its fields, except for `attributes`.
    pub fn new( attributes : Option< Rc< RefCell< AttributesData > > > ) -> Self
    {
      Self
      {
        name : None,
        attributes,
        parent : None,
        behavior : Behavior::default(),
        transform : Transform::default(),
      }
    }
  }

  /// Converts a vector of `PrimitiveData` structs into a `GLTF` scene.
  ///
  /// This function takes a collection of `PrimitiveData` objects, each representing
  /// a single graphical primitive, and converts them into a `GLTF` format suitable for rendering.
  /// It aggregates all position and index data into global buffers, creates the
  /// corresponding `Node`, `Mesh`, `Primitive`, `Geometry`, and `Material` objects,
  /// and constructs the scene graph hierarchy.
  ///
  /// # Arguments
  ///
  /// * `gl` - A `WebGl2RenderingContext` instance for creating and uploading buffer data.
  /// * `primitives_data` - A `Vec` of `PrimitiveData` structs to convert.
  ///
  /// # Returns
  ///
  /// A `GLTF` struct representing the constructed scene.
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
        make_buffer_attribute_info
        (
          &position_buffer,
          BufferDescriptor::new::< [ f32; 3 ] >(),
          0,
          3,
          0,
          false,
          VectorDataType::new( mingl::DataType::F32, 3, 1 )
        )
        .unwrap()
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

    for primitive_data in &primitives_data
    {
      let object = if let Some( attributes ) = &primitive_data.attributes
      {
        let last_positions_count = positions.len() as u32;
        positions.extend( attributes.borrow().positions.clone() );
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
          geometry.add_attribute( gl, *name, info.clone(), false ).unwrap();
        }

        geometry.add_index( gl, index_info.clone() ).unwrap();
        geometry.vertex_count = attributes.borrow().positions.len() as u32;

        let primitive = Primitive
        {
          geometry : Rc::new( RefCell::new( geometry ) ),
          material : material.clone()
        };

        let mesh = Rc::new( RefCell::new( Mesh::new() ) );
        mesh.borrow_mut().add_primitive( Rc::new( RefCell::new( primitive ) ) );
        meshes.push( mesh.clone() );
        Object3D::Mesh( mesh )
      }
      else
      {
        Object3D::Other
      };

      let node = Rc::new( RefCell::new( Node::new() ) );
      node.borrow_mut().object = object;

      if let Some( name ) = &primitive_data.name
      {
        node.borrow_mut().set_name( name.clone() );
      }

      primitive_data.transform.set_node_transform( node.clone() );

      nodes.push( node.clone() );
      if primitive_data.parent.is_none()
      {
        scenes[ 0 ].borrow_mut().children.push( node );
      }
    }

    gl::buffer::upload( &gl, &position_buffer, &positions, GL::STATIC_DRAW );
    gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

    let node_iter = nodes.iter()
    .zip( primitives_data.iter().map( | p | p.parent ) );

    for ( child, parent ) in node_iter
    {
      if let Some( parent ) = parent
      {
        if let Some( parent ) = nodes.get( parent )
        {
          if parent.borrow().get_name() != child.borrow().get_name() && parent.borrow().get_name().is_some()
          {
            child.borrow_mut().set_parent( Some( parent.clone() ) );
            parent.borrow_mut().add_child( child.clone() );
          }
        }
      }
    }

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

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  orphan use
  {
    Behavior,
    PrimitiveData,
    primitives_data_to_gltf
  };
}
