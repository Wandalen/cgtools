/// Internal namespace.
mod private
{

  /// Enum representing basic WebGL data types.
  #[ derive( Clone, Copy, Debug, PartialEq, Hash, Eq ) ]
  #[ repr( u32 ) ]
  #[ non_exhaustive ]
  pub enum DataType
  {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
  }

  /// Represents a data type with a specified size.
  #[ derive( Clone, Copy, Debug, PartialEq, Hash, Eq ) ]
  pub struct VectorDataType
  {
    pub scalar : DataType,
    pub len : i32,
    pub element_len : i32,
    // xxx : usize?
  }

  impl VectorDataType
  {
    /// Creates a new `VectorDataType` with the given data type and size.
    pub fn new( scalar : DataType, len : i32, element_len : i32 ) -> Self
    {
      VectorDataType { scalar, len, element_len }
    }

    /// Returns the total byte size of the data type.
    pub fn byte_size( &self ) -> i32
    {
      self.scalar.byte_size() * self.len
    }

    /// Length in number of scalars of the data type.
    /// For flat structures it's equal to number of elements( components ).
    /// For multidimensional structures it's not equal to number of elements( components ).
    // xxx : usize?
    pub fn len( &self ) -> i32
    {
      self.len
    }

    // /// Length of an element( component ). For flat strcuture it'
    // pub fn element_len( &self ) -> i32
    // {
    //   self.len / self.element_len
    // }

    /// Length of an element. For flat strcutures it's always 1.
    /// For matrices it's number of scalars a row has.
    // xxx : qqq : verify
    pub fn element_len( &self ) -> i32
    {
      self.element_len
    }

    /// Returns the underlying data type.
    pub fn scalar( &self ) -> DataType
    {
      self.scalar
    }
  }

  impl DataType
  {
    /// Returns the size in bytes of the data type.
    pub fn byte_size( &self ) -> i32
    {
      match self
      {
        DataType::I8 | DataType::U8 => 1,
        DataType::I16 | DataType::U16 => 2,
        DataType::I32 | DataType::U32 => 4,
        DataType::F32 => 4,
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
