/// Internal namespace.
mod private
{
  use crate::{ NdFloat, Scalar, VectorIter, VectorIterMut, vector };
  use crate::approx::ulps_eq;

  /// Computes the dot product of two vectors.
  ///
  /// This function calculates the dot product of two vectors by iterating over their elements,
  /// multiplying corresponding elements, and summing the results. The vectors must implement
  /// the `VectorIter` trait to provide an iterator over their elements.
  ///
  /// # Type Parameters
  /// - `E`: The scalar type of the vector elements, which must implement `NdFloat`.
  /// - `A`: The type of the first vector, which must implement `VectorIter<E, SIZE>`.
  /// - `B`: The type of the second vector, which must implement `VectorIter<E, SIZE>`.
  /// - `SIZE`: The size of the vectors.
  ///
  /// # Parameters
  /// - `a`: A reference to the first vector.
  /// - `b`: A reference to the second vector.
  ///
  /// # Returns
  /// - The dot product of the two vectors as a scalar of type `E`.
  ///
  /// # Overflow
  /// For integer scalars the per-element products and their summation are not
  /// overflow-checked: they panic in debug / wrap in release once any
  /// intermediate value exceeds `E::MAX`. Widen the element type or use a float
  /// scalar when that is possible.
  ///
  /// # Example
  /// ```rust
  /// use mdmath_core::vector;
  /// let vec_a = [ 1.0, 2.0, 3.0 ];
  /// let vec_b = [ 4.0, 5.0, 6.0 ];
  /// let r = vector::dot( &vec_a, &vec_b );
  /// assert_eq!( r, 32.0 );
  /// ```
  #[ inline ]
  pub fn dot< E, A, B, const SIZE : usize >( a : &A, b : &B ) -> E
  where
    A : VectorIter< E, SIZE >,
    B : VectorIter< E, SIZE >,
    E : Scalar,
  {
    a.vector_iter()
    .zip( b.vector_iter() )
    .map( | ( a_elem, b_elem ) | *a_elem * *b_elem )
    .fold( E::zero(), | sum, val | sum + val )
  }

  /// Computes the squared magnitude of a vector (the dot product with itself).
  ///
  /// # Overflow
  /// For integer scalars the per-element products and their summation are not
  /// overflow-checked: they panic in debug / wrap in release once any
  /// intermediate value exceeds `E::MAX`. Widen the element type or use a float
  /// scalar when that is possible.
  #[ inline ]
  pub fn mag2< E, A, const SIZE : usize >( a : &A ) -> E
  where
    A : VectorIter< E, SIZE >,
    E : Scalar,
  {
    dot( a, a )
  }

  /// Computes the magnitude of a vector.
  #[ inline ]
  pub fn mag< E, A, const SIZE : usize >( a : &A ) -> E
  where
    A : VectorIter< E, SIZE >,
    E : NdFloat,
  {
    mag2( a ).sqrt()
  }

  /// Normalizes a vector to unit length.
  ///
  /// # Panics
  /// Panics if `a`'s iterator yields fewer than `SIZE` elements.
  ///
  /// # Zero-magnitude input
  /// If `a` has zero magnitude (e.g. the zero vector), every written component is `0.0 / 0.0`,
  /// i.e. `NaN` -- this is intentional, not an oversight: a zero-length vector has no defined
  /// direction, so `NaN` is the honest IEEE-754 encoding of "undefined" rather than an arbitrary
  /// fallback (e.g. silently returning the zero vector, which would falsely claim the zero
  /// vector's direction *is* the zero vector). Callers that need a defined fallback for
  /// degenerate input must check magnitude before calling. See BUG-448.
  // Fix(BUG-124): the write loop now reads `a`'s own elements (`*aiter.next().unwrap() / mag`)
  // instead of dividing whatever `r` already held.
  // Root cause: the loop only ever touched `r.vector_iter_mut()`, never `a`'s iterator beyond
  // the single aggregate `mag(a)` call — so this computed `r / |a|`, not `a / |a|`, silently
  // correct only when the caller had pre-set `r` equal to `a` (as the sole in-crate caller
  // `normalized()` does via `r = a.clone()`), despite `R`/`A` being independent, unconstrained
  // generic parameters with no `r == a` precondition documented anywhere in the signature.
  // Pitfall: when a "write into `r`, derived from `a`" function's loop body only reads `r`,
  // check whether it was ever meant to read `a` too — the sibling `project_on(r,b)` a few
  // lines below shows the correct pattern (`*elem = *biter.next().unwrap() * scalar`); a
  // same-crate sibling function is often the cheapest oracle for "should this dereference the
  // *other* argument."
  #[ inline ]
  pub fn normalize< E, R, A, const SIZE : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, SIZE >,
    A : VectorIter< E, SIZE >,
    E : NdFloat,
  {
    let mag = mag( a );
    let mut aiter = a.vector_iter();
    for elem in r.vector_iter_mut()
    {
      *elem = *aiter.next().unwrap() / mag;
    }
  }

