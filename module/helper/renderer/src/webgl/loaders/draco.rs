//! `KHR_draco_mesh_compression` geometry decode for the glTF loader.
//!
//! The decode itself is delegated to the pure-Rust `draco-gltf` / `draco-core`
//! crates; this layer only bridges the decoded [`draco_core::mesh::Mesh`] into
//! the loader's existing [`Geometry`] / [`AttributeInfo`] / [`IndexInfo`] types,
//! reusing the same attribute slots as the uncompressed accessor path.
//!
//! Attribute **format** ( component type, dimensions, `normalized` ) is read
//! from the glTF **accessor**, not from the decoded Draco attribute: Draco does
//! not carry glTF's `normalized` semantic, and models authored with
//! `KHR_mesh_quantization` store positions / texcoords as normalized integers
//! that the GPU must normalize. The Draco attribute only supplies the decoded
//! bytes ( which match the accessor's component type ).

mod private
{
  use minwebgl as gl;
  use gl::geometry::BoundingBox;
  use crate::webgl::{ AttributeInfo, Geometry, IndexInfo };

  use draco_core::geometry_indices::{ FaceIndex, PointIndex };

  /// Map a glTF accessor component type to the WebGL scalar type used by the
  /// loader — mirrors the accessor path's `make_attibute_info` in `gltf.rs`.
  fn to_gl_data_type( dt : gltf::accessor::DataType ) -> gl::DataType
  {
    match dt
    {
      gltf::accessor::DataType::U8 => gl::DataType::U8,
      gltf::accessor::DataType::I8 => gl::DataType::I8,
      gltf::accessor::DataType::U16 => gl::DataType::U16,
      gltf::accessor::DataType::I16 => gl::DataType::I16,
      gltf::accessor::DataType::U32 => gl::DataType::U32,
      gltf::accessor::DataType::F32 => gl::DataType::F32,
    }
  }

  /// Divisor mapping a normalized integer component to its `[ -1, 1 ]` / `[ 0, 1 ]`
  /// float value, per the glTF spec. Used to bring the POSITION accessor's
  /// `min`/`max` ( stored in the integer space under `KHR_mesh_quantization` )
  /// into the same normalized local space the vertex shader sees. `None` for
  /// float ( already in local space ).
  fn normalize_divisor( dt : gltf::accessor::DataType ) -> Option< f32 >
  {
    match dt
    {
      gltf::accessor::DataType::I8 => Some( 127.0 ),
      gltf::accessor::DataType::U8 => Some( 255.0 ),
      gltf::accessor::DataType::I16 => Some( 32767.0 ),
      gltf::accessor::DataType::U16 => Some( 65535.0 ),
      gltf::accessor::DataType::U32 | gltf::accessor::DataType::F32 => None,
    }
  }

  /// Map a glTF attribute semantic — as spelled in the Draco extension's
  /// attribute map ( `POSITION`, `TEXCOORD_0`, `JOINTS_0`, … ) — to the loader's
  /// `( slot, attribute name, optional vertex-shader define )`. The slots and
  /// names mirror the accessor path in `gltf.rs` exactly.
  fn slot_of( semantic : &str ) -> Option< ( u32, String, Option< String > ) >
  {
    let indexed = | prefix : &str | -> Option< u32 >
    {
      semantic.strip_prefix( prefix ).and_then( | s | s.parse::< u32 >().ok() )
    };

    match semantic
    {
      "POSITION" => Some( ( 0, "positions".into(), None ) ),
      "NORMAL" => Some( ( 1, "normals".into(), None ) ),
      "TANGENT" => Some( ( 9, "tangents".into(), Some( "tangents".into() ) ) ),
      _ =>
      {
        indexed( "TEXCOORD_" ).map( | i | ( 2 + i, format!( "texture_coordinates_{}", 2 + i ), None ) )
        .or_else( || indexed( "COLOR_" ).map( | i | ( 7 + i, format!( "colors_{}", 7 + i ), None ) ) )
        .or_else( || indexed( "JOINTS_" ).map( | i | ( 10 + i, format!( "joints_{}", i ), Some( format!( "joints_{}", i ) ) ) ) )
        .or_else( || indexed( "WEIGHTS_" ).map( | i | ( 13 + i, format!( "weights_{}", i ), Some( format!( "weights_{}", i ) ) ) ) )
      }
    }
  }

