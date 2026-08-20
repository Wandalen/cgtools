/// Internal namespace.
mod private
{
  use crate::{ Collection, VectorIter, VectorIterMut, Mat3, mat, MatEl, nd, Ix2, Indexable, ScalarMut, RawSliceMut };
  use mdmath_core::vector::arithmetics::{ normalized, cross, dot };

  /// Trait for representing and manipulating rotations.
  ///
  /// This trait provides methods for creating and manipulating rotations, allowing
  /// for operations such as aligning vectors, rotating vectors, and inverting rotations.
  pub trait Rotation< const SIZE : usize >
  where
    Self : Collection,
  {
    /// The size of the vector space.
    const SIZE : usize = SIZE;

    /// Creates a rotation that aligns the `dir` vector with the forward direction,
    /// using `up` as the reference for the up direction.
    ///
    /// # Parameters
    /// - `dir`: The direction vector to align with the forward direction.
    /// - `up`: The reference up vector.
    ///
    /// # Returns
    /// - A rotation that aligns `dir` with the forward direction.
    // Fix(TASK-395): rebound `Dir`/`Up` (and, identically below, `A`/`B`/`V`) from
    // `VectorSpace<SIZE>` to `VectorIter<Scalar,SIZE>`, matching `mat3x3h::transformation`'s
    // established, actually-working `look_to_rh`/`look_at_rh` precedent.
    // Root cause: `VectorSpace` requires `Indexable`, and in this crate `Indexable` is
    // implemented ONLY by `Mat` -- no vector type (`Vector<E,N>`, `[E;N]`) implements it, so
    // `VectorSpace` could never be satisfied by any actual vector argument. The trait had zero
    // implementors and zero callers before this task, so the gap was never exercised.
    // Pitfall: a marker trait bundling several capabilities (`VectorSpace` = `Collection +
    // Indexable + VectorIter`) can look like the "obvious" bound for a vector-like parameter
    // while silently requiring a capability (`Indexable`) that no vector type in the crate
    // actually has -- check what concretely implements a bundling trait, not just its name.
    fn look_at< Dir, Up >( dir : &Dir, up : &Up ) -> Self
    where
      Dir : VectorIter< < Self as Collection >::Scalar, SIZE >,
      Up : VectorIter< < Self as Collection >::Scalar, SIZE >;

    /// Creates a rotation that aligns vector `a` with vector `b`.
    ///
    /// # Parameters
    /// - `a`: The initial vector.
    /// - `b`: The target vector.
    ///
    /// # Returns
    /// - A rotation that aligns `a` with `b`.
    fn between_vectors< A, B >( a : &A, b : &B ) -> Self
    where
      A : VectorIter< < Self as Collection >::Scalar, SIZE >,
      B : VectorIter< < Self as Collection >::Scalar, SIZE >;

    /// Rotates a vector by this rotation.
    ///
    /// # Parameters
    /// - `vec`: The vector to rotate.
    ///
    /// # Returns
    /// - The rotated vector.
    // Fix(TASK-395): widened the bound from `VectorSpace<SIZE>` to `VectorIter<Scalar,SIZE> +
    // VectorIterMut<Scalar,SIZE>` (see the `look_at` Fix(TASK-395) comment above for why
    // `VectorSpace` itself is unusable here).
    // Root cause: this method takes `&mut V` and is documented to rotate `vec` in place, but
    // `VectorSpace` only grants read-only iteration even where it IS satisfiable -- no
    // implementor could ever actually write a rotated result back into `vec`.
    // Pitfall: a `&mut` parameter in a signature is not itself proof the trait bound backing it
    // grants write access -- check the bound's own supertraits, not just the reference kind.
    fn vector_rotate< V >( &self, vec : &mut V )
    where
      V : VectorIter< < Self as Collection >::Scalar, SIZE > + VectorIterMut< < Self as Collection >::Scalar, SIZE >;

    /// Inverts this rotation.
    ///
    /// # Returns
    /// - The inverse of this rotation.
    #[ must_use ]
    fn invert( &self ) -> Self;
  }

