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
  ///
  /// Implemented for the owning type itself (see `d2/mat.rs`) and for `&mut T` when
  /// `T : MatWithShapeMut`. A shared reference `&T` deliberately does NOT implement this trait,
  /// since it cannot provide the mutable access the trait promises (BUG-289).
  ///
  /// ```rust
  /// // `&mut T` must satisfy `MatWithShapeMut` and actually compile -- pre-BUG-289-fix this
  /// // overflowed trait resolution (E0275) instead, via a self-referential where-clause.
  /// fn requires_mut_shape< M : ndarray_cg::MatWithShapeMut< 2, 2 > >( _m : M ) {}
  /// let mut m = ndarray_cg::F32x2x2::default();
  /// requires_mut_shape( &mut m );
  /// ```
  ///
  /// ```compile_fail
  /// // a shared reference must NOT satisfy `MatWithShapeMut`.
  /// fn requires_mut_shape< M : ndarray_cg::MatWithShapeMut< 2, 2 > >( _m : M ) {}
  /// let m = ndarray_cg::F32x2x2::default();
  /// requires_mut_shape( &m );
  /// ```
  pub trait MatWithShapeMut< const ROWS : usize, const COLS : usize >
  where
    Self : MatWithShape< ROWS, COLS >,
  {
  }

  // Fix(BUG-289): `MatWithShapeMut` was blanket-implemented for both `&T` and `&mut T` with a
  // self-referential `where Self : MatWithShape<..> + MatWithShapeMut<..>` bound -- requiring
  // the very fact being proven as its own premise. This didn't just wrongly grant `&T` the
  // "supports mutable shape access" marker; it broke trait resolution for BOTH reference kinds,
  // including the legitimately-intended `&mut T` case, which overflowed (E0275) instead of
  // compiling whenever anything actually tried to use it as `M : MatWithShapeMut<..>`.
  // Root cause: the `&T` impl was copy-pasted from `&mut T` with only the impl target changed,
  // leaving the mutable-implying, circular where-clause on both -- unlike the correct sibling
  // `MatWithShape for &T`/`&mut T` pair above, which each bound `T : MatWithShape<..>` (the
  // referent, not `Self`).
  // Pitfall: a circular trait bound like `Self : SameTrait<..>` doesn't just silently do nothing
  // -- it can overflow trait resolution for every concrete type that would otherwise satisfy it,
  // masking the defect as "unused/dead code" (the only real consumers were commented out) rather
  // than an obvious compile error at the point of use.
  /// Implementation of `MatWithShapeMut` for mutable references to entities.
  impl< T, const ROWS : usize, const COLS : usize > MatWithShapeMut< ROWS, COLS > for &mut T
  where
    T : MatWithShapeMut< ROWS, COLS >,
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