  /// Decode a Draco-compressed primitive and populate `geometry` with its
  /// attributes and indices, reusing the loader's regular buffer/attribute
  /// upload types.
  ///
  /// Returns the list of vertex-shader define names ( `tangents`, `joints_N`,
  /// `weights_N` ) the caller must register on the primitive's material, exactly
  /// as the accessor path does via its `add_define` closure.
  ///
  /// `buffers` are the resolved buffer bytes ( the loader's `bin_buffers` ), and
  /// `document` / `primitive` are the parsed glTF handles — the accessor
  /// metadata ( component type, `normalized`, POSITION `min`/`max` ) stays the
  /// source of truth per the extension spec.
  pub fn load_into_geometry
  (
    gl : &gl::WebGl2RenderingContext,
    geometry : &mut Geometry,
    document : &gltf::Document,
    buffers : &[ Vec< u8 > ],
    primitive : &gltf::Primitive< '_ >,
  ) -> Result< Vec< String >, gl::WebglError >
  {
    let mesh = draco_gltf::decode_primitive( document, buffers, primitive )
    .map_err( | e |
    {
      gl::browser::error!( "Failed to decode Draco primitive: {e}" );
      gl::WebglError::Other( "Failed to decode Draco geometry" )
    } )?;

    let num_points = mesh.num_points();
    geometry.vertex_count = num_points as u32;

    let Some( attribute_map ) = draco_gltf::draco_attribute_map( primitive )
    else
    {
      return Err( gl::WebglError::Other( "Draco primitive is missing its attribute map" ) );
    };

    let mut defines = Vec::new();

    // Iterate the primitive's glTF attributes so the accessor stays the source
    // of truth for the vertex format; the Draco attribute ( looked up by the
    // extension's attribute map ) only supplies the decoded bytes.
    for ( semantic, accessor ) in primitive.attributes()
    {
      let sem = semantic.to_string();
      let Some( ( slot, name, define ) ) = slot_of( &sem )
      else
      {
        gl::warn!( "Unsupported Draco attribute semantic: {sem}" );
        continue;
      };

      let Some( unique_id ) = attribute_map.iter().find( | ( s, _ ) | *s == sem ).map( | ( _, u ) | *u )
      else
      {
        gl::warn!( "Draco attribute map has no entry for {sem}" );
        continue;
      };

      let att_id = mesh.attribute_id_by_unique_id( unique_id );
      if att_id < 0
      {
        gl::warn!( "Draco attribute {sem} ( id {unique_id} ) missing in decoded mesh" );
        continue;
      }
      let att = mesh.attribute( att_id );

      // Vertex format comes from the accessor ( see module docs ).
      let data_type = to_gl_data_type( accessor.data_type() );
      let components = accessor.dimensions().multiplicity() as i32;
      let normalized = accessor.normalized();
      let value_size = components as usize * data_type.byte_size() as usize;

      // Draco deduplicates attribute values, so a point id maps to a value id
      // via `mapped_index`. Expand to one tightly-packed value per point so the
      // per-point index buffer ( built from faces below ) addresses it directly.
      let src_stride = att.byte_stride() as usize;
      if value_size > src_stride
      {
        return Err( gl::WebglError::Other( "Draco attribute is narrower than its glTF accessor" ) );
      }
      let data = att.buffer().data();
      let mut packed = Vec::with_capacity( num_points * value_size );
      for p in 0 .. num_points as u32
      {
        let value_index = att.mapped_index( PointIndex( p ) ).0 as usize;
        let start = value_index * src_stride;
        let end = start + value_size;
        if end > data.len()
        {
          return Err( gl::WebglError::Other( "Decoded Draco attribute value out of range" ) );
        }
        packed.extend_from_slice( &data[ start .. end ] );
      }

      let buffer = gl::buffer::create( gl )?;
      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &buffer ) );
      gl.buffer_data_with_u8_array( gl::ARRAY_BUFFER, &packed, gl::STATIC_DRAW );

      let descriptor = gl::BufferDescriptor::new::< [ f32; 1 ] >()
      .offset( 0 )
      .normalized( normalized )
      .stride( 0 )
      .vector( gl::VectorDataType::new( data_type, components, 1 ) );

      let mut attr_info = AttributeInfo
      {
        slot,
        buffer,
        descriptor,
        bounding_box : Default::default(),
      };

      // The POSITION accessor still carries valid min/max. Under
      // `KHR_mesh_quantization` they live in the integer space, so bring them
      // into the same normalized local space the shader renders in.
      if slot == 0
      {
        let bb = primitive.bounding_box();
        let ( mut min, mut max ) = ( bb.min, bb.max );
        if normalized
        {
          if let Some( d ) = normalize_divisor( accessor.data_type() )
          {
            for c in 0 .. 3
            {
              min[ c ] = ( min[ c ] / d ).max( -1.0 );
              max[ c ] = ( max[ c ] / d ).max( -1.0 );
            }
          }
        }
        attr_info.bounding_box = BoundingBox::new( min, max );
      }

      geometry.add_attribute( gl, name, attr_info )?;

      if let Some( d ) = define
      {
        defines.push( d );
      }
    }

    // Indices: each Draco face is a triangle of point indices.
    let num_faces = mesh.num_faces();
    let mut index_bytes = Vec::with_capacity( num_faces * 3 * 4 );
    for f in 0 .. num_faces as u32
    {
      let face = mesh.face( FaceIndex( f ) );
      for corner in face
      {
        index_bytes.extend_from_slice( &corner.0.to_le_bytes() );
      }
    }

    let index_buffer = gl::buffer::create( gl )?;
    gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &index_buffer ) );
    gl.buffer_data_with_u8_array( gl::ELEMENT_ARRAY_BUFFER, &index_bytes, gl::STATIC_DRAW );

    geometry.add_index
    (
      gl,
      IndexInfo
      {
        buffer : index_buffer,
        count : ( num_faces * 3 ) as u32,
        offset : 0,
        data_type : gl::UNSIGNED_INT,
      },
    )?;

    Ok( defines )
  }
}

crate::mod_interface!
{
  own use
  {
    load_into_geometry,
  };
}
