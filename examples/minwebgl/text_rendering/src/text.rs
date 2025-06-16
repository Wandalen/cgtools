
pub mod norad
{
  use std::collections::HashMap;

use gltf::Material;
use kurbo::flatten;
use norad::Codepoints;
use renderer::webgl::{primitive, Geometry};
use std::rc::Rc;
use std::cell::RefCell;
use triangulate::{ PoligonList, ListFormat };

use crate::
  { 
    AttributesData, PrimitiveData, Transform 
  };

  #[ derive( Clone ) ]
  struct Glyph
  {
    contours : Vec< Vec< [ f64; 2 ] > >,
    bounding_box : [ [ f64; 2 ]; 2 ]
  }

  impl Glyph
  {
    fn new( mut contours : Vec< Vec< [ f64; 2 ] > > ) -> Self
    {
      let mut bounding_box = [ [ f64::MAX; 2 ], [ f64::MIN; 2 ] ];

      for contour in contours
      {
        for point in contour
        {
          if point < bounding_box[ 0 ]
          {
            bounding_box[ 0 ] = point;
          }
          if point > bounding_box[ 1 ]
          {
            bounding_box[ 1 ] = point;
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
        contours,
        bounding_box
      }
    }
  }

  #[ derive( Clone ) ]
  pub struct Font
  {
    glyphs : HashMap< char, Glyph >
  }

  impl From< norad::Font > for Font 
  {
    fn from( font : norad::Font ) -> Self
    {
      let mut glyphs = HashMap::< char, Glyph >::new();
      'glyph : for glyph in font.iter_names()
      {
        if let Some( glyph ) = font.get_glyph( glyph.as_str() )
        {
          for codepoint in glyph.codepoints
          {
            let mut contours = vec![];

            for contour in glyph.contours
            {
              let mut path = vec![];
              let Ok( bez_path ) = contour.to_kurbo() 
              else
              {
                continue 'glyph;
              };

              flatten( 
                bez_path.elements(), 
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
                    kurbo::PathEl::MoveTo( point ) => 
                    {
                      contours.push( contour.clone() );
                      contour.clear();
                      contour.push( [ point.x, point.y ] );
                    }
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

            let _glyph = Glyph::new( contours );

            glyphs.insert( codepoint, _glyph );
          }
        }
      }

      Self
      {
        glyphs
      }
    }
  }

  struct Glyph3D
  {
    data : PrimitiveData,
    bounding_box : [ [ f64; 3 ]; 2 ]
  }

  impl From< Glyph > for Glyph3D
  {
    fn from( glyph : Glyph ) -> Self 
    {
      let mut flat_positions = Vec::< [ f64; 2 ] >::new();
      let mut indices = Vec::< u32 >::new(); 

      // Glyph surface triangulation
      glyph.contours.triangulate( triangulate::format::DeindexedListFormat::new( &mut flat_positions ).into_fan_format() )
      .expect( "Triangulation failed" );

      glyph.contours.triangulate( triangulate::format::IndexedListFormat::new( &mut indices ).into_fan_format() )
      .expect( "Triangulation failed" );

      // Create two surface of glyph
      let mut positions = flat_positions.iter()
      .map(
        | p |
        {
          [ p[ 0 ], p[ 1 ], 0.5 ]
        }
      )
      .collect::< Vec< _ > >();

      let second_surface_positions = flat_positions.iter()
      .map(
        | p |
        {
          [ p[ 0 ], p[ 1 ], -0.5 ]
        }
      )
      .collect::< Vec< _ > >();

      positions.extend( second_surface_positions );

      let vertex_count = flat_positions.len();
      let second_surface_indices = indices.iter()
      .map( | i | i + vertex_count )
      .collect::< Vec< _ > >();

      indices.extend( second_surface_indices );  

      // Add border to glyph mesh
      let vc1 = positions.len() as u32;
      let vc2 = vc1 + glyph.contours.iter().flatten().count() as u32;

      for z in [ 0.5, -0.5 ]
      {
        for c in glyph.contours
        {
          positions.extend( c.iter().map( | p | [ p[ 0 ], p[ 1 ], z ] ) );
        }
      }

      let mut edges = vec![];

      for c in glyph.contours.iter()
      {
        let mut contour_edges = vec![];
        for ( i, _ ) in c.iter().enumerate() 
        {
          contour_edges.push( [ i as u32 + vc, i as u32 + vc2 ] ); 
        }

        edges.push( contour_edges );
      }

      for ce in edges 
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

          let last = ce.len() - 1;
          let [ a, b ] = [ ce[ last ][ 0 ], ce[ last ][ 1 ] ];
          let [ c, d ] = [ ce[ 0 ][ 0 ], ce[ 0 ][ 1 ] ];
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
          let t = ( 0..3 ).map( | i | F32x3::from( positions[ ids[ i ] as usize ] ) )
          .collect::< Vec< _ > >();
          let e1 = t[ 0 ] - t[ 1 ];
          let e2 = t[ 2 ] - t[ 1 ];
          let c = ndarray_cg::vector::cross( &e1, &e2 );
          ( 0..3 ).for_each
          (
            | i | normals[ ids[ i ] as usize ] = [ c[ 0 ], c[ 1 ], c[ 2 ] ]
          );
        }
      );

      normals.iter_mut()
      .for_each( 
        | n | *n = *F32x3::from_array( *n ).normalize()
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

      let [ a, b ] = glyph.bounding_box;
      let bounding_box = [ [ a[ 0 ], a[ 1 ], -0.5 ], [ b[ 0 ], b[ 1 ], 0.5 ] ];

      Self
      {
        data : primitive_data,
        bounding_box
      }
    }
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

  pub fn load_fonts( font_names : Vec< String > ) -> HashMap< String, Font >
  {
    let mut fonts = HashMap::< String, norad::Font >::new();

    for font_name in font_names
    {
      let font_path = "fonts/ufo/".to_string() + &font_name + ".ufo";
      let font = norad::Font::load( font_path ).expect( "failed to load font" );
      fonts.insert( font_name, font );
    }
    
    fonts
  }

  pub fn load_fonts_3d( font_names : Vec< String > ) -> HashMap< String, Font >
  {
    load_fonts( font_names )
    .iter()
    .map( | ( n, f ) | ( n.clone(), f.clone().into() ) )
    .collect::< HashMap< _, text::norad::Font3D > >()
  }

  pub fn text_to_mesh( text : &str, font : &Font3D ) -> Vec< PrimitiveData >
  {
    let mut mesh = vec![]; 

    let mut transform = Transform::default();
    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char ).cloned() 
      else
      {
        continue;
      };

      let half = ( glyph.bounding_box[ 1 ][ 0 ] - glyph.bounding_box[ 0 ][ 0 ] ) / 2.0;
      transform.translation[ 0 ] -= half as f32; 
    }

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char ).cloned() 
      else
      {
        continue;
      };

      let half = ( glyph.bounding_box[ 1 ][ 0 ] - glyph.bounding_box[ 0 ][ 0 ] ) / 2.0;
      transform.translation[ 0 ] += half as f32; 
      glyph.data.transform = transform;
      transform.translation[ 0 ] += half as f32; 
      mesh.push( glyph );
    }

