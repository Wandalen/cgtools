/// Internal namespace.
mod private
{
  use crate::*;
  // use vector::arithmetics::inner_product::*;
  // use vector::arithmetics::{ normalized, mag };
  use vector::{ normalized, mag, mag2, min, max, dot };

  impl< E : MatNum, const LEN : usize > Vector< E, LEN >
  {

    /// Compute the squared length of the vector. Available for any numeric
    /// scalar (integers included) since it does not require `sqrt`. Note that
    /// for integer scalars the per-component squaring and summation are not
    /// overflow-checked: they panic in debug / wrap in release once the sum of
    /// squares exceeds `E::MAX`. Widen the element type or use a float scalar
    /// when that is possible.
    pub fn mag2( &self ) -> E
    {
      mag2( self )
    }

    /// Compute the dot product of two vectors.
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
    pub fn distance_squared( &self, rhs : &Self ) -> E
    {
      mag2( &( *self - *rhs ) )
    }
  }

  impl< E : MatEl + NdFloat, const LEN : usize > Vector< E, LEN >
  {

    /// Normalize the vector. Requires float scalar (uses `sqrt`).
    pub fn normalize( self ) -> Self
    {
      normalized( &self )
    }

    /// Compute the length of the vector. Requires float scalar (uses `sqrt`).
    pub fn mag( &self ) -> E
    {
      mag( self )
    }

    /// Compute a vector whose elements are the minimum of both vectors:
    /// `r[ i ] = a[ i ].min( b[ i ] )`. Currently float-only.
    pub fn min( self, rhs : Self ) -> Self
    {
      min( &self, &rhs )
    }

    /// Compute a vector whose elements are the maximum of both vectors:
    /// `r[ i ] = a[ i ].max( b[ i ] )`. Currently float-only.
    pub fn max( self, rhs : Self ) -> Self
    {
      max( &self, &rhs )
    }

    /// Compute length of the vector between two points in space. Requires
    /// float scalar (uses `sqrt`).
    pub fn distance( &self, rhs : &Self ) -> E
    {
      ( rhs - self ).mag()
    }
  }

}

crate::mod_interface!
{
  // xxx : reuse
  reuse ::mdmath_core::vector::arithmetics;
}
