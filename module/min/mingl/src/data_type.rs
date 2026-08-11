/// Internal namespace.
mod private
{

  /// Enum representing basic WebGL data types.
  #[ derive( Clone, Copy, Debug, PartialEq, Hash, Eq ) ]
  #[ repr( u32 ) ]
  #[ non_exhaustive ]
  pub enum DataType
  {
    /// 8-bit signed integer.
    I8,
    /// 8-bit unsigned integer.
    U8,
    /// 16-bit signed integer.
    I16,
    /// 16-bit unsigned integer.
    U16,
    /// 32-bit signed integer.
    I32,
    /// 32-bit unsigned integer.
    U32,
    /// 32-bit floating-point number.
    F32,
  }

  /// Represents a data type with a specified size.
  ///
  /// Atom represent undivisible part of vector.
  /// While element could have subelements.
  ///
  /// Code below illustrate of what each field means
  /// ```rust, ignore
  /// impl< const N : usize, const N2 : usize > IntoVectorDataType for [ [ u32 ; N2 ] ; N ]
  /// {
  ///   fn into_vector_data_type() -> VectorDataType
  ///   {
  ///     VectorDataType
  ///     {
  ///       scalar : DataType::U32,
  ///       natoms : ( N * N2 ) as i32,
  ///       nelements : N2 as _,
  ///     }
  ///   }
  /// }
  /// ```
  // Counts are `i32`, not `usize`, deliberately: descriptors feed WebGL parameter slots
  // (`GLint` size/stride/offset in `vertex_attrib_pointer`-family calls), and consumers
  // (minwebgl buffer/geometry, renderer gltf loader) do `i32` arithmetic on them directly —
  // `usize` would force a cast at every GL boundary.
  #[ derive( Clone, Copy, Debug, PartialEq, Hash, Eq ) ]
  #[ non_exhaustive ]
  pub struct VectorDataType
  {
    /// The scalar data type used for the elements (e.g., f32, f64).
    pub scalar : DataType,
    /// The number of atoms in the data structure.
    pub natoms : i32,
    /// The number of elements in the data structure.
    pub nelements : i32,
  }

  impl VectorDataType
  {
    /// Creates a new `VectorDataType` with the given data type and size.
    #[ inline ]
    #[ must_use ]
    pub fn new( scalar : DataType, natoms : i32, nelements : i32 ) -> Self
    {
      VectorDataType { scalar, natoms, nelements }
    }

    /// Returns the total byte size of the data type.
    #[ inline ]
    #[ must_use ]
    pub fn byte_size( &self ) -> i32
    {
      self.scalar.byte_size() * self.natoms
    }

    /// Length in number of scalars of the data type.
    /// For flat structures it's equal to number of atoms( components ).
    /// For multidimensional structures it's not equal to number of atoms( components ).
    #[ inline ]
    #[ must_use ]
    pub fn natoms( &self ) -> i32
    {
      self.natoms
    }

    /// Length of an element. For flat strcutures it's always 1.
    /// For matrices it's number of scalars a row has.
    #[ inline ]
    #[ must_use ]
    pub fn nelements( &self ) -> i32
    {
      self.nelements
    }

    /// Returns the underlying data type.
    #[ inline ]
    #[ must_use ]
    pub fn scalar( &self ) -> DataType
    {
      self.scalar
    }
  }

  impl DataType
  {
    /// Returns the size in bytes of the data type.
    #[ inline ]
    #[ must_use ]
    pub fn byte_size( &self ) -> i32
    {
      match self
      {
        DataType::I8 | DataType::U8 => 1,
        DataType::I16 | DataType::U16 => 2,
        DataType::I32 | DataType::U32 | DataType::F32 => 4,
      }
    }
  }

  /// Trait for converting types into `VectorDataType`.
  pub trait IntoVectorDataType
  {
    /// Converts the type into a `VectorDataType`.
    fn into_vector_data_type() -> VectorDataType;
  }

}

// `n` below is the component count of a GL vector/matrix attribute type (see
// `VectorDataType`'s doc comment above). The WebGL spec caps `vertexAttribPointer`'s
// `size` parameter at 4, so any `n` that is actually usable as a GL vector/matrix
// dimension (at most 4, or 16 for a 4x4 matrix) is far below `i32::MAX` — this cast
// can never wrap or truncate for such `n`.
#[ allow( clippy::cast_possible_truncation, clippy::cast_possible_wrap ) ]
const fn dim_as_i32( n : usize ) -> i32
{
  n as i32
}

mod f32;
mod i8;
mod i16;
mod i32;
mod u8;
mod u16;
mod u32;

crate::mod_interface!
{

  exposed use
  {
    DataType,
    VectorDataType,
    IntoVectorDataType,
  };

}