    mesh
  }

  // pub fn text_to_image( text : &str, font : &Font, dpi : f32 ) -> PrimitiveData
  // {

  // }
}

// mod parley
// {
//   // pub fn load_fonts( font_names : Vec< String > ) -> FontContext
//   // {
//   //   let font_cx = FontContext::new();

//   //   for font_name in font_names
//   //   {
//   //     let font_path = "fonts/ttf/".to_string() + &font_name + "ttf";
//   //     let font = ;
//   //     font_cx.collection.register_fonts( data, info_override );
//   //     font_cx
//   //   }
//   // }

//   fn text_to_layout( font_cx : &mut FontContext, text : &str, font : &str ) -> Layout< () >
//   {
//     let mut layout_cx = LayoutContext::new();

//     let mut builder = layout_cx.ranged_builder( &mut font_cx, &text, 1.0, true );

//     builder.push_default( StyleProperty::FontSize( 14.0 ) );
//   }

//   fn layout_to_mesh( layout : Layout< () > ) -> PrimitiveData
//   {
//     let positions = ;
//     let normals = ;
//     let indices = ;

//     for line in layout.lines() 
//     {
//       for item in line.items() 
//       {
//         match item 
//         {
//           PositionedLayoutItem::GlyphRun( glyph_run ) => 
//           {

//           }
//           PositionedLayoutItem::InlineBox( inline_box ) => 
//           {
            
//           }
//         };
//       }
//     }

//     PrimitiveData
//     {
//       positions,
//       normals,
//       indices,
//       material : Rc::new( RefCell::new( Material::default() ) ),
//       transform : Tra,
//     }
//   }

//   // fn layout_to_image( layout : Layout< () >, dpi : f32 ) -> DynamicImage
//   // {

//   // }

//   pub fn text_to_mesh( font_cx : &mut FontContext, text : &str, font : &str ) -> PrimitiveData
//   {
//     let layout = text_to_layout( font_cx, text, font );
//     layout_to_mesh( layout )
//   }

//   // pub fn text_to_image( font_cx : &mut FontContext, text : &str, font : &str, dpi : f32 ) -> PrimitiveData
//   // {
//   //   let layout = text_to_layout( &mut font_cx, text, font );
//   //   layout_to_image( layout, dpi )
//   // }
// }