  /// Normalizes a vector to unit length.
  ///
  /// # Zero-magnitude input
  /// Returns a vector of all `NaN` components if `a` has zero magnitude -- see
  /// [`normalize`]'s "Zero-magnitude input" doc note (BUG-448).
  #[ inline ]
  pub fn normalized< E, A, const SIZE : usize >( a : &A ) -> A
  where
    A : VectorIter< E, SIZE > + VectorIterMut< E, SIZE > + Clone,
    E : NdFloat,
  {
    let mut r : A = a.clone();
    normalize( &mut r, a );
    r
  }

  /// Normalizes a vector to a specified magnitude.
  ///
  /// # Zero-magnitude input
  /// Writes `NaN` to every component if `r` has zero magnitude -- see [`normalize`]'s
  /// "Zero-magnitude input" doc note (BUG-448).
  #[ inline ]
  pub fn normalize_to< E, R, const SIZE : usize >( r : &mut R, mag : E )
  where
    R : VectorIterMut< E, SIZE >,
    E : NdFloat,
  {
    let amag = vector::mag( r );
    for elem in r.vector_iter_mut()
    {
      *elem *= mag / amag;
    }
  }

  /// Normalizes a vector to a specified magnitude.
  ///
  /// # Zero-magnitude input
  /// Returns a vector of all `NaN` components if `a` has zero magnitude -- see
  /// [`normalize`]'s "Zero-magnitude input" doc note (BUG-448).
  #[ inline ]
  pub fn normalized_to< E, A, const SIZE : usize >( a : &A, mag : E ) -> A
  where
    A : VectorIterMut< E, SIZE > + Clone,
    E : NdFloat,
  {
    let mut r : A = a.clone();
    normalize_to( &mut r, mag );
    r
  }

  /// Projects vector `a` onto vector `b`.
  ///
  /// # Panics
  /// Panics if `r` or `b`'s iterator yields fewer than `SIZE` elements.
  ///
  /// # Zero-magnitude `b`
  /// If `b` has zero magnitude, every written component is `NaN` (`scalar = dot(r,b) / mag2(b)`
  /// is `0.0 / 0.0`) -- this is intentional: projection onto a degenerate (zero-length) axis is
  /// mathematically undefined, so `NaN` is the honest result rather than an arbitrary fallback
  /// (e.g. silently returning the zero vector). Callers that need a defined fallback for
  /// degenerate `b` must check its magnitude before calling. See BUG-448.
  #[ inline ]
  pub fn project_on< E, R, B, const SIZE : usize >( r : &mut R, b : &B )
  where
    R : VectorIterMut< E, SIZE >,
    B : VectorIter< E, SIZE >,
    E : NdFloat,
  {
    let scalar = dot( r, b ) / mag2( b );
    // let mut r = *b;
    let mut biter = b.vector_iter();
    for elem in r.vector_iter_mut()
    {
      *elem = *biter.next().unwrap() * scalar;
    }
  }

