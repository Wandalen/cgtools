/// Internal namespace.
mod private
{
  use crate::*;
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
    E : NdFloat,
  {
    a.vector_iter()
    .zip( b.vector_iter() )
    .map( | ( a_elem, b_elem ) | *a_elem * *b_elem )
    .fold( E::zero(), | sum, val | sum + val )
  }

  /// Computes the squared magnitude of a vector.
  #[ inline ]
  pub fn mag2< E, A, const SIZE : usize >( a : &A ) -> E
  where
    A : VectorIter< E, SIZE >,
    E : NdFloat,
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
  #[ inline ]
  pub fn normalize< E, R, A, const SIZE : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, SIZE >,
    A : VectorIter< E, SIZE >,
    E : NdFloat,
  {
    let mag = mag( a );
    for elem in r.vector_iter_mut()
    {
      *elem /= mag;
    }
  }

  /// Normalizes a vector to unit length.
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
  #[ inline ]
  pub fn angle< E, A, B, const SIZE : usize >( a : &A, b : &B ) -> E
  where
    A : VectorIter< E, SIZE >,
    B : VectorIter< E, SIZE >,
    E : NdFloat,
  {
    let cos_theta = dot( a, b ) / ( mag( a ) * mag( b ) );
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

  #[ inline ]
  pub fn cross_mut< E, R, B >( r : &mut R, b : &B )
  where
    R : VectorIterMut< E, 3 >,
    B : VectorIter< E, 3 >,
    E : NdFloat,
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

  #[ inline ]
  pub fn cross< E, A, B >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, 3 > + Clone,
    B : VectorIter< E, 3 >,
    E : NdFloat,
  {
    let mut r = a.clone();
    cross_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn sum_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r += *a;
    }
  }

  #[ inline ]
  pub fn sum< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : NdFloat,
  {
    let mut r = a.clone();
    sum_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn sum_scalar_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r += a;
    }
  }

  #[ inline ]
  pub fn sum_scalar< E, A, const N : usize >( a : &A, b : E ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    E : NdFloat,
  {
    let mut r = a.clone();
    sum_scalar_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn sub_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r -= *a;
    }
  }

  #[ inline ]
  pub fn sub< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : NdFloat,
  {
    let mut r = a.clone();
    sub_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn sub_scalar_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r -= a;
    }
  }

  #[ inline ]
  pub fn sub_scalar< E, A, const N : usize >( a : &A, b : E ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    E : NdFloat,
  {
    let mut r = a.clone();
    sub_scalar_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn mul_mut< E, R, A, const N : usize >( r : &mut R, a : A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r *= *a;
    }
  }

  #[ inline ]
  pub fn mul< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : NdFloat,
  {
    let mut r = a.clone();
    mul_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn mul_scalar_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r *= a;
    }
  }

  #[ inline ]
  pub fn mul_scalar< E, R, const N : usize >( a : &R, b : E ) -> R
  where
    R : VectorIterMut< E, N >  + Clone,
    E : NdFloat,
  {
    let mut r = a.clone();
    mul_scalar_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn div_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r /= *a;
    }
  }

  #[ inline ]
  pub fn div< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : NdFloat,
  {
    let mut r = a.clone();
    div_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn div_scalar_mut< E, R, const N : usize >( r : &mut R, a : E )
  where
    R : VectorIterMut< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut();
    for r in iter
    {
      *r /= a;
    }
  }

  #[ inline ]
  pub fn div_scalar< E, R, const N : usize >( a : &R, b : E ) -> R
  where
    R : VectorIterMut< E, N >  + Clone,
    E : NdFloat,
  {
    let mut r = a.clone();
    div_scalar_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn min_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r = ( *r ).min( *a );
    }
  }

  #[ inline ]
  pub fn min< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : NdFloat,
  {
    let mut r = a.clone();
    min_mut( &mut r, b );
    r
  }

  #[ inline ]
  pub fn max_mut< E, R, A, const N : usize >( r : &mut R, a : &A )
  where
    R : VectorIterMut< E, N >,
    A : VectorIter< E, N >,
    E : NdFloat,
  {
    let iter = r.vector_iter_mut().zip( a.vector_iter() );
    for ( r, a ) in iter
    {
      *r = ( *r ).max( *a );
    }
  }

  #[ inline ]
  pub fn max< E, A, B, const N : usize >( a : &A, b : &B ) -> A
  where
    A : VectorIterMut< E, N > + Clone,
    B : VectorIter< E, N >,
    E : NdFloat,
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
    sum,
    sum_mut,
    sub,
    sub_mut,
    mul,
    mul_mut,
    mul_scalar,
    mul_scalar_mut,
    div_scalar,
    div_scalar_mut,
    min,
    min_mut,
    max,
    max_mut,
    div,
    div_mut,
    sub_scalar,
    sub_scalar_mut,
    sum_scalar,
    sum_scalar_mut
  };
}