  /// Reads the first 3 scalars of a `VectorIter` into an owned `[ E ; 3 ]`, so the rest of this
  /// module can do plain scalar arithmetic without fighting borrowed iteration.
  fn to_array3< V, E >( v : &V ) -> [ E; 3 ]
  where
    V : VectorIter< E, 3 >,
    E : MatEl,
  {
    let mut it = v.vector_iter();
    [ *it.next().unwrap(), *it.next().unwrap(), *it.next().unwrap() ]
  }

  impl< E, Descriptor > Rotation< 3 > for Mat3< E, Descriptor >
  where
    E : MatEl + nd::NdFloat + ::num_traits::Signed,
    Descriptor : mat::Descriptor,
    Self : Indexable< Index = Ix2 > + ScalarMut< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    fn look_at< Dir, Up >( dir : &Dir, up : &Up ) -> Self
    where
      Dir : VectorIter< E, 3 >,
      Up : VectorIter< E, 3 >,
    {
      let dir_arr = to_array3( dir );
      let up_arr = to_array3( up );

      // Same right-handed basis construction as `mat3x3h::transformation::look_to_rh`, minus
      // the translation column a bare rotation (no eye position) has no use for.
      let z = normalized::< E, _, 3 >( &dir_arr );
      let x = normalized::< E, _, 3 >( &cross::< E, _, _ >( &z, &up_arr ) );
      let y = cross::< E, _, _ >( &x, &z );

      Self::from_row_major
      ([
         x[ 0 ],  x[ 1 ],  x[ 2 ],
         y[ 0 ],  y[ 1 ],  y[ 2 ],
        -z[ 0 ], -z[ 1 ], -z[ 2 ],
      ])
    }

    fn between_vectors< A, B >( a : &A, b : &B ) -> Self
    where
      A : VectorIter< E, 3 >,
      B : VectorIter< E, 3 >,
    {
      let a_n = normalized::< E, _, 3 >( &to_array3( a ) );
      let b_n = normalized::< E, _, 3 >( &to_array3( b ) );

      let v = cross::< E, _, _ >( &a_n, &b_n );
      let c = dot::< E, _, _, 3 >( &a_n, &b_n );
      let one = E::one();
      let two = one + one;

      // `a_n`/`b_n` (numerically) antiparallel: `1 + c` -> 0, and the closed form below divides
      // by it. Fall back to an explicit 180-degree rotation about any axis perpendicular to
      // `a_n`, built via a helper axis chosen away from `a_n`'s dominant component so the cross
      // product used to derive it stays numerically well-conditioned.
      if ( c + one ).abs() < E::from( 1.0e-6 ).unwrap()
      {
        let helper = if a_n[ 0 ].abs() <= a_n[ 1 ].abs() && a_n[ 0 ].abs() <= a_n[ 2 ].abs()
        {
          [ one, E::zero(), E::zero() ]
        }
        else if a_n[ 1 ].abs() <= a_n[ 2 ].abs()
        {
          [ E::zero(), one, E::zero() ]
        }
        else
        {
          [ E::zero(), E::zero(), one ]
        };
        let axis = normalized::< E, _, 3 >( &cross::< E, _, _ >( &a_n, &helper ) );
        let ( x, y, z ) = ( axis[ 0 ], axis[ 1 ], axis[ 2 ] );

        // 180-degree Rodrigues rotation about `axis`: R = 2 * axis⊗axis - I.
        return Self::from_row_major
        ([
          two * x * x - one, two * x * y,       two * x * z,
          two * x * y,       two * y * y - one, two * y * z,
          two * x * z,       two * y * z,       two * z * z - one,
        ]);
      }

      // Moller & Hughes, "Efficiently Building a Matrix to Rotate One Vector to Another" (1999).
      let k = one / ( one + c );

      Self::from_row_major
      ([
        v[ 0 ] * v[ 0 ] * k + c,      v[ 1 ] * v[ 0 ] * k - v[ 2 ], v[ 2 ] * v[ 0 ] * k + v[ 1 ],
        v[ 0 ] * v[ 1 ] * k + v[ 2 ], v[ 1 ] * v[ 1 ] * k + c,      v[ 2 ] * v[ 1 ] * k - v[ 0 ],
        v[ 0 ] * v[ 2 ] * k - v[ 1 ], v[ 1 ] * v[ 2 ] * k + v[ 0 ], v[ 2 ] * v[ 2 ] * k + c,
      ])
    }

