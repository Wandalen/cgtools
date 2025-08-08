mod private
{
  use std::{collections::HashMap, str::FromStr};
  use kurbo::flatten;
  use mingl::geometry::BoundingBox;
  use norad::{ PointType, ContourPoint, Contour };
  use std::rc::Rc;
  use std::cell::RefCell;
  use minwebgl as gl;
  use gl::{ F32x3, F32x4 };
  use quick_xml::{ Reader, events::Event };
  use crate::
  { 
    AttributesData, PrimitiveData, Transform 
  };

  /// Represents a single character glyph with its contour data and rendered mesh.
  #[ derive( Clone ) ]
  pub struct Glyph
  {
    _character : char,
    contours : Vec< Vec< [ f32; 2 ] > >,
    body : Option< PrimitiveData >,
    bounding_box : BoundingBox
  }

  impl Glyph
  {
    fn new( contours : Vec< Vec< [ f64; 2 ] > >, character : char ) -> Self
    {
      let mut contours = contours.into_iter()
      .map
      ( 
        | v | 
        v.into_iter()
        .map
        ( 
          | [ a, b ] | [ a as f32, b as f32 ] 
        )
        .collect::< Vec< _ > >() 
      )
      .collect::< Vec< _ > >();

      let flat_contours = contours.iter().flatten().flatten().cloned().collect::< Vec< _ > >();
      let bounding_box = BoundingBox::compute2d( &flat_contours );

      let [ x1, y1 ] = [ bounding_box.left(), bounding_box.down() ];
      let [ x2, y2 ] = [ bounding_box.right(), bounding_box.up() ];

      let halfx = ( x2 - x1 ) / 2.0;
      let halfy = ( y2 - y1 ) / 2.0;
      let offsetx = x1;
      let offsety = y1;
      let offsetx = - halfx - offsetx;
      let offsety = - halfy - offsety;

      for contour in contours.iter_mut()
      {
        for point in contour.iter_mut()
        {
          point[ 0 ] += offsetx;
          point[ 1 ] += offsety;
        }
      }

      let bounding_box = BoundingBox
      {
        min : [ ( x1 + offsetx ) as f32, ( y1 + offsety ) as f32, 0.0 ].into(),
        max : [ ( x2 + offsetx ) as f32, ( y2 + offsety ) as f32, 0.0 ].into()
      };
      
      Self
      {
        _character : character,
        contours,
        body : None,
        bounding_box
      }
    }

    fn scale( &mut self, scale : f32)
    { 
      let [ x1, y1 ] = [ self.bounding_box.left(), self.bounding_box.down() ];
      let [ x2, y2 ] = [ self.bounding_box.right(), self.bounding_box.up() ];

      for contour in self.contours.iter_mut()
      {
        for point in contour.iter_mut()
        {
          point[ 0 ] *= scale;
          point[ 1 ] *= scale;
        }
      }

      self.bounding_box.min = [ x1 * scale, y1 * scale, 0.0 ].into();
      self.bounding_box.max = [ x2 * scale, y2 * scale, 0.0 ].into();
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

            contour_points.push
            (
              ContourPoint::new
              (
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
      let mut curves = vec![];

      for contour in _contours
      {
        let mut path = vec![];
        let Ok( bez_path ) = contour.to_kurbo() 
        else
        {
          return None;
        };

        flatten
        ( 
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

        curves.push( bez_path );
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

  /// UFO font containing a collection of glyphs with size information.
  #[ derive( Clone ) ]
  pub struct Font
  {
    glyphs : HashMap< char, Glyph >,
    max_size : BoundingBox
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

      let [ mut max_x, mut max_y ] = [ 0.0, 0.0 ];
      for ( _, glyph ) in &glyphs
      {
        let [ x1, y1 ] = [ glyph.bounding_box.left(), glyph.bounding_box.down() ];
        let [ x2, y2 ] = [ glyph.bounding_box.right(), glyph.bounding_box.up() ];
        let x = x2 - x1;
        let y = y2 - y1;
        if max_x < x
        {
          max_x = x;
        }
        if max_y < y
        {
          max_y = y;
        }
      }

      let scale = 250.0;
      for ( _, glyph ) in glyphs.iter_mut()
      {
        glyph.scale( scale / max_y );
      }

      let mut min = F32x3::MAX; 
      let mut max = F32x3::MIN; 
      for ( _, glyph ) in &glyphs
      {
        if min > glyph.bounding_box.min
        {
          min = glyph.bounding_box.min;
        }
        if max < glyph.bounding_box.max
        {
          max = glyph.bounding_box.max;
        }
      }

      for ( _, glyph ) in glyphs.iter_mut()
      {
        glyph.body = contours_to_mesh( &glyph.contours );
      }

      Self
      {
        glyphs,
        max_size : BoundingBox 
        { 
          min, 
          max  
        }
      }
    }
  }

  /// Converts a set of 2D contours into a triangulated mesh with holes support.
  pub fn contours_to_mesh( contours : &[ Vec< [ f32; 2 ] > ] ) -> Option< PrimitiveData >
  {
    if contours.is_empty()
    {
      return None;
    }

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

    let body_bounding_box = BoundingBox::compute2d
    ( 
      contours.get( body_id ).unwrap()
      .iter()
      .flatten()
      .cloned()
      .collect::< Vec< _ > >()
      .as_slice()
    );

    let mut outside_body_list = vec![];
    let mut inside_body_list = vec![];
    for ( i, contour ) in contours.iter().enumerate()
    {
      if body_id == i
      {
        continue;
      }

      let bounding_box = BoundingBox::compute2d
      ( 
        contour
        .iter()
        .flatten()
        .cloned()
        .collect::< Vec< _ > >() 
        .as_slice()
      );

      let has_part_outside_body = bounding_box.left() < body_bounding_box.left() ||
      bounding_box.right() > body_bounding_box.right() ||
      bounding_box.up() > body_bounding_box.up() ||
      bounding_box.down() < body_bounding_box.down();

      if has_part_outside_body
      {
        outside_body_list.push( contour.clone() );
      }
      else
      {
        inside_body_list.push( contour.clone() );
      }
    }

    let mut base = vec![ contours[ body_id ].clone() ];
    base.extend( inside_body_list );

    let mut bodies = vec![ base ];
    bodies.extend( outside_body_list.into_iter().map( | c | vec![ c ] ) );

    let mut positions = vec![];
    let mut indices = vec![];

    for contours in bodies
    {
      let mut flat_positions: Vec< f64 > = Vec::new();
      let mut hole_indices: Vec< usize > = Vec::new();

      if let Some( outer_contour ) = contours.get( 0 ) 
      {
        if outer_contour.is_empty() 
        {
          return None;
        }
        for &[ x, y ] in outer_contour 
        {
          flat_positions.push( x as f64 );
          flat_positions.push( y as f64 );
        }
      } 
      else 
      {
        return None;
      }

      // Process holes (remaining contours)
      // Their winding order must be opposite to the outer (e.g., CW for holes)
      for i in 1..contours.len() 
      {
        let hole_contour = &contours[ i ];
        if hole_contour.is_empty() 
        {
          continue;
        }

        hole_indices.push( flat_positions.len() / 2 );

        for &[ x, y ] in hole_contour 
        {
          flat_positions.push( x as f64 );
          flat_positions.push( y as f64 );
        }
      }

      // Perform triangulation
      let Ok( body_indices ) = earcutr::earcut( &flat_positions, &hole_indices, 2 ) 
      else
      {
        continue;
      };

      let body_indices = body_indices.into_iter()
      .map( | i | i as u32 )
      .collect::< Vec< _ > >();

      let body_positions = flat_positions.chunks( 2 )                                     
      .map( | c | [ c[ 0 ] as f32, c[ 1 ] as f32, 0.0 ] )
      .collect::< Vec< _ > >();

      let positions_count = positions.len();
      positions.extend( body_positions );
      indices.extend
      ( 
        body_indices.iter()
        .map( | i | i + positions_count as u32 ) 
      );
    }

    let attributes = AttributesData
    {
      positions, 
      indices, 
    };

    let primitive_data = PrimitiveData 
    { 
      attributes : Rc::new( RefCell::new( attributes ) ),
      color : F32x4::default(),
      transform : Transform::default()  
    };

    Some( primitive_data )
  }

  /// Loads multiple UFO fonts by name from the fonts/ufo directory.
  pub async fn load_fonts( font_names : &[ &str ] ) -> HashMap< String, Font >
  {
    let mut fonts = HashMap::< String, Font >::new();

    for font_name in font_names
    {
      let font_path = format!( "fonts/ufo/{}.ufo", font_name );
      fonts.insert( font_name.to_string(), Font::new( &font_path ).await );
    }
    
    fonts
  }

  /// Converts text string into a collection of filled mesh primitives using the specified font.
  pub fn text_to_mesh( text : &str, font : &Font, transform : &Transform ) -> Vec< PrimitiveData >
  {
    let mut mesh = vec![]; 

    let start_transform = transform.clone();
    let mut transform = start_transform.clone();
    transform.scale = [ 0.003, 0.003, 1.0 ].into();
    let max_x = font.max_size.max[ 0 ] - font.max_size.min[ 0 ];
    let max_y = font.max_size.max[ 1 ] - font.max_size.min[ 1 ];
    let halfx = max_x * transform.scale[ 0 ];

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char )
      else
      {
        transform.translation[ 0 ] -= halfx / 2.0; 
        continue;
      };

      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] -= if glyph_x < halfx / 4.0
      {
        halfx / 2.0
      }
      else
      {
        glyph_x / 2.0
      }
    }

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char ).cloned() 
      else
      {
        transform.translation[ 0 ] += halfx; 
        continue;
      };

      let glyph_y = glyph.bounding_box.height();
      let diff = ( max_y - ( glyph_y * 0.5 ) ) * transform.scale[ 1 ];
      transform.translation[ 1 ] = start_transform.translation[ 1 ];
      transform.translation[ 1 ] -= diff;
      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] += if glyph_x < halfx / 4.0
      {
        halfx
      }
      else
      {
        glyph_x
      };
      if let Some( mut geometry ) = glyph.body.clone()
      {
        geometry.transform = transform.clone();
        mesh.push( geometry );
      }
    }

    mesh
  }

  /// Converts text string into outlined contour meshes with specified line width.
  pub fn text_to_countour_mesh( 
    text : &str, 
    font : &Font, 
    transform : &Transform, 
    width : f32 
  ) -> Vec< PrimitiveData >
  {
    let mut mesh = vec![]; 

    let start_transform = transform.clone();
    let mut transform = start_transform.clone();
    transform.scale = [ 0.003, 0.003, 1.0 ].into();
    let max_x = font.max_size.max[ 0 ] - font.max_size.min[ 0 ];
    let max_y = font.max_size.max[ 1 ] - font.max_size.min[ 1 ];
    let halfx = max_x * transform.scale[ 0 ];

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char )
      else
      {
        transform.translation[ 0 ] -= halfx / 2.0; 
        continue;
      };

      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] -= if glyph_x < halfx / 4.0
      {
        halfx / 2.0
      }
      else
      {
        glyph_x / 2.0
      }
    }

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char ).cloned() 
      else
      {
        transform.translation[ 0 ] += halfx; 
        continue;
      };

      let glyph_y = glyph.bounding_box.height();
      let diff = ( max_y - ( glyph_y * 0.5 ) ) * transform.scale[ 1 ];
      transform.translation[ 1 ] = start_transform.translation[ 1 ];
      transform.translation[ 1 ] -= diff;
      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] += if glyph_x < halfx / 4.0
      {
        halfx
      }
      else
      {
        glyph_x
      };
      
      for curve in glyph.contours
      {
        let Some( mut geometry ) = crate::primitive::curve_to_geometry( &curve, width )
        else
        {
          continue;
        };

        geometry.transform = transform.clone();
        mesh.push( geometry );
      }
    }

    mesh
  }
}

crate::mod_interface!
{
  orphan use
  {
    load_fonts,
    Glyph,
    Font,
    contours_to_mesh,
    text_to_mesh,
    text_to_countour_mesh
  };
}