  /// Projects vector `a` onto vector `b`.
  ///
  /// # Zero-magnitude `b`
  /// Returns a vector of all `NaN` components if `b` has zero magnitude -- see [`project_on`]'s
  /// "Zero-magnitude `b`" doc note (BUG-448).
  #[ inline ]
  pub fn projected_on< E, A, B, const SIZE : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, SIZE > + Clone,
    B : VectorIter< E, SIZE >,
    E : NdFloat,
  {
    let mut r : A = a.clone();
    project_on( &mut r, b );
    r
  }

  /// Computes the angle between two vectors.
  // Fix(BUG-446): clamp `cos_theta` to `[ -1, 1 ]` before calling `.acos()`.
  // Root cause: `dot(a,b) / (mag(a)*mag(b))` is the mathematically-correct cosine formula, but
  // ordinary floating-point rounding in the dot-product and magnitude computations routinely
  // pushes the result marginally outside `[ -1, 1 ]` for near-identical/parallel vectors (e.g.
  // `angle(&v,&v)` for almost any nontrivial `v`) -- `.acos()` of an out-of-domain input silently
  // returns `NaN` for any such input, not just contrived edge cases.
  // Pitfall: any `.acos()`/`.asin()` call fed a value derived from a `dot`/magnitude ratio needs
  // an explicit clamp to its `[ -1, 1 ]` domain -- the ratio is only guaranteed to be in range
  // algebraically, not in finite-precision floating point; see the identical pattern already
  // fixed at BUG-272 (`Quat::to_euler_xyz`'s `asin` argument).
  #[ inline ]
  pub fn angle< E, A, B, const SIZE : usize >( a : &A, b : &B ) -> E
  where
    A : VectorIter< E, SIZE >,
    B : VectorIter< E, SIZE >,
    E : NdFloat,
  {
    let cos_theta = dot( a, b ) / ( mag( a ) * mag( b ) );
    // Fix(BUG-446): use `clamp` (NaN-preserving), not a `max`/`min` chain (NaN-clearing).
    // Root cause: `mag(a)*mag(b)` rounds, so a mathematically in-range ratio can land
    // fractionally outside `[-1,1]` -- but `dot/( mag(a)*mag(b) )` is also genuinely `NaN`
    // when either vector has zero magnitude (`0.0/0.0`), which is a real, pre-existing,
    // tested contract (`test_angle`'s zero-vector case expects `NaN`, not a fabricated
    // angle). `f32::max`/`f32::min` follow IEEE `maxNum`/`minNum` semantics: "if one operand
    // is NaN, return the other" -- so `NaN.max(-1.0).min(1.0)` silently produces `-1.0`
    // (`.acos()` -> `PI`), laundering an undefined zero-vector angle into a bogus finite
    // one. `clamp` instead returns `self` unchanged in its `else` branch whenever
    // `self < min` and `self > max` are both false -- true for any `self` compared against
    // NaN operands, so a NaN `self` passes through unclamped while genuinely out-of-range
    // finite values are still rescued.
    // Pitfall: `x.max(lo).min(hi)` and `x.clamp(lo,hi)` are NOT interchangeable when `x` may
    // be NaN -- `max`/`min` silently discard NaN, `clamp` preserves it. Prefer `clamp` for
    // any defensive pre-`acos`/`asin`/`sqrt` rescue where the input could legitimately be
    // NaN from an upstream 0/0 or negative-sqrt case that must stay NaN.
    let cos_theta = cos_theta.clamp( -E::one(), E::one() );
    cos_theta.acos()
  }

  /// Returns `true` if the vector `a` is perpendicular (orthogonal) to vector `b`.
  #[ inline ]
  pub fn is_orthogonal< E, A, B, const SIZE : usize >( a : &A, b : &B ) -> bool
  where
    A : VectorIter< E, SIZE >,
    B : VectorIter< E, SIZE >,
    E : NdFloat + approx::UlpsEq,
  {
    ulps_eq!( dot( a, b ), &E::zero() )
  }

