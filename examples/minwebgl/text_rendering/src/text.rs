
pub mod ufo
{
  use std::{collections::HashMap, str::FromStr};
  use kurbo::flatten;
  use mingl::IntoArray;
use norad::{ PointType, ContourPoint, Contour };
  use std::rc::Rc;
  use std::cell::RefCell;
  use minwebgl as gl;
  use gl::{ F32x3, math::vector::cross };
  use quick_xml::{ Reader, events::Event };
  use i_float::int::point::IntPoint;
  use i_triangle::int::triangulatable::IntTriangulatable;
  use crate::
  { 
    AttributesData, PrimitiveData, Transform 
  };

  #[ derive( Clone ) ]
  struct Glyph
  {
    _character : char,
    contours : Vec< Vec< [ f64; 2 ] > >,
    bounding_box : [ [ f64; 2 ]; 2 ]
  }

  impl Glyph
  {
    fn new( mut contours : Vec< Vec< [ f64; 2 ] > >, character : char ) -> Self
    {
      let mut bounding_box = [ [ f64::MAX; 2 ], [ f64::MIN; 2 ] ];

      for contour in &contours
      {
        for point in contour
        {
          if *point < bounding_box[ 0 ]
          {
            bounding_box[ 0 ] = *point;
          }
          if *point > bounding_box[ 1 ]
          {
            bounding_box[ 1 ] = *point;
          }
        }
      }

      let halfx = ( bounding_box[ 1 ][ 0 ] - bounding_box[ 0 ][ 0 ] ) / 2.0;
      let halfy = ( bounding_box[ 1 ][ 1 ] - bounding_box[ 0 ][ 1 ] ) / 2.0;

      for contour in contours.iter_mut()
      {
        for point in contour.iter_mut()
        {
          point[ 0 ] -= halfx;
          point[ 1 ] -= halfy;
        }
      }

      let [ [ x1, y1 ], [ x2,  y2 ] ] = bounding_box;
      bounding_box = [ [ x1 - halfx, y1 - halfy ], [ x2 - halfx, y2 - halfy ] ];

      Self
      {
        _character : character,
        contours,
        bounding_box
      }
    }

    fn scale( &mut self, scale : f64 )
    {
      let bounding_box = self.bounding_box;
      let halfx = ( self.bounding_box[ 1 ][ 0 ] - bounding_box[ 0 ][ 0 ] ) / 2.0;
      let halfy = ( bounding_box[ 1 ][ 1 ] - bounding_box[ 0 ][ 1 ] ) / 2.0;

      for contour in self.contours.iter_mut()
      {
        for point in contour.iter_mut()
        {
          point[ 0 ] -= halfx;
          point[ 1 ] -= halfy;
          point[ 0 ] *= scale;
          point[ 1 ] *= scale;
        }
      }

      let [ [ x1, y1 ], [ x2,  y2 ] ] = bounding_box;
      self.bounding_box = [ [ ( x1 - halfx ) * scale, ( y1 - halfy ) * scale ], [ ( x2 - halfx ) * scale, ( y2 - halfy ) * scale ] ];
    }

