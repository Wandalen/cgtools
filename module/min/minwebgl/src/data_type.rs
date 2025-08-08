/// Internal namespace.
mod private
{
  use crate::*;
  use core::
  {
    marker::PhantomData,
    ops::Deref,
  };

  /// A generic wrapper for a `u32` value, used to represent WebGL constants for a specific `Marker` type.
  #[ derive( Debug, PartialEq, Eq, Hash, Clone, Copy ) ]
  pub struct Const< Marker >( u32, PhantomData< Marker > );

  impl< Marker > Const< Marker >
  {
    /// Creates a new `Const` instance with the given `u32` value.
    fn new( val : u32 ) -> Self
    {
      Self( val, Default::default() )
    }
  }

  impl< Marker > Deref for Const< Marker >
  {
    /// The target type of the dereference operation.
    type Target = u32;
    /// Dereferences the `Const` to a reference to the inner `u32`.
    fn deref( &self ) -> &Self::Target
    {
      &self.0
    }
  }

  /// Represents errors related to data types incompatibility.
  #[ derive( Debug, error::typed::Error ) ]
  pub enum Error
  {
    /// The given data type cannot be converted because it has no corresponding WebGL2 value.
    #[ error( "Type {0} can't be converted into {1}, because there is no corresponding value for {2} for WebGL2" ) ]
    NoCorrespndingType( &'static str, &'static str, String ),
  }

  impl TryFrom< DataType > for Const< DataType >
  {
    /// The error type returned if the conversion fails.
    type Error = Error;

    /// Attempts to convert a `DataType` enum variant to its corresponding WebGL `u32` constant.
    fn try_from( value : DataType ) -> Result< Self, Self::Error >
    {
      use core::any::{ type_name_of_val, type_name };
      match value
      {
        DataType::I8 => Ok( Const::new( 0x1400 ) ),
        DataType::U8 => Ok( Const::new( 0x1401 ) ),
        DataType::I16 => Ok( Const::new( 0x1402 ) ),
        DataType::U16 => Ok( Const::new( 0x1403 ) ),
        DataType::I32 => Ok( Const::new( 0x1404 ) ),
        DataType::U32 => Ok( Const::new( 0x1405 ) ),
        DataType::F32 => Ok( Const::new( 0x1406 ) ),
        _ => Err( Error::NoCorrespndingType( type_name_of_val( &value ), type_name::< Self >(), format!( "{:?}", value ) ) ),
      }
    }
  }

  impl TryFrom< Const< DataType > > for DataType
  {
    /// The error type returned if the conversion fails.
    type Error = Error;

    /// Attempts to convert a WebGL `u32` constant wrapped in `Const<DataType>` back to its corresponding `DataType` enum variant.
    fn try_from( value : Const< DataType > ) -> Result< Self, Self::Error >
    {
      use core::any::{ type_name_of_val, type_name };
      match value
      {
        Const( 0x1400, .. ) => Ok( DataType::I8 ),
        Const( 0x1401, .. ) => Ok( DataType::U8 ),
        Const( 0x1402, .. ) => Ok( DataType::I16 ),
        Const( 0x1403, .. ) => Ok( DataType::U16 ),
        Const( 0x1404, .. ) => Ok( DataType::I32 ),
        Const( 0x1405, .. ) => Ok( DataType::U32 ),
        Const( 0x1406, .. ) => Ok( DataType::F32 ),
        _ => Err( Error::NoCorrespndingType( type_name_of_val( &value ), type_name::< Self >(), format!( "{:?}", value ) ) ),
      }
    }
  }

}

crate::mod_interface!
{

  reuse ::mingl::data_type;

  own use
  {
    Const,
    Error,
  };

}
