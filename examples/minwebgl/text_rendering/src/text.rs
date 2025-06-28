
pub mod ufo
{
  use std::{collections::HashMap, str::FromStr};
  use kurbo::flatten;
  use mingl::{geometry::BoundingBox, IntoArray};
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
    character : char,
    contours : Vec< Vec< [ f32; 2 ] > >,
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
        character,
        contours,
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
    glyphs : HashMap< char, Glyph >,
    _max_size : BoundingBox
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

      Self
      {
        glyphs,
        _max_size : BoundingBox 
        { 
          min, 
          max  
        }
      }
    }
  }

  #[ derive( Clone ) ]
  struct Glyph3D
  {
    _character : char,
    data : PrimitiveData,
    bounding_box : BoundingBox
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
          _character : ' ',
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
          bounding_box : BoundingBox::default()
        };
      };

      let a = glyph.bounding_box.min.0;
      let b = glyph.bounding_box.max.0;
      let bounding_box = BoundingBox 
      { 
        min : [ a[ 0 ], a[ 1 ], -0.5 ].into(), 
        max : [ b[ 0 ], b[ 1 ], 0.5 ].into() 
      };

      Self
      {
        _character : glyph.character,
        data : primitive_data,
        bounding_box
      }
    }
  }

  fn contours_to_mesh( contours : &[ Vec< [ f32; 2 ] > ] ) -> Option< PrimitiveData >
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
    glyphs : HashMap< char, Glyph3D >,
    max_size : BoundingBox
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

      let mut min = F32x3::MAX; 
      let mut max = F32x3::MIN; 
      for ( _, glyph ) in glyphs.iter_mut()
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

  pub async fn load_fonts( font_names : &[ String ] ) -> HashMap< String, Font >
  {
    let mut fonts = HashMap::< String, Font >::new();

    for font_name in font_names
    {
      let font_path = format!( "/fonts/ufo/{}.ufo", font_name );
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

    let start_transform = transform.clone();
    let mut transform = start_transform.clone();
    transform.scale = [ 0.003, 0.003, 0.05 ];
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
      let Some( mut glyph ) = font.glyphs.get( &char ).cloned() 
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
      glyph.data.transform = transform.clone();

      mesh.push( glyph.data.clone() );
    }

    mesh
  }
}

pub mod ttf 
{
  use crate::{ PrimitiveData, AttributesData, Transform };
  use csgrs::CSG;
  use minwebgl as gl;
  use gl::
  {
    geometry::BoundingBox,
    web::file,
    math::vector::cross,
    F32x3,
    IntoArray
  };
  use std::rc::Rc;
  use std::cell::RefCell;
  use std::collections::HashMap;

  #[ derive( Clone ) ]
  struct Glyph3D
  {
    _character : char,
    data : PrimitiveData,
    bounding_box : BoundingBox
  }

  impl Glyph3D
  {
    fn from_ttf( ttf_bytes : &[ u8 ], character : char ) -> Self
    {
      let c = character.to_string();
      let mut csg : CSG< () > = CSG::text( &c, ttf_bytes, 1.0, None )
      .extrude( 0.5 );

      let m = csg.bounding_box().mins;
      csg = csg.translate_vector( [ - m.x, - m.y, - m.z ].into() );

      let m = csg.bounding_box().maxs;
      csg = csg.translate_vector( [ - m.x / 2.0, - m.y / 2.0, - m.z / 2.0 ].into() );

      let min = csg.bounding_box().mins;
      let max = csg.bounding_box().maxs;
      let min = [ min.x as f32, min.y as f32, min.z as f32 ];
      let max = [ max.x as f32, max.y as f32, max.z as f32 ];

      let bounding_box = BoundingBox::new( min, max );
      
      let mesh = csg.to_trimesh().unwrap();

      let positions = mesh.vertices()
      .iter()
      .map( | p | [ p.coords.x as f32, p.coords.y as f32, p.coords.z as f32 ] )
      .collect::< Vec< _ > >();

      let indices = mesh.indices()
      .iter()
      .cloned()
      .flatten()
      .collect::< Vec< _ > >();

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
        positions : positions, 
        normals, 
        indices, 
      };

      let data = PrimitiveData 
      { 
        attributes : Rc::new( RefCell::new( attributes ) ),
        material : Rc::new( RefCell::new( renderer::webgl::Material::default() ) ), 
        transform : Transform::default()  
      };

      Self
      {
        _character : character,
        data,
        bounding_box,
      }
    }

    fn scale( &mut self, scale : [ f32; 3 ] )
    { 
      for position in self.data.attributes.borrow_mut().positions.iter_mut()
      {
        position[ 0 ] *= scale[ 0 ];
        position[ 1 ] *= scale[ 1 ];
        position[ 2 ] *= scale[ 2 ];
      }

      let scale : F32x3 = scale.into();
      self.bounding_box.min = self.bounding_box.min * scale;
      self.bounding_box.max = self.bounding_box.max * scale;
    }
  }

  pub struct Font3D
  {
    glyphs : HashMap< char, Glyph3D >,
    max_size : BoundingBox
  }

  impl Font3D
  {
    async fn new( path : &str ) -> Self
    {
      let ttf_bytes = file::load( path ).await
      .expect( "Failed to load ttf file" );

      let mut glyphs = HashMap::< char, Glyph3D >::new();

      for c in [ 'C', 'G', 'T', 'o', 'l', 's' ]
      {
        glyphs.insert( c, Glyph3D::from_ttf( &ttf_bytes, c as char ) );
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
        glyph.scale( [ scale / max_y, scale / max_y, 1.0 ] );
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

  pub async fn load_fonts_3d( font_names : &[ String ] ) -> HashMap< String, Font3D >
  {
    let mut fonts = HashMap::< String, Font3D >::new();

    for font_name in font_names
    {
      let font_path = format!( "/fonts/ttf/{}.ttf", font_name );
      fonts.insert( font_name.to_string(), Font3D::new( &font_path ).await );
    }
    
    fonts
  }

  pub fn text_to_mesh( text : &str, font : &Font3D, transform : &Transform ) -> Vec< PrimitiveData >
  {
    let mut mesh = vec![]; 

    let start_transform = transform.clone();
    let mut transform = start_transform.clone();
    transform.scale = [ 0.003, 0.003, 0.05 ];
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
      let Some( mut glyph ) = font.glyphs.get( &char ).cloned() 
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
      glyph.data.transform = transform.clone();

      mesh.push( glyph.data.clone() );
    }

    mesh
  }
}