    fn from_glif( glif_bytes : Vec< u8 >, character : char ) -> Option< Self >
    {
      let glif_str = std::str::from_utf8( &glif_bytes ).unwrap();
      let mut reader = Reader::from_str( glif_str );
      reader.config_mut().trim_text( true );

      let mut _contours = vec![];
      let mut contour_points = vec![];
      let mut typ = PointType::Move;

      loop 
      {
        let event = reader.read_event();
        match event
        { 
          Ok( Event::Empty( e ) ) if e.starts_with( b"point" ) => 
          {
            let element = e.clone(); 

            let mut x = None;
            let mut y = None;
            let smooth = true;

            for attr in element.attributes()
            {
              let Ok( attr ) = attr
              else 
              {
                continue;
              };

              let Ok( value ) = String::from_utf8( attr.value.to_vec() )
              else 
              {
                continue;
              };

              match attr.key.0
              {
                b"x" => x = value.parse::< f64 >().ok(),
                b"y" => y = value.parse::< f64 >().ok(),
                b"typ" => 
                {
                  let Ok( t ) = PointType::from_str( &value )
                  else
                  {
                    continue;
                  };
                  typ = t;
                }
                _ => continue
              }
            }

            if x.is_none() || y.is_none()
            {
              continue;
            }

            contour_points.push(
              ContourPoint::new(
                x.unwrap(),
                y.unwrap(),
                typ,
                smooth,
                None,
                None
              )
            )
          },
          Ok( Event::End( e ) ) if e.starts_with( b"contour" ) => 
          {
            typ = PointType::Move;
            let mut contour = Contour::default();
            contour.points = contour_points.drain( .. ).collect::< Vec< _ > >();
            _contours.push( contour );
          },
          Ok( Event::Eof ) => break,
          _ => ()
        }
      }

      let mut contours = vec![];

      for contour in _contours
      {
        let mut path = vec![];
        let Ok( bez_path ) = contour.to_kurbo() 
        else
        {
          return None;
        };

        flatten( 
          bez_path.elements().iter().cloned(), 
          0.25, 
          | p | path.push( p ) 
        );

        let mut contour = vec![];

        path.iter()
        .for_each
        ( 
          | p |
          {
            match p
            {
              kurbo::PathEl::MoveTo( point ) |
              kurbo::PathEl::LineTo( point ) => contour.push( [ point.x, point.y ] ),
              kurbo::PathEl::ClosePath => 
              {
                contours.push( contour.clone() );
                contour.clear();
              },
              _ => ()
            }
          }
        );

        contours.push( contour );
      }

      contours.retain( | c | !c.is_empty() );

      if contours.is_empty()
      {
        return None;
      }

      Some( Glyph::new( contours, character ) )
    }
  }

  #[ derive( Clone ) ]
  pub struct Font
  {
    glyphs : HashMap< char, Glyph >
  }

  impl Font
  {
    async fn new( path : &str ) -> Self
    {
      let mut glyphs = HashMap::< char, Glyph >::new();
      let glyphs_path = path.to_string() + "/glyphs";

      for c in b'a'..=b'z' 
      {
        let glyph_path = format!( "{}/{}.glif", glyphs_path, c as char );
        let glif_bytes = gl::file::load( &glyph_path ).await
        .expect( "Failed to load glif file" );
        if let Some( glyph ) = Glyph::from_glif( glif_bytes, c as char )
        {
          glyphs.insert( c as char, glyph );
        }
      }

      for c in b'A'..=b'Z' 
      {
        let glyph_path = format!( "{}/{}_.glif", glyphs_path, c as char );
        let glif_bytes = gl::file::load( &glyph_path ).await
        .expect( "Failed to load glif file" );
        if let Some( glyph ) = Glyph::from_glif( glif_bytes, c as char )
        {
          glyphs.insert( c as char, glyph );
        }
      }

      for ( c, name ) in 
      [
        ( '0', "zero" ),
        ( '1', "one" ),
        ( '2', "two" ),
        ( '3', "three" ),
        ( '4', "four" ),
        ( '5', "five" ),
        ( '6', "six" ),
        ( '7', "seven" ),
        ( '8', "eight" ),
        ( '9', "nine" )
      ]
      {
        let glyph_path = format!( "{}/{}.glif", glyphs_path, name );
        let glif_bytes = gl::file::load( &glyph_path ).await
        .expect( "Failed to load glif file" );
        if let Some( glyph ) = Glyph::from_glif( glif_bytes, c )
        {
          glyphs.insert( c, glyph );
        }
      }

      let mut max_size = 0.0;
      for ( _, glyph ) in &glyphs
      {
        let glyph_size = glyph.bounding_box[ 1 ][ 0 ] - glyph.bounding_box[ 0 ][ 0 ];
        if max_size < glyph_size
        {
          max_size = glyph_size;
        }
      }

      for ( _, glyph ) in glyphs.iter_mut()
      {
        glyph.scale( 250.0 / max_size );
      }

      Self
      {
        glyphs
      }
    }
  }

  #[ derive( Clone ) ]
  struct Glyph3D
  {
    data : PrimitiveData,
    bounding_box : [ [ f64; 3 ]; 2 ]
  }

