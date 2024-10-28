/// Internal namespace.
mod private
{
  use crate::*;
  use core::
  {
    marker::PhantomData,
    ops::Deref,
  };

  #[ derive( Debug, PartialEq, Eq, Hash, Clone, Copy ) ]
  pub struct Const< Marker >( u32, PhantomData< Marker > );

  impl< Marker > Const< Marker >
  {
    fn new( val : u32 ) -> Self
    {
      Self( val, Default::default() )
    }
  }

  impl< Marker > Deref for Const< Marker >
  {
    type Target = u32;
    fn deref( &self ) -> &Self::Target
    {
      &self.0
    }
  }

  /// Represents errors related to data types incompatibility.
  #[ derive( Debug, error::typed::Error ) ]
  pub enum Error
  {
    /// Manifest data not loaded.
    #[ error( "Type {0} can't be converted into {1}, because there is no corresponding value for {2} for WebGL2" ) ]
    NoCorrespndingType( &'static str, &'static str, String ),
  }

  impl TryFrom< DataType > for Const< DataType >
  {
    type Error = Error;

    /// Attempts to convert `DataType` to u32.
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
    type Error = Error;

    /// Attempts to convert a `u32` to a `DataType`.
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