  /// Computes the cross product of two 3D vectors.
  /// This function modifies the first vector in place.
  ///
  /// # Overflow
  /// For integer scalars the per-component multiplications and subtractions are
  /// not overflow-checked: they panic in debug / wrap in release once any
  /// intermediate value leaves `E`'s range.
  ///
  /// # Panics
  /// Panics if `r` or `b`'s iterator yields fewer than 3 elements.
  #[ inline ]
  pub fn cross_mut< E, R, B >( r : &mut R, b : &B )
  where
    R : VectorIterMut< E, 3 >,
    B : VectorIter< E, 3 >,
    E : Scalar + ::num_traits::Signed,
  {
    let u =
    {
      let mut iter = r.vector_iter();
      let x = *iter.next().unwrap();
      let y = *iter.next().unwrap();
      let z = *iter.next().unwrap();
      [ x, y, z ]
    };

    let v =
    {
      let mut iter = b.vector_iter();
      let x = *iter.next().unwrap();
      let y = *iter.next().unwrap();
      let z = *iter.next().unwrap();
      [ x, y, z ]
    };

    let x = u[ 1 ] * v[ 2 ] - u[ 2 ] * v[ 1 ];
    let y = u[ 2 ] * v[ 0 ] - u[ 0 ] * v[ 2 ];
    let z = u[ 0 ] * v[ 1 ] - u[ 1 ] * v[ 0 ];

    let mut iter = r.vector_iter_mut();
    *iter.next().unwrap() = x;
    *iter.next().unwrap() = y;
    *iter.next().unwrap() = z;
  }

  /// Computes the cross product of two 3D vectors.
  ///
  /// # Overflow
  /// For integer scalars the per-component multiplications and subtractions are
  /// not overflow-checked: they panic in debug / wrap in release once any
  /// intermediate value leaves `E`'s range.
  #[ inline ]
  pub fn cross< E, A, B >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, 3 > + Clone,
    B : VectorIter< E, 3 >,
    E : Scalar + ::num_traits::Signed,
  {
    let mut r = a.clone();
    cross_mut( &mut r, b );
    r
  }

  /// Returns a unit vector along whichever world axis (X, Y, or Z) is furthest from `v`'s own
  /// dominant direction, so it is guaranteed not to be (numerically) parallel to `v`.
  ///
  /// Used to build a well-defined fallback perpendicular basis when the natural reference
  /// vector for that basis is itself degenerate (parallel or antiparallel to `v`) -- e.g.
  /// picking a fallback "up" hint for a camera basis whose real `up` is parallel to its view
  /// direction, or a fallback rotation axis when aligning two antiparallel vectors. See
  /// BUG-445.
  #[ inline ]
  pub fn non_parallel_hint< E >( v : &[ E; 3 ] ) -> [ E; 3 ]
  where
    E : NdFloat,
  {
    let one = E::one();
    let zero = E::zero();
    if v[ 0 ].abs() <= v[ 1 ].abs() && v[ 0 ].abs() <= v[ 2 ].abs()
    {
      [ one, zero, zero ]
    }
    else if v[ 1 ].abs() <= v[ 2 ].abs()
    {
      [ zero, one, zero ]
    }
    else
    {
      [ zero, zero, one ]
    }
  }

