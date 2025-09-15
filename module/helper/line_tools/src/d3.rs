mod private
{
  /// The vertex shader for the line rendering.
  pub const MAIN_VERTEX_SHADER : &'static str = include_str!( "./d3/shaders/main.vert" );

  /// The fragment shader for the line rendering.
  pub const MAIN_FRAGMENT_SHADER : &'static str = include_str!( "./d3/shaders/main.frag" );

  /// Return the types corresponding to the provided length
  #[ macro_export ]
  macro_rules! dim_to_vec 
  {
    ( f32, 2 ) => 
    {
      gl::F32x2
    };

    ( f32, 3 ) =>
    {
      gl::F32x3
    };

    ( f64, 2 ) => 
    {
      gl::F64x2
    };

    ( f64, 3 ) =>
    {
      gl::F64x3
    };
  }

  /// Creates a vector using the provided expression and the length of the vector
  #[ macro_export ]
  macro_rules! splat_vector 
  {
    ( $value:expr, f32, 2 ) => 
    {
      gl::F32x2::new( $value, $value )
    };

    ( $value:expr, f32, 3 ) => 
    {
      gl::F32x3::new( $value, $value, $value )
    };

    ( $value:expr, f64, 2 ) => 
    {
      gl::F64x2::new( $value, $value )
    };

    ( $value:expr, f64, 3 ) => 
    {
      gl::F64x3::new( $value, $value, $value )
    };
  }

  /// Implements the basic line functionality shared between all lines
  #[ macro_export ]
  macro_rules! impl_basic_line
  {
    ( $line_type:ty, $primitive_type:tt, $dimensions:tt ) =>
    {
      impl $line_type
      {
        /// Sets whether the vertex color attribute will be used or not
        pub fn use_vertex_color( &mut self, value : bool )
        {
          self.use_vertex_color = value;
          self.defines_changed = true;
        }

        /// Adds a new point to the back of the list.
        pub fn point_add_back< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : P )
        {
          let mut iter = point.vector_iter();
          let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

          self.points.push_back( point );
          self.points_changed = true;
        }

        /// Adds a new point to the front of the list.
        pub fn point_add_front< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : P )
        {
          let mut iter = point.vector_iter();
          let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

          self.points.push_front( point );
          self.points_changed = true;
        }

        /// Adds a new point to the back of the list.
        pub fn points_add_back< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, points : &[ P ] )
        {
          for i in 0..points.len()
          {
            let mut iter = points[ i ].vector_iter();
            let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

            self.points.push_back( point );
          }

          self.points_changed = true;
        }

        /// Adds a new point to the front of the list.
        pub fn points_add_front< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, points : &[ P ] )
        {
          for i in 0..points.len()
          {
            let mut iter = points[ i ].vector_iter();
            let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

            self.points.push_front( point );
          }

          self.points_changed = true;
        }

        /// Adds the color to a list of colors. Each color belongs to a point with the same index;
        pub fn color_add< C : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, color : C )
        {
          let mut iter = color.vector_iter();
          let color = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

          self.colors.push_back( color );
          self.colors_changed = true;
        }

        /// Retrieves the points at the specified position.
        /// Will panic if index is out of range
        pub fn point_get( &self, index : usize ) -> dim_to_vec!( $primitive_type, $dimensions )
        {
          self.points[ index ]
        }

        /// Sets the points at the specified position.
        /// Will panic if index is out of range
        pub fn point_set< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : P, index : usize )
        {
          let mut iter = point.vector_iter();
          let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );
          self.points[ index ] = point;
          self.points_changed = true;
        }

        /// Remove a point at the specified index
        pub fn point_remove( &mut self, index : usize )
        {
          self.points.remove( index );
          self.colors.remove( index );
          self.points_changed = true
        }

        /// Removes a points from the front
        pub fn point_remove_front( &mut self )
        {
          self.points.pop_front();
          self.colors.pop_front();
          self.points_changed = true
        }

        /// Remove a point from the back
        pub fn point_remove_back( &mut self )
        {
          self.points.pop_back();
          self.colors.pop_back();
          self.points_changed = true
        }

        /// Remove the specified amount of points from the front of the list
        pub fn points_remove_front( &mut self, amount : usize )
        {
          for _ in 0..amount
          {
            self.points.pop_front();
            self.colors.pop_front();
          }
          self.points_changed = true
        }

        /// Remove the specified amount of points from the back of the list
        pub fn points_remove_back( &mut self, amount : usize )
        {
          for _ in 0..amount
          {
            self.points.pop_back();
            self.colors.pop_back();
          }
          self.points_changed = true
        }

        /// Retrieves a reference to the mesh.
        pub fn mesh_get( &self ) -> Result< &Mesh, gl::WebglError >
        {
          self.mesh.as_ref().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
        }  

        /// Retrieves a mutable reference to the mesh.
        pub fn mesh_get_mut( &mut self ) -> Result< &mut Mesh, gl::WebglError >
        {
          self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
        }  

        /// Retrieves a slice of the line's points.
        pub fn points_get( &mut self ) -> &[ dim_to_vec!( $primitive_type, $dimensions ) ]
        {
          self.points.make_contiguous();
          self.points.as_slices().0
        }  

        /// Return the number of points that form this line
        pub fn num_points( &self ) -> usize
        {
          self.points.len()
        }
      }
    }
  }
}

crate::mod_interface!
{
  /// Layer for line-related functionalities.
  layer line;

  own use
  {
    MAIN_VERTEX_SHADER,
    MAIN_FRAGMENT_SHADER
  };
}