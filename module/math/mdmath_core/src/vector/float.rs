/// Internal namespace.
mod private
{
  use crate::{ ToRef, Float };
  use core::iter::{ Map, Iterator };

  // Sealing the trait to prevent external implementations
  mod sealed
  {
    pub trait Sealed {}
    impl< T > Sealed for T where T : Iterator {}
  }

  /// Extension trait for iterators over floating-point numbers.
  pub trait IterExt
  where
    Self : Iterator + sealed::Sealed,
  {
    /// True only if all elemts are true.
    fn all_true( &mut self ) -> bool
    where
      Self : Sized,
      < Self as Iterator >::Item : ToRef< bool >,
      ;
    /// True if any elemts is true.
    fn any_true( &mut self ) -> bool
    where
      Self : Sized,
      < Self as Iterator >::Item : ToRef< bool >,
      ;
  }

  impl< I > IterExt for I
  where
    I : Iterator,
    // IntoBool : Into< bool >,
  {
    #[ inline ]
    fn all_true( &mut self ) -> bool
    where
      Self : Sized,
      < Self as Iterator >::Item : ToRef< bool >,
    {
      self.all( | x : Self::Item | *x.to_ref() )
    }
    #[ inline ]
    fn any_true( &mut self ) -> bool
    where
      Self : Sized,
      < Self as Iterator >::Item : ToRef< bool >,
    {
      self.any( | x : Self::Item | *x.to_ref() )
    }
  }

  /// Extension trait for iterators over floating-point numbers.
  pub trait IterFloat
  where
    Self : Iterator + IterExt + sealed::Sealed,
  {
    /// Maps each element of the iterator to whether it is `NaN`.
    fn map_is_nan( self ) -> Map< Self, fn( Self::Item ) -> bool >
    where
      Self : Sized;
  }

  impl<'item_ref, I, Item > IterFloat for I
  where
    I : Iterator< Item = &'item_ref Item > + IterExt,
    Item : Copy + Float + 'item_ref,
  {
    #[ inline ]
    fn map_is_nan( self ) -> Map< Self, fn( Self::Item ) -> bool >
    where
      Self : Sized,
    {
      self.map( | x : Self::Item | x.is_nan() )
    }
  }

}

crate::mod_interface!
{
  orphan use
  {
    IterExt,
    IterFloat,
  };
}