  /// Performs element-wise addition operation on vectors.
  /// Modifies first vector in place.
  ///
  /// # Overflow
  /// For integer scalars the per-element addition is not overflow-checked: it
  /// panics in debug / wraps in release once a sum leaves `E`'s range.
  #[ inline ]
  pub fn sum_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r += *a;
    }
  }

  /// Performs element-wise addition operation on vectors.
  ///
  /// # Overflow
  /// For integer scalars the per-element addition is not overflow-checked: it
  /// panics in debug / wraps in release once a sum leaves `E`'s range.
  #[ inline ]
  pub fn sum< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : Scalar,
  {
    let mut r = a.clone();
    sum_mut( &mut r, b );
    r
  }

  /// Performs element-wise addition operation on vector with a scalar.
  /// Modifies first vector in place.
  ///
  /// # Overflow
  /// For integer scalars the per-element addition is not overflow-checked: it
  /// panics in debug / wraps in release once a sum leaves `E`'s range.
  #[ inline ]
  pub fn scalar_sum_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r += a;
    }
  }

  /// Performs element-wise addition operation on vector with a scalar.
  ///
  /// # Overflow
  /// For integer scalars the per-element addition is not overflow-checked: it
  /// panics in debug / wraps in release once a sum leaves `E`'s range.
  #[ inline ]
  pub fn scalar_sum< E, A, const N : usize >( a : &A, b : E ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    E : Scalar,
  {
    let mut r = a.clone();
    scalar_sum_mut( &mut r, b );
    r
  }

  /// Performs element-wise subtraction operation of vectors.
  /// Modifies first vector in place.
  ///
  /// # Overflow
  /// For integer scalars the per-element subtraction is not overflow-checked: it
  /// panics in debug / wraps in release whenever a result leaves `E`'s range —
  /// e.g. unsigned underflow when a component of `a` exceeds the matching
  /// component of `r`.
  #[ inline ]
  pub fn sub_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r -= *a;
    }
  }

  /// Performs element-wise subtraction operation of vectors.
  ///
  /// # Overflow
  /// For integer scalars the per-element subtraction is not overflow-checked: it
  /// panics in debug / wraps in release whenever a result leaves `E`'s range —
  /// e.g. unsigned underflow when a component of `b` exceeds the matching
  /// component of `a`.
  #[ inline ]
  pub fn sub< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : Scalar,
  {
    let mut r = a.clone();
    sub_mut( &mut r, b );
    r
  }

  /// Performs element-wise subtraction operation of vector with a scalar.
  /// Modifies first vector in place.
  ///
  /// # Overflow
  /// For integer scalars the per-element subtraction is not overflow-checked: it
  /// panics in debug / wraps in release whenever a result leaves `E`'s range —
  /// e.g. unsigned underflow when `a` exceeds a component of `r`.
  #[ inline ]
  pub fn scalar_sub_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r -= a;
    }
  }

  /// Performs element-wise subtraction operation of vector with a scalar.
  ///
  /// # Overflow
  /// For integer scalars the per-element subtraction is not overflow-checked: it
  /// panics in debug / wraps in release whenever a result leaves `E`'s range —
  /// e.g. unsigned underflow when `b` exceeds a component of `a`.
  #[ inline ]
  pub fn scalar_sub< E, A, const N : usize >( a : &A, b : E ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    E : Scalar,
  {
    let mut r = a.clone();
    scalar_sub_mut( &mut r, b );
    r
  }

  /// Performs element-wise multiplication operation on vectors.
  /// Modifies first vector in place.
  ///
  /// # Overflow
  /// For integer scalars the per-element multiplication is not overflow-checked:
  /// it panics in debug / wraps in release once a product leaves `E`'s range.
  #[ inline ]
  pub fn mul_mut< E, R, A, const N : usize >( r : &mut R, a : A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r *= *a;
    }
  }

  /// Performs element-wise multiplication operation on vectors.
  ///
  /// # Overflow
  /// For integer scalars the per-element multiplication is not overflow-checked:
  /// it panics in debug / wraps in release once a product leaves `E`'s range.
  #[ inline ]
  pub fn mul< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : Scalar,
  {
    let mut r = a.clone();
    mul_mut( &mut r, b );
    r
  }

  /// Performs element-wise multiplication operation on vector with a scalar.
  /// Modifies first vector in place.
  ///
  /// # Overflow
  /// For integer scalars the per-element multiplication is not overflow-checked:
  /// it panics in debug / wraps in release once a product leaves `E`'s range.
  #[ inline ]
  pub fn scalar_mul_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r *= a;
    }
  }

  /// Performs element-wise multiplication operation on vector with a scalar.
  ///
  /// # Overflow
  /// For integer scalars the per-element multiplication is not overflow-checked:
  /// it panics in debug / wraps in release once a product leaves `E`'s range.
  #[ inline ]
  pub fn scalar_mul< E, R, const N : usize >( a : &R, b : E ) -> R
  where
    R : VectorIterMut< E, N >  + Clone,
    E : Scalar,
  {
    let mut r = a.clone();
    scalar_mul_mut( &mut r, b );
    r
  }

  /// Performs element-wise division operation of vectors.
  /// Modifies first vector in place.
  ///
  /// # Panics
  /// For integer `E` this panics if any component of `a` is zero, in both debug
  /// and release mode. For float `E`, division by zero yields `INFINITY` or
  /// `NAN` instead.
  #[ inline ]
  pub fn div_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r /= *a;
    }
  }

  /// Performs element-wise division operation of vectors.
  ///
  /// # Panics
  /// For integer `E` this panics if any component of `b` is zero, in both debug
  /// and release mode. For float `E`, division by zero yields `INFINITY` or
  /// `NAN` instead.
  #[ inline ]
  pub fn div< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : Scalar,
  {
    let mut r = a.clone();
    div_mut( &mut r, b );
    r
  }

  /// Performs element-wise division operation of vector with a scalar.
  /// Modifies first vector in place.
  ///
  /// # Panics
  /// For integer `E` this panics if `a` is zero, in both debug and release
  /// mode. For float `E`, division by zero yields `INFINITY` or `NAN` instead.
  #[ inline ]
  pub fn scalar_div_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : Scalar,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r /= a;
    }
  }

  /// Performs element-wise division operation of vector with a scalar.
  ///
  /// # Panics
  /// For integer `E` this panics if `b` is zero, in both debug and release
  /// mode. For float `E`, division by zero yields `INFINITY` or `NAN` instead.
  #[ inline ]
  pub fn scalar_div< E, R, const N : usize >( a : &R, b : E ) -> R
  where
    R : VectorIterMut< E, N >  + Clone,
    E : Scalar,
  {
    let mut r = a.clone();
    scalar_div_mut( &mut r, b );
    r
  }

  /// Performs element-wise minimum operation on vectors.
  /// Modifies first vector in place.
  ///
  /// Satisfied by all integer primitives and floats alike (`E : Scalar + PartialOrd`) — this is
  /// pure ordering comparison, not floating-point arithmetic. NaN tie-break: if either operand is
  /// unordered with respect to the other (i.e. either is NaN), `r`'s original value is kept — `a`'s
  /// value is only ever selected when `*a < *r` is a well-defined `true`.
  #[ inline ]
  pub fn min_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : Scalar + PartialOrd,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r = if *a < *r { *a } else { *r };
    }
  }

  /// Performs element-wise minimum operation on vectors.
  ///
  /// Satisfied by all integer primitives and floats alike — see [`min_mut`] for the NaN tie-break
  /// behavior.
  #[ inline ]
  pub fn min< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : Scalar + PartialOrd,
  {
    let mut r = a.clone();
    min_mut( &mut r, b );
    r
  }

  /// Performs element-wise maximum operation on vectors.
  /// Modifies first vector in place.
  ///
  /// Satisfied by all integer primitives and floats alike (`E : Scalar + PartialOrd`) — this is
  /// pure ordering comparison, not floating-point arithmetic. NaN tie-break: if either operand is
  /// unordered with respect to the other (i.e. either is NaN), `r`'s original value is kept — `a`'s
  /// value is only ever selected when `*a > *r` is a well-defined `true`.
  #[ inline ]
  pub fn max_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : Scalar + PartialOrd,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r = if *a > *r { *a } else { *r };
    }
  }

  /// Performs element-wise maximum operation on vectors.
  ///
  /// Satisfied by all integer primitives and floats alike — see [`max_mut`] for the NaN tie-break
  /// behavior.
  #[ inline ]
  pub fn max< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : Scalar + PartialOrd,
  {
    let mut r = a.clone();
    max_mut( &mut r, b );
    r
  }
}

crate::mod_interface!
{
  orphan use
  {
    dot,
    mag2,
    mag,
    normalize,
    normalized,
    normalize_to,
    normalized_to,
    project_on,
    projected_on,
    angle,
    is_orthogonal,
    cross_mut,
    cross,
    non_parallel_hint,
    sum,
    sum_mut,
    sub,
    sub_mut,
    mul,
    mul_mut,
    scalar_mul,
    scalar_mul_mut,
    scalar_div,
    scalar_div_mut,
    min,
    min_mut,
    max,
    max_mut,
    div,
    div_mut,
    scalar_sub,
    scalar_sub_mut,
    scalar_sum,
    scalar_sum_mut
  };
}
