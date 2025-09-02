//! Contains traits for describing shapes of matrices

mod private
{
  // use crate::*;

  // =

  /// A trait indicate that matrix in case of referencing it can be interpreted as such having specified shape `ROWS x COLS`.
  ///
  /// This trait defines a constant `ROWS, COLS`, representing the length of the entity.
  pub trait MatWithShape< const ROWS : usize, const COLS : usize >
  {
  }

  /// Implementation of `MatWithShape` for references to entities.
  impl< T, const ROWS : usize, const COLS : usize > MatWithShape< ROWS, COLS > for &T
  where
    T : MatWithShape< ROWS, COLS >,
  {
  }

  /// Implementation of `MatWithShape` for mutable references to entities.
  impl< T, const ROWS : usize, const COLS : usize > MatWithShape< ROWS, COLS > for &mut T
  where
    T : MatWithShape< ROWS, COLS >,
  {
  }

  // =

  /// A trait indicate that matrix in case of mutable referencing it can be interpreted as such having specified shape `ROWS x COLS`.
  ///
  /// This trait defines a constant `ROWS, COLS`, representing the length of the entity.
  pub trait MatWithShapeMut< const ROWS : usize, const COLS : usize >
  where
    Self : MatWithShape< ROWS, COLS >,
  {
  }

  /// Implementation of `MatWithShapeMut` for references to entities.
  impl< T, const ROWS : usize, const COLS : usize > MatWithShapeMut< ROWS, COLS > for &T
  where
    Self : MatWithShape< ROWS, COLS > + MatWithShapeMut< ROWS, COLS > +,
  {
  }

  /// Implementation of `MatWithShapeMut` for mutable references to entities.
  impl< T, const ROWS : usize, const COLS : usize > MatWithShapeMut< ROWS, COLS > for &mut T
  where
    Self : MatWithShape< ROWS, COLS > + MatWithShapeMut< ROWS, COLS >,
  {
  }

  // =

}

crate::mod_interface!
{

  exposed use
  {
    MatWithShape,
    MatWithShapeMut,
  };

}
