//! This module provides functionality for loading UFO
//! fonts and converting text into a 3D mesh representation.


// Fix(TASK-021)
// Root cause : this module was gated on `text` while its glyph pipeline calls
// `contours_to_fill_geometry`, which only exists under `font-processing` — so
// `--features text` alone could never compile (surfaced by TASK-055).
// Pitfall : gate a module on the feature that provides everything it calls,
// not on the broadest feature it is thematically related to.
#[ cfg( feature = "font-processing" ) ]
mod private
{
  use rustc_hash::FxHashMap;
  use std::str::FromStr;
  use kurbo::flatten;
  use mingl::geometry::BoundingBox;
  use norad::{ PointType, ContourPoint, Contour };
  use minwebgl as gl;
  use gl::F32x3;
  use quick_xml::{ Reader, events::Event };
  use crate::
  {
    PrimitiveData,
    Transform,
    contours_to_fill_geometry
  };

  /// Represents a single character glyph, including its contours and a generated 3D body.
  #[ derive( Clone ) ]
  pub struct Glyph
  {
    /// The character associated with the glyph.
    _character : char,
    /// A vector of contours, where each contour is a vector of 2D points.
    contours : Vec< Vec< [ f32; 2 ] > >,
    /// The generated 3D primitive data for the glyph's body.
    body : Option< PrimitiveData >,
    /// The bounding box of the glyph.
    bounding_box : BoundingBox
  }

  impl Glyph
  {
    /// Creates a new `Glyph` from a vector of 2D contours and a character.
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

      let flat_contours = contours.iter().flatten().flatten().copied().collect::< Vec< _ > >();
      let bounding_box = BoundingBox::compute2d( &flat_contours );

      let [ x1, y1 ] = [ bounding_box.left(), bounding_box.down() ];
      let [ x2, y2 ] = [ bounding_box.right(), bounding_box.up() ];

      let half_x = ( x2 - x1 ) / 2.0;
      let half_y = ( y2 - y1 ) / 2.0;
      let offset_x = x1;
      let offset_y = y1;
      let offset_x = - half_x - offset_x;
      let offset_y = - half_y - offset_y;

      for contour in &mut contours
      {
        for point in contour.iter_mut()
        {
          point[ 0 ] += offset_x;
          point[ 1 ] += offset_y;
        }
      }

      let bounding_box = BoundingBox::new
      (
        [ ( x1 + offset_x ), ( y1 + offset_y ), 0.0 ],
        [ ( x2 + offset_x ), ( y2 + offset_y ), 0.0 ]
      );

      Self
      {
        _character : character,
        contours,
        body : None,
        bounding_box
      }
    }

