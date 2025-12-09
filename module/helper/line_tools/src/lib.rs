//! Line drawing and manipulation utilities for 2D and 3D graphics.
#![ doc( html_root_url = "https://docs.rs/line_tools/latest/line_tools/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Line drawing and manipulation utilities for 2D and 3D graphics" ) ]

mod private
{
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
        /// Clears the points from the line without releasing the memory
        pub fn clear( &mut self )
        {
          self.geometry.points.clear();
          self.geometry.colors.clear();

          #[ cfg( feature = "distance" ) ]
          {
            self.geometry.distances.clear();
            self.geometry.total_distance = 0.0;
          }

          self.change_state.points_changed = true;
          self.change_state.colors_changed = true;
        }

        /// Sets whether the vertex color attribute will be used or not
        pub fn use_vertex_color( &mut self, value : bool )
        {
          self.render_state.use_vertex_color = value;
          self.change_state.defines_changed = true;
        }

        /// Adds a new point to the back of the list.
        pub fn point_add_back< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : &P )
        {
          let mut iter = point.vector_iter();
          let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

          #[ cfg( feature = "distance" ) ]
          {
            let distance = if let Some( last ) = self.geometry.points.back().copied()
            {
              if ( last - point ).mag2() <= std::$primitive_type::EPSILON 
              {
                return;
              }

              ( point - last ).mag() 
            }
            else
            {
              0.0
            };

            self.geometry.total_distance += distance;
            self.geometry.distances.push_back( self.geometry.total_distance );
          }

          self.geometry.points.push_back( point );
          self.change_state.points_changed = true;
        }

        /// Adds a new point to the front of the list.
        pub fn point_add_front< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : &P )
        {
          let mut iter = point.vector_iter();
          let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

          let geometry = &mut self.geometry;

          #[ cfg( feature = "distance" ) ]
          {
            let distance = if let Some( last ) = geometry.points.front().copied()
            {
              if ( last - point ).mag2() <= std::$primitive_type::EPSILON 
              {
                return;
              }

              ( point - last ).mag() 
            }
            else
            {
              0.0
            };

            for d in geometry.distances.iter_mut()
            {
              *d += distance;
            }

            geometry.total_distance += distance;
            geometry.distances.push_front( 0.0 );
          }

          geometry.points.push_front( point );
          self.change_state.points_changed = true;
        }

        /// Adds a new point to the back of the list.
        pub fn points_add_back< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, points : &[ P ] )
        {
          for i in 0..points.len()
          {
            self.point_add_back( &points[ i ] );
          }

          self.change_state.points_changed = true;
        }

        /// Adds a new point to the front of the list.
        pub fn points_add_front< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, points : &[ P ] )
        {
          for i in 0..points.len()
          {
            self.point_add_front( &points[ i ] );
          }

          self.change_state.points_changed = true;
        }

        /// Adds the color to a list of colors. Each color belongs to a point with the same index;
        pub fn color_add_front< C : gl::VectorIter< $primitive_type, 3 > >( &mut self, color : C )
        {
          let mut iter = color.vector_iter();
          let color = splat_vector!( *iter.next().unwrap(), $primitive_type, 3 );

          self.geometry.colors.push_front( color );
          self.change_state.colors_changed = true;
        }

        /// Adds the color to a list of colors. Each color belongs to a point with the same index;
        pub fn color_add_back< C : gl::VectorIter< $primitive_type, 3 > >( &mut self, color : C )
        {
          let mut iter = color.vector_iter();
          let color = splat_vector!( *iter.next().unwrap(), $primitive_type, 3 );

          self.geometry.colors.push_back( color );
          self.change_state.colors_changed = true;
        }

        /// Retrieves the points at the specified position.
        /// Will panic if index is out of range
        pub fn point_get( &self, index : usize ) -> dim_to_vec!( $primitive_type, $dimensions )
        {
          self.geometry.points[ index ]
        }

        /// Sets the points at the specified position.
        /// Will panic if index is out of range
        pub fn point_set< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : P, index : usize )
        {
          if let Some( p ) = self.geometry.points.get_mut( index )
          {
            let mut iter = point.vector_iter();
            let point = splat_vector!( *iter.next().unwrap(), $primitive_type, $dimensions );

            *p = point;
            #[ cfg( feature = "distance" ) ]
            self.distances_update_from( index );

            self.change_state.points_changed = true;
          }
        }

        /// Sets the points at the specified position.
        /// Will panic if index is out of range
        pub fn color_set< C : gl::VectorIter< $primitive_type, 3 > >( &mut self, color : C, index : usize )
        {
          if let Some( c ) = self.geometry.colors.get_mut( index )
          {
            let mut iter = color.vector_iter();
            let color = splat_vector!( *iter.next().unwrap(), $primitive_type, 3 );

            *c = color;
            self.change_state.colors_changed = true;
          }
        }

        /// Removes a point at the specified index
        pub fn point_remove( &mut self, index : usize ) -> Option< dim_to_vec!( $primitive_type, $dimensions ) >
        {
          let point = self.geometry.points.remove( index );
          #[ cfg( feature = "distance" ) ]
          self.distances_update_from( index );
          self.change_state.points_changed = true;

          point
        }

        /// Removes a color an the color at the specified index
        pub fn color_remove( &mut self, index : usize ) -> Option< dim_to_vec!( $primitive_type, 3 ) >
        {
          let color = self.geometry.colors.remove( index );
          self.change_state.colors_changed = true;

          color
        }

        /// Removes a points from the front
        pub fn point_remove_front( &mut self ) -> Option< dim_to_vec!( $primitive_type, $dimensions ) >
        {
          let geometry = &mut self.geometry;
          #[ cfg( feature = "distance" ) ]
          {
            if geometry.distances.len() > 1
            {
              let delta_dist = geometry.distances[ 1 ];
              for d in geometry.distances.iter_mut().skip( 1 )
              {
                *d -= delta_dist;
              }
            }
            geometry.distances.pop_front();
          }
          let point = geometry.points.pop_front();
          self.change_state.points_changed = true;

          point
        }

        /// Removes a points from the front
        pub fn color_remove_front( &mut self ) -> Option< dim_to_vec!( $primitive_type, 3 ) >
        {
          let color = self.geometry.colors.pop_front();
          self.change_state.colors_changed = true;

          color
        }

        /// Remove a point from the back
        pub fn point_remove_back( &mut self ) -> Option< dim_to_vec!( $primitive_type, $dimensions ) >
        {
          let geometry = &mut self.geometry;
          #[ cfg( feature = "distance" ) ]
          {
            if geometry.distances.len() > 0
            {
              let delta_dist = geometry.distances.back().unwrap();
              geometry.total_distance -= delta_dist;
              geometry.distances.pop_back();
            }
          }

          let point = geometry.points.pop_back();
          self.change_state.points_changed = true;

          point
        }

        /// Removes a points from the front
        pub fn color_remove_back( &mut self ) -> Option< dim_to_vec!( $primitive_type, 3 ) >
        {
          let color = self.geometry.colors.pop_back();
          self.change_state.colors_changed = true;

          color
        }

        /// Remove the specified amount of points from the front of the list
        pub fn points_remove_front( &mut self, amount : usize )
        {
          for _ in 0..amount
          {
            self.point_remove_front();
          }
          self.change_state.points_changed = true
        }

        /// Remove the specified amount of colors from the front of the list
        pub fn colors_remove_front( &mut self, amount : usize )
        {
          for _ in 0..amount
          {
            self.color_remove_front();
          }
          self.change_state.colors_changed = true
        }

        /// Remove the specified amount of points from the back of the list
        pub fn points_remove_back( &mut self, amount : usize )
        {
          for _ in 0..amount
          {
            self.point_remove_back();
          }
          self.change_state.points_changed = true
        }

        /// Remove the specified amount of colors from the back of the list
        pub fn colors_remove_back( &mut self, amount : usize )
        {
          for _ in 0..amount
          {
            self.color_remove_back();
          }
          self.change_state.colors_changed = true
        }

        /// Retrieves a reference to the mesh.
        pub fn mesh_get( &self ) -> Result< &Mesh, gl::WebglError >
        {
          self.render_state.mesh.as_ref().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
        }  

        /// Retrieves a mutable reference to the mesh.
        pub fn mesh_get_mut( &mut self ) -> Result< &mut Mesh, gl::WebglError >
        {
          self.render_state.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
        }  

        /// Retrieves a slice of the line's points.
        pub fn points_get( &mut self ) -> &[ dim_to_vec!( $primitive_type, $dimensions ) ]
        {
          self.geometry.points.make_contiguous();
          self.geometry.points.as_slices().0
        }

        /// Retrieves a slice of the colors.
        pub fn colors_get( &mut self ) -> &[ dim_to_vec!( $primitive_type, 3 ) ]
        {
          self.geometry.colors.make_contiguous();
          self.geometry.colors.as_slices().0
        }

        #[ cfg( feature = "distance" ) ]
        /// Retrieves a slice of the distances.
        pub fn distances_get( &mut self ) -> &[ $primitive_type ]
        {
          self.geometry.distances.make_contiguous();
          self.geometry.distances.as_slices().0
        }

        /// Return the number of points that form this line
        pub fn num_points( &self ) -> usize
        {
          self.geometry.points.len()
        }

        #[ cfg( feature = "distance" ) ]
        /// Return the total lenth of the line
        pub fn total_distance_get( &self ) -> f32
        {
          self.geometry.total_distance
        }

        #[ cfg( feature = "distance" ) ]
        /// Recalculates the distance value for all points
        pub fn distances_update( &mut self )
        {
          self.geometry.total_distance = 0.0;
          self.geometry.distances.clear();
          self.geometry.distances.push_back( 0.0 );
          for ( i, p ) in self.geometry.points.iter().skip( 1 ).enumerate()
          {
            let dist = ( *p - *self.geometry.points.get( i ).unwrap() ).mag();
            self.geometry.total_distance += dist;
            self.geometry.distances.push_back( self.geometry.total_distance );
          } 
        }

        #[ cfg( feature = "distance" ) ]
        fn distances_update_from( &mut self, index : usize )
        {
          let geometry = &mut self.geometry;
          if index > 0
          {
            if let ( Some( prev_point ), Some( set_point ) ) = ( geometry.points.get( index - 1 ), geometry.points.get( index ) )
            {
              let delta_dist = ( set_point - prev_point ).mag();
              geometry.distances[ index ] = geometry.distances[ index - 1 ] + delta_dist;
            }

          }
          else
          {
            geometry.total_distance = 0.0;
            geometry.distances[ 0 ] = 0.0;
          }

          for i in ( index + 1 )..geometry.points.len()
          {
            let delta_dist = ( geometry.points[ i - 1 ] - geometry.points[ i ] ).mag();
            geometry.distances[ i ] = geometry.distances[ i - 1 ] + delta_dist;
          }

          geometry.total_distance = *geometry.distances.back().unwrap();
        }
      }
    }
  }
}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// A layer for 2D graphics-related functionalities.
  layer d2;
  /// A layer for 3D graphics-related functionalities.
  layer d3;

  /// A layer dedicated to line join styles (e.g., miter, bevel, round).
  layer joins;
  /// A layer dedicated to line cap styles (e.g., butt, round, square).
  layer caps;

  /// A layer for mesh generation and manipulation.
  layer mesh;
  /// A layer for shader programs and related functionality.
  layer program;

  /// A layer for helper functions and utilities used by other modules.
  layer helpers;
  
  /// Module for handling uniform operations
  layer uniform;
}