  impl From< Glyph > for Glyph3D
  {
    fn from( glyph : Glyph ) -> Self 
    {
      let Some( primitive_data ) = contours_to_mesh( &glyph.contours )
      else
      {
        return Self
        {
          data : PrimitiveData { 
            attributes : Rc::new( 
              RefCell::new( 
                AttributesData 
                { 
                  positions: vec![], 
                  normals: vec![], 
                  indices: vec![] 
                } 
              ) 
            ), 
            material : Rc::new( RefCell::new( Default::default() ) ), 
            transform : Default::default() 
          },
          bounding_box : [ [ 0.0; 3 ]; 2 ]
        };
      };

      let [ a, b ] = glyph.bounding_box;
      let bounding_box = [ [ a[ 0 ], a[ 1 ], -0.5 ], [ b[ 0 ], b[ 1 ], 0.5 ] ];

      Self
      {
        data : primitive_data,
        bounding_box
      }
    }
  }

  fn contours_to_mesh( contours : &[ Vec< [ f64; 2 ] > ] ) -> Option< PrimitiveData >
  {
    let mut body_id = 0;
    let mut max_box_diagonal_size = 0;
    for ( i, contour ) in contours.iter().enumerate()
    {
      if contour.is_empty()
      {
        continue;
      }
      let [ x1, y1 ] = contour.iter()
      .map( | [ a, b ] | [ *a as isize, *b as isize ] )
      .min().unwrap();
      let [ x2, y2 ] = contour.iter()
      .map( | [ a, b ] | [ *a as isize, *b as isize ] )
      .max().unwrap();
      let controur_size = ( ( x2 - x1 ).pow( 2 ) + ( y2 - y1 ).pow( 2 ) ).isqrt();
      if max_box_diagonal_size < controur_size
      {
        max_box_diagonal_size = controur_size;
        body_id = i;
      }
    }

    let mut contours = contours.to_vec();

    contours.swap( body_id, 0 );

    let contours = contours.into_iter()
    .map( 
      | c | 
      {
        c.into_iter()
        .map( 
          | [ x, y ] |
          {
            IntPoint
            {
              x : x as i32, 
              y : y as i32
            }
          } 
        )
        .collect::< Vec< _ > >()
      } 
    )
    .collect::< Vec< _ > >();

    let triangulation = contours.triangulate().to_triangulation::< u32 >();

    let flat_positions = triangulation.points;
    let mut indices = triangulation.indices;

    // Create two surface of glyph
    let mut positions = flat_positions.iter()
    .map(
      | p |
      {
        [ p.x as f32, p.y as f32, 0.5 ]
      }
    )
    .collect::< Vec< _ > >();

    let second_surface_positions = flat_positions.iter()
    .map(
      | p |
      {
        [ p.x as f32, p.y as f32, -0.5 ]
      }
    )
    .collect::< Vec< _ > >();

    positions.extend( second_surface_positions );

    let vertex_count = flat_positions.len() as u32;
    let second_surface_indices = indices.iter()
    .map( | i | i + vertex_count )
    .collect::< Vec< _ > >();

    indices.extend( second_surface_indices );  

    // Add border to glyph mesh
    let vc1 = positions.len() as u32;
    let vc2 = vc1 + contours.iter().flatten().count() as u32;

    for z in [ 0.5, -0.5 ]
    {
      for c in &contours
      {
        positions.extend( c.iter().map( | p | [ p.x as f32, p.y as f32, z ] ) );
      }
    }

    let mut edges = vec![];

    let mut offset = 0;
    for c in contours.iter()
    {
      let mut contour_edges = vec![];
      for ( i, _ ) in c.iter().enumerate() 
      {
        contour_edges.push( [ i as u32 + offset + vc1, i as u32 + offset + vc2 ] ); 
      }
      offset += c.len() as u32;

      edges.push( contour_edges );
    }

    for ce in &edges 
    {
      if ce.len() > 2
      {
        let mut i = 0; 
        while i < ce.len() - 1
        {
          // Counter clockwise ↺
          // [ i + 1 ] c *---* d
          //             |\  |
          //             | \ |
          //             |  \|        
          // [   i   ] a *---* b
          let [ a, b ] = [ ce[ i ][ 0 ], ce[ i ][ 1 ] ];
          let [ c, d ] = [ ce[ i + 1 ][ 0 ], ce[ i + 1 ][ 1 ] ];
          indices.extend( [ c, a, b ] );
          indices.extend( [ c, b, d ] );
          i += 1;
        }

        let first = ce.first().unwrap();
        let last = ce.last().unwrap();
        let [ a, b ] = [ last[ 0 ], last[ 1 ] ];
        let [ c, d ] = [ first[ 0 ], first[ 1 ] ];
        indices.extend( [ c, a, b ] );
        indices.extend( [ c, b, d ] );
      }
    }

    let mut normals = vec![ [ 0.0; 3 ]; positions.len() ];
    indices.chunks( 3 )
    .for_each
    ( 
      | ids | 
      {
        let a = F32x3::from( positions[ ids[ 0 ] as usize ] );
        let b = F32x3::from( positions[ ids[ 1 ] as usize ] );
        let c = F32x3::from( positions[ ids[ 2 ] as usize ] );
        let e1 = a - b;
        let e2 = c - b;
        let c = cross( &e1, &e2 );
        ( 0..3 ).for_each
        (
          | i | normals[ ids[ i ] as usize ] = c.normalize().as_array()
        );
      }
    );

    let attributes = AttributesData
    {
      positions, 
      normals, 
      indices, 
    };

    let primitive_data = PrimitiveData 
    { 
      attributes : Rc::new( RefCell::new( attributes ) ),
      material : Rc::new( RefCell::new( renderer::webgl::Material::default() ) ), 
      transform : Transform::default()  
    };

    Some( primitive_data )
  }

