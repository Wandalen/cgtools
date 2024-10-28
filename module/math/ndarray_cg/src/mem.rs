/// Internal namespace.
mod private
{
  use crate::*;

  pub use bytemuck::
  {
    Pod,
  };

  /// Trait for converting data to byte slices.
  pub trait AsBytes
  {

    /// Returns the data as a byte slice.
    fn as_bytes( &self ) -> &[ u8 ];

    /// Returns the byte length of the data.
    #[ inline ]
    fn byte_size( &self ) -> usize
    {
      self.as_bytes().len()
    }

    /// Length in number of scalars of the data type.
    /// For flat structures it's equal to number of elements( components ).
    /// For multidimensional structures it's not equal to number of elements( components ).
    fn len( &self ) -> usize;

  }

  impl< A, D > AsBytes for nd::Array< A, D >
  where
    A : Pod,
    D : nd::Dimension,
  {

    #[ inline ]
    fn as_bytes( &self ) -> &[ u8 ]
    {
      bytemuck::cast_slice( self.as_slice().expect( "Multi-dimensional array is not continious to be treared as slice" ) )
    }

    #[ inline ]
    fn byte_size( &self ) -> usize
    {
      nd::Array::< A, D >::len( self ) * std::mem::size_of::< A >() / std::mem::size_of::< u8 >()
    }

    #[ inline ]
    fn len( &self ) -> usize
    {
      self.len()
    }

  }

  impl< T : Pod > AsBytes for Vec< T >
  {

    #[ inline ]
    fn as_bytes( &self ) -> &[ u8 ]
    {
      bytemuck::cast_slice( self )
    }

    #[ inline ]
    fn byte_size( &self ) -> usize
    {
      self.len() * std::mem::size_of::< T >() / std::mem::size_of::< u8 >()
    }

    #[ inline ]
    fn len( &self ) -> usize
    {
      self.len()
    }

  }

  impl< T : Pod > AsBytes for [ T ]
  {

    #[ inline ]
    fn as_bytes( &self ) -> &[ u8 ]
    {
      bytemuck::cast_slice( self )
    }

    #[ inline ]
    fn byte_size( &self ) -> usize
    {
      self.len() * std::mem::size_of::< T >() / std::mem::size_of::< u8 >()
    }

    #[ inline ]
    fn len( &self ) -> usize
    {
      self.len()
    }

  }

  impl< T : Pod, const N : usize > AsBytes for [ T ; N ]
  {

    #[ inline ]
    fn as_bytes( &self ) -> &[ u8 ]
    {
      bytemuck::cast_slice( self )
    }

    #[ inline ]
    fn byte_size( &self ) -> usize
    {
      self.len() * std::mem::size_of::< T >() / std::mem::size_of::< u8 >()
    }

    #[ inline ]
    fn len( &self ) -> usize
    {
      N
    }

  }

}

crate::mod_interface!
{
  // #![ debug ]

  orphan use
  {
    Pod,
    AsBytes,
  };

  own use ::bytemuck::*;

}