    /// Scales the glyph's contours and bounding box by a given factor.
    fn scale( &mut self, scale : f32)
    {
      let [ x1, y1 ] = [ self.bounding_box.left(), self.bounding_box.down() ];
      let [ x2, y2 ] = [ self.bounding_box.right(), self.bounding_box.up() ];

      for contour in &mut self.contours
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

    /// Creates a `Glyph` from a `.glif` file's byte data.
    #[ expect( clippy::too_many_lines, reason = "the glif XML event loop is one linear state machine; splitting it into helpers would scatter the per-event state transitions without shrinking the logic" ) ]
    fn from_glif( glif_bytes : &[ u8 ], character : char ) -> Option< Self >
    {
      let glif_str = std::str::from_utf8( glif_bytes ).unwrap();
      let mut reader = Reader::from_str( glif_str );
      reader.config_mut().trim_text( true );

      let mut raw_contours = vec![];
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
                _ => {}
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
            );
          },
          Ok( Event::End( e ) ) if e.starts_with( b"contour" ) =>
          {
            typ = PointType::Move;
            let mut contour = Contour::default();
            contour.points = std::mem::take(&mut contour_points);
            raw_contours.push( contour );
          },
          Ok( Event::Eof ) => break,
          _ => ()
        }
      }

      let mut contours = vec![];
      let mut curves = vec![];

      for contour in raw_contours
      {
        let mut path = vec![];
        let Ok( bez_path ) = contour.to_kurbo()
        else
        {
          return None;
        };

        flatten
        (
          bez_path.elements().iter().copied(),
          0.25,
          | p | path.push( p )
        );

        let mut contour = vec![];

        for p in &path
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

  /// Represents a font loaded from UFO files, containing a collection of glyphs.
  #[ derive( Clone ) ]
  pub struct Font
  {
    /// A map of characters to their corresponding glyphs.
    glyphs : FxHashMap< char, Glyph >,
    /// The maximum bounding box of glyph in the font.
    max_size : BoundingBox
  }

  impl Font
  {
    /// Asynchronously loads a new `Font` from a UFO directory path.
    async fn new( path : &str ) -> Self
    {
      let mut glyphs = FxHashMap::< char, Glyph >::default();
      let glyphs_path = path.to_string() + "/glyphs";

      for c in b'a'..=b'z'
      {
        let glyph_path = format!( "{}/{}.glif", glyphs_path, c as char );
        let glif_bytes = gl::file::load( &glyph_path ).await
        .expect( "Failed to load glif file" );
        if let Some( glyph ) = Glyph::from_glif( &glif_bytes, c as char )
        {
          glyphs.insert( c as char, glyph );
        }
      }

      for c in b'A'..=b'Z'
      {
        let glyph_path = format!( "{}/{}_.glif", glyphs_path, c as char );
        let glif_bytes = gl::file::load( &glyph_path ).await
        .expect( "Failed to load glif file" );
        if let Some( glyph ) = Glyph::from_glif( &glif_bytes, c as char )
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
        let glyph_path = format!( "{glyphs_path}/{name}.glif" );
        let glif_bytes = gl::file::load( &glyph_path ).await
        .expect( "Failed to load glif file" );
        if let Some( glyph ) = Glyph::from_glif( &glif_bytes, c )
        {
          glyphs.insert( c, glyph );
        }
      }

      let [ mut max_x, mut max_y ] = [ 0.0, 0.0 ];
      for glyph in glyphs.values()
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
      for glyph in glyphs.values_mut()
      {
        glyph.scale( scale / max_y );
      }

      let mut min = F32x3::MAX;
      let mut max = F32x3::MIN;
      for glyph in glyphs.values()
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

      for glyph in glyphs.values_mut()
      {
        glyph.body = contours_to_fill_geometry( &glyph.contours );
      }

      Self
      {
        glyphs,
        max_size : BoundingBox::new( min, max )
      }
    }
  }

  /// Asynchronously loads multiple fonts from a list of font names.
  pub async fn fonts_load( font_names : &[ &str ] ) -> FxHashMap< String, Font >
  {
    let mut fonts = FxHashMap::< String, Font >::default();

    for font_name in font_names
    {
      let font_path = format!( "static/fonts/ufo/{font_name}.ufo" );
      fonts.insert( (*font_name).to_string(), Font::new( &font_path ).await );
    }

    fonts
  }

  /// Converts text string into a collection of filled mesh primitives using the specified font.
  #[ must_use ]
  pub fn text_to_mesh( text : &str, font : &Font, transform : &Transform ) -> Vec< PrimitiveData >
  {
    let mut mesh = vec![];

    let start_transform = transform.clone();
    let mut transform = start_transform.clone();
    transform.scale = [ 0.003, 0.003, 1.0 ].into();
    let max_x = font.max_size.max[ 0 ] - font.max_size.min[ 0 ];
    let max_y = font.max_size.max[ 1 ] - font.max_size.min[ 1 ];
    let half_x = max_x * transform.scale[ 0 ];

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char )
      else
      {
        transform.translation[ 0 ] -= half_x / 2.0;
        continue;
      };

      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] -= if glyph_x < half_x / 4.0
      {
        half_x / 2.0
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
        transform.translation[ 0 ] += half_x;
        continue;
      };

      let glyph_y = glyph.bounding_box.height();
      let diff = ( max_y - ( glyph_y * 0.5 ) ) * transform.scale[ 1 ];
      transform.translation[ 1 ] = start_transform.translation[ 1 ];
      transform.translation[ 1 ] -= diff;
      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] += if glyph_x < half_x / 4.0
      {
        half_x
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
  #[ must_use ]
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
    let half_x = max_x * transform.scale[ 0 ];

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char )
      else
      {
        transform.translation[ 0 ] -= half_x / 2.0;
        continue;
      };

      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] -= if glyph_x < half_x / 4.0
      {
        half_x / 2.0
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
        transform.translation[ 0 ] += half_x;
        continue;
      };

      let glyph_y = glyph.bounding_box.height();
      let diff = ( max_y - ( glyph_y * 0.5 ) ) * transform.scale[ 1 ];
      transform.translation[ 1 ] = start_transform.translation[ 1 ];
      transform.translation[ 1 ] -= diff;
      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] += if glyph_x < half_x / 4.0
      {
        half_x
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

// Without `font-processing` the UFO pipeline simply does not exist — the
// symbols are configured out (loud absence at compile time), matching how
// `contours_to_fill_geometry` is gated in `primitive.rs`. The former
// always-return-None/empty stubs were silent-failure machinery with drifted
// signatures and are gone (TASK-021).
#[ cfg( not( feature = "font-processing" ) ) ]
mod private
{
}

crate::mod_interface!
{
  #[ cfg( feature = "font-processing" ) ]
  orphan use
  {
    fonts_load,
    Glyph,
    Font,
    text_to_mesh,
    text_to_countour_mesh
  };
}