  pub struct Font3D
  {
    glyphs : HashMap< char, Glyph3D > 
  }

  impl From< Font > for Font3D
  {
    fn from( font : Font ) -> Self 
    {
      let mut glyphs = HashMap::< char, Glyph3D >::new();

      for ( char, glyph ) in font.glyphs 
      {
        glyphs.insert( char, glyph.into() );
      }

      Self
      {
        glyphs
      }
    }
  }

  pub async fn load_fonts( font_names : &[ String ] ) -> HashMap< String, Font >
  {
    let mut fonts = HashMap::< String, Font >::new();

    for font_name in font_names
    {
      let font_path = format!( "fonts/ufo/{}.ufo", font_name );
      fonts.insert( font_name.to_string(), Font::new( &font_path ).await );
    }
    
    fonts
  }

  pub async fn load_fonts_3d( font_names : &[ String ] ) -> HashMap< String, Font3D >
  {
    load_fonts( font_names )
    .await
    .iter()
    .map( | ( n, f ) | ( n.clone(), f.clone().into() ) )
    .collect::< HashMap< _, Font3D > >()
  }

  pub fn text_to_mesh( text : &str, font : &Font3D, transform : &Transform ) -> Vec< PrimitiveData >
  {
    let mut mesh = vec![]; 

    let mut max_x = 0.0;
    for ( _, glyph ) in &font.glyphs
    {
      let glyph_size_x = glyph.bounding_box[ 1 ][ 0 ] - glyph.bounding_box[ 0 ][ 0 ];
      if max_x < glyph_size_x
      {
        max_x = glyph_size_x;
      }
    }    

    let start_transform = transform.clone();
    let mut transform = start_transform.clone();
    transform.scale = [ 0.003, 0.003, 0.05 ];
    let halfx = ( max_x / 2.0 ) * transform.scale[ 0 ] as f64;
    transform.translation[ 0 ] -= ( text.len() as f32 * halfx as f32 ) / 2.0; 

    for char in text.chars()
    {
      let Some( mut glyph ) = font.glyphs.get( &char ).cloned() 
      else
      {
        transform.translation[ 0 ] += halfx as f32; 
        continue;
      };

      let diff = ( 250.0 - ( glyph.bounding_box[ 1 ][ 1 ] - glyph.bounding_box[ 0 ][ 1 ] ) ) * transform.scale[ 1 ] as f64;
      transform.translation[ 1 ] = start_transform.translation[ 1 ];
      transform.translation[ 1 ] -= diff as f32;
      transform.translation[ 0 ] += halfx as f32; 
      glyph.data.transform = transform.clone();

      mesh.push( glyph.data.clone() );
    }

    mesh
  }
}