    fn vector_rotate< V >( &self, vec : &mut V )
    where
      V : VectorIter< E, 3 > + VectorIterMut< E, 3 >,
    {
      let v = to_array3( vec );

      let r =
      [
        [ *self.scalar_ref( Ix2( 0, 0 ) ), *self.scalar_ref( Ix2( 0, 1 ) ), *self.scalar_ref( Ix2( 0, 2 ) ) ],
        [ *self.scalar_ref( Ix2( 1, 0 ) ), *self.scalar_ref( Ix2( 1, 1 ) ), *self.scalar_ref( Ix2( 1, 2 ) ) ],
        [ *self.scalar_ref( Ix2( 2, 0 ) ), *self.scalar_ref( Ix2( 2, 1 ) ), *self.scalar_ref( Ix2( 2, 2 ) ) ],
      ];

      let rotated =
      [
        r[ 0 ][ 0 ] * v[ 0 ] + r[ 0 ][ 1 ] * v[ 1 ] + r[ 0 ][ 2 ] * v[ 2 ],
        r[ 1 ][ 0 ] * v[ 0 ] + r[ 1 ][ 1 ] * v[ 1 ] + r[ 1 ][ 2 ] * v[ 2 ],
        r[ 2 ][ 0 ] * v[ 0 ] + r[ 2 ][ 1 ] * v[ 1 ] + r[ 2 ][ 2 ] * v[ 2 ],
      ];

      let mut it = vec.vector_iter_mut();
      *it.next().unwrap() = rotated[ 0 ];
      *it.next().unwrap() = rotated[ 1 ];
      *it.next().unwrap() = rotated[ 2 ];
    }

    fn invert( &self ) -> Self
    {
      // A rotation matrix is orthogonal: its inverse is its transpose.
      let mut result = Self::default();
      for row in 0..3
      {
        for col in 0..3
        {
          *result.scalar_mut( Ix2( col, row ) ) = *self.scalar_ref( Ix2( row, col ) );
        }
      }
      result
    }
  }

  /// Creates a rotation that aligns vector `a` with vector `b` in place.
  ///
  /// # Parameters
  /// - `dst`: The destination where the rotation will be stored.
  /// - `a`: The initial vector.
  /// - `b`: The target vector.
  pub fn inplace_between_vectors< Dst, A, B, const SIZE : usize >( dst : &mut Dst, a : &A, b : &B )
  where
    Dst : Rotation< SIZE >,
    A : VectorIter< < Dst as Collection >::Scalar, SIZE >,
    B : VectorIter< < Dst as Collection >::Scalar, SIZE >,
  {
    *dst = Dst::between_vectors( a, b );
  }

  /// Creates a rotation that aligns the `dir` vector with the forward direction in place,
  /// using `up` as the reference for the up direction.
  ///
  /// # Parameters
  /// - `dst`: The destination where the rotation will be stored.
  /// - `dir`: The direction vector to align with the forward direction.
  /// - `up`: The reference up vector.
  pub fn inplace_look_at< Dst, Dir, Up, const SIZE : usize >( dst : &mut Dst, dir : &Dir, up : &Up )
  where
    Dst : Rotation< SIZE >,
    Dir : VectorIter< < Dst as Collection >::Scalar, SIZE >,
    Up : VectorIter< < Dst as Collection >::Scalar, SIZE >,
  {
    *dst = Dst::look_at( dir, up );
  }

}

crate::mod_interface!
{
  exposed use
  {
    Rotation,
    inplace_between_vectors,
    inplace_look_at,
  };
}
