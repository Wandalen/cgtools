/// Internal namespace.
mod private
{
  use crate::{vector, Vector, MatNum, MatEl, NdFloat};
  // use vector::arithmetics::inner_product::*;
  // use vector::arithmetics::{ normalized, mag };
  use vector::{ normalized, try_normalized, is_finite, mag, mag2, min, max, dot };

  impl< E : MatNum, const LEN : usize > Vector< E, LEN >
  {

    /// Compute the squared length of the vector. Available for any numeric
    /// scalar (integers included) since it does not require `sqrt`. Note that
    /// for integer scalars the per-component squaring and summation are not
    /// overflow-checked: they panic in debug / wrap in release once the sum of
    /// squares exceeds `E::MAX`. Widen the element type or use a float scalar
    /// when that is possible.
    #[ inline ]
    pub fn mag2( &self ) -> E
    {
      mag2( self )
    }

    /// Compute the dot product of two vectors. Note that for integer scalars
    /// the per-element products and their summation are not overflow-checked:
    /// they panic in debug / wrap in release once any intermediate value
    /// exceeds `E::MAX`. Widen the element type or use a float scalar when that
    /// is possible.
    #[ inline ]
    pub fn dot( &self, rhs : &Self ) -> E
    {
      dot( self, rhs )
    }
  }

  impl< E, const LEN : usize > Vector< E, LEN >
  where
    E : MatNum + ::num_traits::Signed,
  {
    /// Compute squared Euclidean distance between two points. Requires a
    /// signed scalar because the intermediate subtraction can produce negative
    /// values; use `saturating_sub` + `mag2` manually for unsigned types.
    ///
    /// # Overflow
    ///
    /// For integer scalars this is **not** overflow-checked. Two independent
    /// overflows are possible and will panic in debug / wrap in release:
    /// - the component subtraction `self - rhs`, when a coordinate difference
    ///   falls outside `E`'s range (e.g. `E::MAX - E::MIN`);
    /// - the squaring and summation inside `mag2`, even when the differences
    ///   themselves fit.
    ///
    /// `wrapping_sub` only addresses the first step — `mag2` still squares, so
    /// there is no fully overflow-safe integer form at the same width. For
    /// inputs whose squared distance can exceed `E::MAX`, widen first
    /// (e.g. `cast::<i64>()`) or use a floating-point scalar.
    #[ inline ]
    pub fn distance_squared( &self, rhs : &Self ) -> E
    {
      mag2( &( *self - *rhs ) )
    }
  }

  impl< E : MatEl + NdFloat, const LEN : usize > Vector< E, LEN >
  {

    /// Normalize the vector. Requires float scalar (uses `sqrt`).
    ///
    /// Divides by the magnitude unconditionally. For the zero vector that is a division by
    /// zero, so every component becomes `NaN` and is returned as an ordinary value — use
    /// [`Self::try_normalize`] where a zero-length input is reachable.
    #[ must_use ]
    #[ inline ]
    pub fn normalize( self ) -> Self
    {
      normalized( &self )
    }

    /// Normalize the vector, or `None` if it has no direction to normalize to.
    ///
    /// The guarded counterpart of [`Self::normalize`]. A zero-length vector has no unit
    /// direction, and dividing by its magnitude yields `NaN` components that propagate
    /// silently through everything downstream — a real case wherever a direction is derived
    /// from a difference that can vanish (two coincident points, a body exactly at its
    /// primary, a degenerate orbit).
    ///
    /// Returns `None` when the magnitude is zero or non-finite, so `Some` always carries a
    /// finite unit vector. The check is on the *magnitude* rather than the components, which
    /// additionally rejects two cases [`Self::is_finite`] accepts: components small enough
    /// that the sum of their squares underflows to zero, and components large enough that it
    /// overflows to infinity. The magnitude is what the division actually uses.
    ///
    /// ```
    /// use ndarray_cg::F64x3;
    ///
    /// assert!( F64x3::new( 3.0, 0.0, 4.0 ).try_normalize().is_some() );
    /// assert!( F64x3::ZERO.try_normalize().is_none() );
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn try_normalize( self ) -> Option< Self >
    {
      try_normalized( &self )
    }

    /// Whether every component is finite — neither infinite nor `NaN`.
    ///
    /// The component-wise counterpart of [`f64::is_finite`]. Worth having as one call because
    /// a vector is usually validated as a whole: one non-finite component makes the whole
    /// value meaningless, and checking each axis by hand is the step that gets skipped.
    ///
    /// Use this to validate incoming data. To guard a division by the magnitude, use
    /// [`Self::try_normalize`] instead — every component can be finite while the magnitude is
    /// not, so this check alone does not make normalization safe.
    ///
    /// ```
    /// use ndarray_cg::F64x3;
    ///
    /// assert!( F64x3::new( 1.0, 2.0, 3.0 ).is_finite() );
    /// assert!( !F64x3::new( 1.0, f64::NAN, 3.0 ).is_finite() );
    /// assert!( !F64x3::new( 1.0, f64::INFINITY, 3.0 ).is_finite() );
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn is_finite( &self ) -> bool
    {
      is_finite( self )
    }

    /// Compute the length of the vector. Requires float scalar (uses `sqrt`).
    #[ inline ]
    pub fn mag( &self ) -> E
    {
      mag( self )
    }

    /// Compute length of the vector between two points in space. Requires
    /// float scalar (uses `sqrt`).
    #[ inline ]
    pub fn distance( &self, rhs : &Self ) -> E
    {
      ( rhs - self ).mag()
    }
  }

  impl< E : MatNum + PartialOrd, const LEN : usize > Vector< E, LEN >
  {
    /// Compute a vector whose elements are the minimum of both vectors:
    /// `r[ i ] = a[ i ].min( b[ i ] )`. Satisfied by all integer primitives and
    /// floats alike — see `mdmath_core::vector::min_mut` for the float NaN
    /// tie-break behavior.
    #[ must_use ]
    #[ inline ]
    pub fn min( self, rhs : Self ) -> Self
    {
      min( &self, &rhs )
    }

    /// Compute a vector whose elements are the maximum of both vectors:
    /// `r[ i ] = a[ i ].max( b[ i ] )`. Satisfied by all integer primitives and
    /// floats alike — see `mdmath_core::vector::max_mut` for the float NaN
    /// tie-break behavior.
    #[ must_use ]
    #[ inline ]
    pub fn max( self, rhs : Self ) -> Self
    {
      max( &self, &rhs )
    }
  }

}

crate::mod_interface!
{
  reuse ::mdmath_core::vector::arithmetics;
}
