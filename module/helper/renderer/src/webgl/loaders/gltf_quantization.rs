//! `KHR_mesh_quantization` support for the glTF loader : turning integer accessor
//! values and bounds into the floats the shaders and the CPU-side skinning code see.

mod private
{
  /// Converts a `POSITION` accessor's `min` / `max` into the values the vertex
  /// shader sees. `KHR_mesh_quantization` stores positions as integers and the
  /// accessor's bounds use the same raw integers, but WebGL reads a normalized
  /// accessor as -1..1 or 0..1 : an `i16` bound of 32767 is really 1.0. Bounds of
  /// float and non-normalized integer accessors are already in shader units.
  #[ must_use ]
  pub fn position_bounds_dequantize
  (
    bounds : gltf::mesh::BoundingBox,
    data_type : gltf::accessor::DataType,
    normalized : bool
  )
  -> gltf::mesh::BoundingBox
  {
    if !normalized
    {
      return bounds;
    }
    let scale = | v : f32 | component_dequantize( v, data_type, normalized );
    gltf::mesh::BoundingBox
    {
      min : bounds.min.map( scale ),
      max : bounds.max.map( scale ),
    }
  }

  /// Maps one raw accessor component to the value WebGL would hand the shader : a
  /// normalized integer becomes -1..1 or 0..1, anything else is used as is.
  fn component_dequantize( v : f32, data_type : gltf::accessor::DataType, normalized : bool ) -> f32
  {
    use gltf::accessor::DataType;

    if !normalized
    {
      return v;
    }
    // Per the glTF spec's normalization equations : signed types clamp at -1.
    match data_type
    {
      DataType::I8 => ( v / 127.0 ).max( -1.0 ),
      DataType::U8 => v / 255.0,
      DataType::I16 => ( v / 32767.0 ).max( -1.0 ),
      DataType::U16 => v / 65535.0,
      DataType::U32 | DataType::F32 => v,
    }
  }

  /// Reads a `VEC3` accessor as floats, whatever its component type. `KHR_mesh_quantization`
  /// lets morph target `POSITION` / `NORMAL` / `TANGENT` be `i8` or `i16`, normalized or not,
  /// but the `gltf` crate's `ReadPositionDisplacements` and its siblings always read `f32`
  /// without checking the type. Reading by the declared type keeps the view's `byteStride`
  /// and sparse substitution, which `gltf::accessor::Iter` already handles.
  ///
  /// `None` when the accessor is not `VEC3` or its data is missing from `buffers`.
  #[ must_use ]
  pub fn accessor_vec3_read( acc : gltf::Accessor< '_ >, buffers : &[ Vec< u8 > ] ) -> Option< Vec< [ f32; 3 ] > >
  {
    use gltf::accessor::{ DataType, Dimensions, Iter };

    if acc.dimensions() != Dimensions::Vec3
    {
      return None;
    }
    let data_type = acc.data_type();
    let normalized = acc.normalized();
    let get = | buffer : gltf::Buffer< '_ > | buffers.get( buffer.index() ).map( Vec::as_slice );
    let scale = | v : [ f32; 3 ] | v.map( | c | component_dequantize( c, data_type, normalized ) );

    let values = match data_type
    {
      DataType::F32 => Iter::< [ f32; 3 ] >::new( acc, get )?.collect(),
      DataType::I8 => Iter::< [ i8; 3 ] >::new( acc, get )?.map( | v | scale( v.map( f32::from ) ) ).collect(),
      DataType::U8 => Iter::< [ u8; 3 ] >::new( acc, get )?.map( | v | scale( v.map( f32::from ) ) ).collect(),
      DataType::I16 => Iter::< [ i16; 3 ] >::new( acc, get )?.map( | v | scale( v.map( f32::from ) ) ).collect(),
      DataType::U16 => Iter::< [ u16; 3 ] >::new( acc, get )?.map( | v | scale( v.map( f32::from ) ) ).collect(),
      #[ allow( clippy::cast_precision_loss, reason = "glTF allows no `u32` displacements, so values past 2^24 losing precision does not matter" ) ]
      DataType::U32 => Iter::< [ u32; 3 ] >::new( acc, get )?.map( | v | v.map( | c | c as f32 ) ).collect(),
    };
    Some( values )
  }
}

crate::mod_interface!
{
  own use position_bounds_dequantize;
}

// Only `gltf.rs`'s morph target reader calls it.
pub( crate ) use private::accessor_vec3_read;

/// `accessor_vec3_read`, reachable from `tests/` under `test_internals`. Without the feature
/// this module does not exist.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
pub mod internal
{
  pub use super::private::accessor_vec3_read;
}
