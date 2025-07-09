use alloc::{ string::String, vec::Vec };
use core::ops::Range;
use hashbrown::HashMap;
use kurbo::{ Affine, PathEl, Point, Size, Vec2 };
use peniko::{ BlendMode, Color };
use serde::{ Deserialize, Deserializer };
use approx::abs_diff_eq;

// Import actual interpoli types
use interpoli::
{
  animated, fixed,
  Composition, Content, Draw, Geometry, GroupTransform, Layer, Mask, Repeater,
  Shape, Stroke, Transform, Brush,
};
use interpoli::value::{ Animated, Easing, EasingHandle, Time, Tween, Value, ValueRef };


// --- Lottie JSON Data Structures ---

#[ derive( Debug, Deserialize ) ]
#[ serde( untagged ) ]
enum LottieValue< T >
{
  Single( T ),
  Keyframes( Vec< LottieKeyframe< T > > ),
}

#[ derive( Debug, Deserialize ) ]
struct LottieKeyframe< T >
{
  #[ serde( rename = "s" ) ]
  start_value : Option< T >, // Start value of the keyframe
  #[ serde( rename = "e" ) ]
  end_value : Option< T >, // End value of the keyframe
  #[ serde( rename = "t" ) ]
  time : f64, // Time ( frame ) of the keyframe
  #[ serde( rename = "i" ) ]
  easing_in : Option< LottieBezierHandle >, // Incoming bezier handle
  #[ serde( rename = "o" ) ]
  easing_out : Option< LottieBezierHandle >, // Outgoing bezier handle
  #[ serde( rename = "h" ) ]
  hold : Option< f64 >, // Hold keyframe ( 1 = hold, 0 = interpolate )
}

#[ derive( Debug, Deserialize ) ]
struct LottieBezierHandle
{
  #[ serde( rename = "x" ) ]
  x : Vec< f64 >,
  #[ serde( rename = "y" ) ]
  y : Vec< f64 >,
}

#[ derive( Debug, Deserialize ) ]
struct LottieColor
{
  #[ serde( rename = "k" ) ]
  value : LottieValue< Vec< f64 > >, // RGBA values ( 0-1 )
}

#[ derive( Debug, Deserialize ) ]
struct LottiePoint
{
  #[ serde( rename = "k" ) ]
  value : LottieValue< Vec< f64 > >, // [ x, y ]
}

#[ derive( Debug, Deserialize ) ]
struct LottieScalar
{
  #[ serde( rename = "k" ) ]
  value : LottieValue< f64 >,
}

#[ derive( Debug, Deserialize ) ]
struct LottieVec2
{
  #[ serde( rename = "k" ) ]
  value : LottieValue< Vec< f64 > >, // [ width, height ]
}

#[ derive( Debug, Deserialize ) ]
struct LottieTransform
{
  #[ serde( rename = "a" ) ]
  anchor_point : Option< LottiePoint >, // Anchor Point
  #[ serde( rename = "p" ) ]
  position : Option< LottiePoint >, // Position
  #[ serde( rename = "px" ) ]
  position_x : Option< LottieScalar >, // Position X ( split )
  #[ serde( rename = "py" ) ]
  position_y : Option< LottieScalar >, // Position Y ( split )
  #[ serde( rename = "s" ) ]
  scale : Option< LottieVec2 >, // Scale
  #[ serde( rename = "r" ) ]
  rotation : Option< LottieScalar >, // Rotation
  #[ serde( rename = "sk" ) ]
  skew : Option< LottieScalar >, // Skew
  #[ serde( rename = "sa" ) ]
  skew_angle : Option< LottieScalar >, // Skew Angle
  #[ serde( rename = "o" ) ]
  opacity : Option< LottieScalar >, // Opacity
}

#[ derive( Debug, Deserialize ) ]
struct LottiePathData
{
  #[ serde( rename = "c" ) ]
  is_closed : Option< bool >, // Is closed path
  #[ serde( rename = "v" ) ]
  vertices : Vec< Vec< f64 > >, // Vertices
  #[ serde( rename = "i" ) ]
  in_tangents : Vec< Vec< f64 > >, // In tangents
  #[ serde( rename = "o" ) ]
  out_tangents : Vec< Vec< f64 > >, // Out tangents
}

#[ derive( Debug, Deserialize ) ]
struct LottiePath
{
  #[ serde( rename = "k" ) ]
  value : LottieValue< LottiePathData >,
}

#[ derive( Debug, Deserialize ) ]
struct LottieRect
{
  #[ serde( rename = "p" ) ]
  position : LottiePoint,
  #[ serde( rename = "s" ) ]
  size : LottieVec2,
  #[ serde( rename = "r" ) ]
  corner_radius : LottieScalar,
  #[ serde( rename = "d" ) ]
  direction : Option< f64 >, // 1 for CW, 3 for CCW
}

#[ derive( Debug, Deserialize ) ]
struct LottieEllipse
{
  #[ serde( rename = "p" ) ]
  position : LottiePoint,
  #[ serde( rename = "s" ) ]
  size : LottieVec2,
  #[ serde( rename = "d" ) ]
  direction : Option< f64 >, // 1 for CW, 3 for CCW
}

#[ derive( Debug, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
enum LottieShapeContent
{
  Gr( LottieGroup ),
  Sh( LottiePath ),      // Path
  Rc( LottieRect ),      // Rectangle
  El( LottieEllipse ),   // Ellipse
  Fl( LottieFill ),      // Fill
  St( LottieStroke ),    // Stroke
  Mm( LottieMerge ),     // Merge Paths
  Tr( LottieTransform ), // Transform ( for group )
  Rp( LottieRepeater ),  // Repeater
  // Add more as needed: Gd ( Gradient Fill ), Gs ( Gradient Stroke ), etc.
}

#[ derive( Debug, Deserialize ) ]
struct LottieGroup
{
  #[ serde( rename = "it" ) ]
  items : Vec< LottieShapeContent >,
  #[ serde( rename = "nm" ) ]
  name : Option< String >,
  #[ serde( rename = "ty" ) ]
  #[ allow( dead_code ) ]
  type_name : String, // "gr"
}

#[ derive( Debug, Deserialize ) ]
struct LottieFill
{
  #[ serde( rename = "c" ) ]
  color : LottieColor,
  #[ serde( rename = "o" ) ]
  opacity : LottieScalar,
  #[ serde( rename = "r" ) ]
  #[ allow( dead_code ) ]
  rule : Option< f64 >, // Fill rule ( 1 = NonZero, 2 = EvenOdd )
  #[ serde( rename = "nm" ) ]
  name : Option< String >,
}

#[ derive( Debug, Deserialize ) ]
struct LottieStroke
{
  #[ serde( rename = "c" ) ]
  color : LottieColor,
  #[ serde( rename = "o" ) ]
  opacity : LottieScalar,
  #[ serde( rename = "w" ) ]
  width : LottieScalar,
  #[ serde( rename = "lc" ) ]
  line_cap : Option< f64 >, // 1 = Butt, 2 = Round, 3 = Square
  #[ serde( rename = "lj" ) ]
  line_join : Option< f64 >, // 1 = Miter, 2 = Round, 3 = Bevel
  #[ serde( rename = "ml" ) ]
  miter_limit : Option< f64 >,
  #[ serde( rename = "nm" ) ]
  name : Option< String >,
  // TODO: Dash properties ( d )
}

#[ derive( Debug, Deserialize ) ]
struct LottieRepeater
{
  #[ serde( rename = "c" ) ]
  copies : LottieScalar,
  #[ serde( rename = "o" ) ]
  offset : LottieScalar,
  #[ serde( rename = "tr" ) ]
  transform : LottieTransform,
  #[ serde( rename = "nm" ) ]
  name : Option< String >,
}

#[ derive( Debug, Deserialize ) ]
struct LottieMerge
{
  #[ serde( rename = "mm" ) ]
  mode : f64, // 1 = Add, 2 = Subtract, 3 = Intersect, 4 = Exclude, 5 = Merge
  #[ serde( rename = "nm" ) ]
  name : Option< String >,
}

#[ derive( Debug, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
enum LottieLayerContent
{
  #[ serde( rename = "sh" ) ]
  Shape( LottieLayerBase ), // Shape layer
  #[ serde( rename = "precomp" ) ]
  PreComp( LottieLayerBase ), // Precomposition layer
  // Add other layer types as needed: solid, image, text, etc.
}

#[ derive( Debug, Deserialize ) ]
struct LottieLayerBase
{
  #[ serde( flatten ) ]
  base : LottieLayerCommon,
  #[ serde( rename = "shapes" ) ]
  shapes : Option< Vec< LottieShapeContent > >, // For shape layers
  #[ serde( rename = "refId" ) ]
  reference_id : Option< String >, // For precomposition layers
  #[ serde( rename = "tm" ) ]
  time_remap : Option< LottieScalar >, // For precomposition layers
}

#[ derive( Debug, Deserialize ) ]
struct LottieLayerCommon
{
  #[ serde( rename = "nm" ) ]
  name : String, // Name
  #[ serde( rename = "ty" ) ]
  layer_type : f64, // 0: Precomp, 1: Solid, 2: Image, 3: Null, 4: Shape, 5: Text, 6: Audio, 7: Video, 8: Placeholder, 9: Camera
  #[ serde( rename = "ind" ) ]
  index : usize, // Index
  #[ serde( rename = "parent" ) ]
  parent_index : Option< usize >, // Parent index
  #[ serde( rename = "sr" ) ]
  stretch : Option< f64 >, // Time Stretch
  #[ serde( rename = "st" ) ]
  start_time : Option< f64 >, // Start Time ( in frames )
  #[ serde( rename = "ip" ) ]
  in_point : f64, // In Point ( frame )
  #[ serde( rename = "op" ) ]
  out_point : f64, // Out Point ( frame )
  #[ serde( rename = "ks" ) ]
  transform : LottieTransform, // Transform properties
  #[ serde( rename = "bm" ) ]
  blend_mode : Option< f64 >, // Blend Mode ( 0: Normal, 1: Multiply, etc. )
  #[ serde( rename = "w" ) ]
  width : Option< f64 >, // Width ( for solid/precomp )
  #[ serde( rename = "h" ) ]
  height : Option< f64 >, // Height ( for solid/precomp )
  #[ serde( rename = "hasMatte" ) ] // Corrected field name
  has_matte : Option< bool >, // If layer has a matte
  #[ serde( rename = "td" ) ]
  matte_target : Option< f64 >, // Matte target ( 1=alpha, 2=alpha inverted, 3=luma, 4=luma inverted )
  #[ serde( rename = "cl" ) ]
  class_name : Option< String >, // Class name ( for web )
  #[ serde( rename = "ln" ) ]
  layer_html_id : Option< String >, // HTML ID ( for web )
  #[ serde( rename = "masksProperties" ) ]
  masks_properties : Option< Vec< LottieMask > >, // Masks
  #[ serde( rename = "hd" ) ]
  hidden : Option< bool >, // Hidden layer
}

#[ derive( Debug, Deserialize ) ]
struct LottieMask
{
  #[ serde( rename = "mode" ) ]
  mode : String, // 'a' ( add ), 's' ( subtract ), 'i' ( intersect ), 'l' ( lighten ), 'd' ( darken )
  #[ serde( rename = "pt" ) ]
  path : LottiePath,
  #[ serde( rename = "o" ) ]
  opacity : LottieScalar,
  #[ serde( rename = "inv" ) ]
  inverted : Option< bool >,
}

#[ derive( Debug, Deserialize ) ]
struct LottieAsset
{
  #[ serde( rename = "id" ) ]
  id : String,
  #[ serde( rename = "layers" ) ]
  layers : Option< Vec< LottieLayerContent > >,
  // Add other asset types ( images, fonts ) if needed
}

#[ derive( Debug, Deserialize ) ]
struct LottieFile
{
  #[ serde( rename = "v" ) ]
  #[ allow( dead_code ) ]
  version : String,
  #[ serde( rename = "fr" ) ]
  frame_rate : f64,
  #[ serde( rename = "ip" ) ]
  in_point : f64,
  #[ serde( rename = "op" ) ]
  out_point : f64,
  #[ serde( rename = "w" ) ]
  width : f64,
  #[ serde( rename = "h" ) ]
  height : f64, // Corrected from f60
  #[ serde( rename = "assets" ) ]
  assets : Option< Vec< LottieAsset > >,
  #[ serde( rename = "layers" ) ]
  layers : Vec< LottieLayerContent >,
}

// --- Conversion Logic ---

#[ derive( Debug ) ]
pub enum LottieParseError
{
  SerdeError( serde_json::Error ),
  MissingProperty( String ),
  UnsupportedType( String ),
  InvalidValue( String ),
  Custom( String ),
}

impl From< serde_json::Error > for LottieParseError
{
  fn from( err : serde_json::Error ) -> Self
  {
    LottieParseError::SerdeError( err )
  }
}

/// Helper to convert Lottie's `k` property ( single value or keyframes ) to `interpoli::Value`.
fn parse_lottie_value< T, F, U >( lottie_val : LottieValue< T >, map_fn : F ) -> Result< Value< U >, LottieParseError >
where
  F : Fn( T ) -> Result< U, LottieParseError >,
  U : Tween + Copy + PartialEq + Default,
{
  match lottie_val
  {
    LottieValue::Single( val ) => Ok( Value::Fixed( map_fn( val )? ) ),
    LottieValue::Keyframes( keyframes ) =>
    {
      let mut times = Vec::new( );
      let mut values = Vec::new( );

      for kf in keyframes
      {
        let start_val = kf.start_value.ok_or_else( || LottieParseError::MissingProperty( "start_value in keyframe".to_string( ) ) )?;

        times.push( Time
        {
          frame : kf.time,
          in_tangent : kf.easing_in.map( | e | EasingHandle { x : e.x[ 0 ], y : e.y[ 0 ] } ),
          out_tangent : kf.easing_out.map( | e | EasingHandle { x : e.x[ 0 ], y : e.y[ 0 ] } ),
          hold : kf.hold.map_or( false, | h | abs_diff_eq!( h, 1.0 ) ),
        } );
        values.push( map_fn( start_val )? );
      }

      // If there's only one keyframe and it's not a hold, it's effectively a fixed value
      if times.len( ) == 1 && !times[ 0 ].hold
      {
        if let Some( val ) = values.get( 0 )
        {
          return Ok( Value::Fixed( *val ) );
        }
      }

      Ok( Value::Animated( Animated { times, values } ) )
    }
  }
}

fn parse_color( lottie_color : LottieColor ) -> Result< Value< Color >, LottieParseError >
{
  parse_lottie_value
  ( 
    lottie_color.value, 
    | v |
    {
      if v.len( ) >= 4
      {
        Ok
        ( 
          Color::new
          (
            [ v[ 0 ] as f32, v[ 1 ] as f32, v[ 2 ] as f32, v[ 3 ] as f32 ],
          ) 
        )
      }
      else if v.len( ) == 3
      {
        Ok
        ( 
          Color::new
          (
            [ v[ 0 ] as f32, v[ 1 ] as f32, v[ 2 ] as f32, 1.0 ],
          ) 
        )
      }
      else
      {
        Err
        ( 
          LottieParseError::InvalidValue
          ( 
            format!
            (
              "Invalid color array length: {}",
              v.len( )
            ) 
          ) 
        )
      }
    } 
  )
}

fn parse_point( lottie_point : LottiePoint ) -> Result< Value< Point >, LottieParseError >
{
  parse_lottie_value
  ( 
    lottie_point.value, 
    | v |
    {
      if v.len( ) >= 2
      {
        Ok( Point::new( v[ 0 ], v[ 1 ] ) )
      }
      else
      {
        Err
        ( 
          LottieParseError::InvalidValue
          ( 
            format!
            (
              "Invalid point array length: {}",
              v.len( )
            ) 
          ) 
        )
      }
    } 
  )
}

fn parse_scalar( lottie_scalar : LottieScalar ) -> Result< Value< f64 >, LottieParseError >
{
  parse_lottie_value( lottie_scalar.value, Ok )
}

fn parse_vec2( lottie_vec2 : LottieVec2 ) -> Result< Value< Vec2 >, LottieParseError >
{
  parse_lottie_value
  ( 
    lottie_vec2.value, 
    | v |
    {
      if v.len( ) >= 2
      {
        Ok( Vec2::new( v[ 0 ], v[ 1 ] ) )
      }
      else
      {
        Err
        ( 
          LottieParseError::InvalidValue
          ( 
            format!
            (
            "Invalid vec2 array length: {}",
            v.len( )
            ) 
          ) 
        )
      }
    } 
  )
}

fn parse_transform( lottie_transform : LottieTransform ) -> Result< animated::Transform, LottieParseError >
{
  let anchor = lottie_transform
    .anchor_point
    .map_or( Ok( Value::Fixed( Point::ZERO ) ), parse_point )?;
  let position = if let Some( p ) = lottie_transform.position
  {
    interpoli::animated::Position::Value( parse_point( p )? )
  }
  else
  {
    interpoli::animated::Position::SplitValues
    ( 
      (
        lottie_transform.position_x.map_or( Ok( Value::Fixed( 0.0 ) ), parse_scalar )?,
        lottie_transform.position_y.map_or( Ok( Value::Fixed( 0.0 ) ), parse_scalar )?,
      ) 
    )
  };

  let rotation = lottie_transform
    .rotation
    .map_or( Ok( Value::Fixed( 0.0 ) ), parse_scalar )?;
  let scale = lottie_transform
    .scale
    .map_or( Ok( Value::Fixed( Vec2::new( 100.0, 100.0 ) ) ), parse_vec2 )?; // Default scale is 100%
  let skew = lottie_transform
    .skew
    .map_or( Ok( Value::Fixed( 0.0 ) ), parse_scalar )?;
  let skew_angle = lottie_transform
    .skew_angle
    .map_or( Ok( Value::Fixed( 0.0 ) ), parse_scalar )?;

  Ok( animated::Transform
  {
    anchor,
    position,
    rotation,
    scale,
    skew,
    skew_angle,
  } )
}


// Corrected parse_geometry for Spline
fn parse_geometry_from_lottie_path( lottie_path : LottiePath ) -> Result< Geometry, LottieParseError >
{
  match lottie_path.value
  {
    LottieValue::Single( path_data ) =>
    {
      let mut points_for_spline = Vec::new( );
      if !path_data.vertices.is_empty( )
      {
        // Add the first vertex as the starting point
        points_for_spline.push( Point::new( path_data.vertices[ 0 ][ 0 ], path_data.vertices[ 0 ][ 1 ] ) );

        for i in 0..path_data.vertices.len( )
        {
          let v = Point::new( path_data.vertices[ i ][ 0 ], path_data.vertices[ i ][ 1 ] );
          let o = Vec2::new( path_data.out_tangents[ i ][ 0 ], path_data.out_tangents[ i ][ 1 ] );
          let next_i = ( i + 1 ) % path_data.vertices.len( );
          let next_v = Point::new( path_data.vertices[ next_i ][ 0 ], path_data.vertices[ next_i ][ 1 ] );
          let next_in = Vec2::new( path_data.in_tangents[ next_i ][ 0 ], path_data.in_tangents[ next_i ][ 1 ] );

          // These are the control points for a cubic bezier segment
          // P0 = current vertex ( v )
          // P1 = current vertex + out_tangent ( v + o )
          // P2 = next vertex + in_tangent ( next_v + next_in )
          // P3 = next vertex ( next_v )
          // The interpoli spline format expects a flat list of points.
          // We're essentially flattening the bezier control points here.
          // This is a simplification and may not perfectly match `kurbo::PathEl` generation
          // without a more sophisticated bezier curve reconstruction.

          points_for_spline.push( v + o ); // Control point 1
          points_for_spline.push( next_v + next_in ); // Control point 2
          points_for_spline.push( next_v ); // End point of segment ( start of next )
        }
        // Remove the last point if the path is closed, as the first point is already added
        if path_data.is_closed.unwrap_or( false ) && points_for_spline.len( ) > 1
        {
          points_for_spline.pop( );
        }
      }

      Ok
      ( 
        Geometry::Spline
        ( 
          animated::Spline
          {
            is_closed : path_data.is_closed.unwrap_or( false ),
            times : Vec::new( ), // No animation for fixed path
            values : vec![ points_for_spline ], // Single set of points
          } 
        ) 
      )
    }
    LottieValue::Keyframes( keyframes ) =>
    {
      let mut times = Vec::new( );
      let mut values_per_keyframe = Vec::new( );
      let mut is_closed = false; // Assume consistent closed state across keyframes

      for kf in keyframes
      {
        let path_data = kf.start_value.ok_or_else( || LottieParseError::MissingProperty( "start_value in path keyframe".to_string( ) ) )?;
        is_closed = path_data.is_closed.unwrap_or( false ); // Get closed state from first keyframe

        let mut points_for_spline = Vec::new( );
        if !path_data.vertices.is_empty( )
        {
          points_for_spline.push( Point::new( path_data.vertices[ 0 ][ 0 ], path_data.vertices[ 0 ][ 1 ] ) );
          for i in 0..path_data.vertices.len( )
          {
            let v = Point::new( path_data.vertices[ i ][ 0 ], path_data.vertices[ i ][ 1 ] );
            let o = Vec2::new( path_data.out_tangents[ i ][ 0 ], path_data.out_tangents[ i ][ 1 ] );
            let next_i = ( i + 1 ) % path_data.vertices.len( );
            let next_v = Point::new( path_data.vertices[ next_i ][ 0 ], path_data.vertices[ next_i ][ 1 ] );
            let next_in = Vec2::new( path_data.in_tangents[ next_i ][ 0 ], path_data.in_tangents[ next_i ][ 1 ] );

            points_for_spline.push( v + o );
            points_for_spline.push( next_v + next_in );
            points_for_spline.push( next_v );
          }
          if is_closed && points_for_spline.len( ) > 1
          {
            points_for_spline.pop( );
          }
        }

        values_per_keyframe.push( points_for_spline );

        times.push
        ( 
          Time
          {
            frame : kf.time,
            in_tangent : kf.easing_in.map( | e | EasingHandle { x : e.x[ 0 ], y : e.y[ 0 ] } ),
            out_tangent : kf.easing_out.map( | e | EasingHandle { x : e.x[ 0 ], y : e.y[ 0 ] } ),
            hold : kf.hold.map_or( false, | h | abs_diff_eq!( h, 1.0 ) ),
          } 
        );
      }
      Ok
      ( 
        Geometry::Spline
        ( 
          animated::Spline
          {
            is_closed,
            times,
            values : values_per_keyframe,
          } 
        ) 
      )
    }
  }
}


fn parse_rect( lottie_rect : LottieRect ) -> Result< Geometry, LottieParseError >
{
  let position = parse_point( lottie_rect.position )?;
  let size = parse_vec2( lottie_rect.size )?;
  let corner_radius = parse_scalar( lottie_rect.corner_radius )?;
  let is_ccw = lottie_rect.direction.map_or( false, | d | abs_diff_eq!( d, 3.0 ) );

  Ok
  ( 
    Geometry::Rect
    ( 
      animated::Rect
      {
        is_ccw,
        position,
        size,
        corner_radius,
      } 
    ) 
  )
}

fn parse_ellipse( lottie_ellipse : LottieEllipse ) -> Result< Geometry, LottieParseError >
{
  let position = parse_point( lottie_ellipse.position )?;
  let size = parse_vec2( lottie_ellipse.size )?;
  let is_ccw = lottie_ellipse.direction.map_or( false, | d | abs_diff_eq!( d, 3.0 ) );

  Ok
  ( 
    Geometry::Ellipse
    ( 
      animated::Ellipse
      {
        is_ccw,
        position,
        size,
      } 
    ) 
  )
}

fn parse_fill( lottie_fill : LottieFill ) -> Result< Draw, LottieParseError >
{
  let color = parse_color( lottie_fill.color )?;
  let opacity = parse_scalar( lottie_fill.opacity )?;

  Ok
  ( 
    Draw
    {
      stroke : None, // This is a fill, no stroke
      brush : Brush::Animated( animated::Brush::Solid( color ) ), // Use animated brush for consistency
      opacity : opacity.map( | v | v / 100.0 ), // Lottie opacity is 0-100
    } 
  )
}

fn parse_stroke( lottie_stroke : LottieStroke ) -> Result< Draw, LottieParseError >
{
  let color = parse_color( lottie_stroke.color )?;
  let opacity = parse_scalar( lottie_stroke.opacity )?;
  let width = parse_scalar( lottie_stroke.width )?;

  let line_cap = lottie_stroke.line_cap.map_or( kurbo::Cap::Butt, | c | match c as usize
  {
    1 => kurbo::Cap::Butt,
    2 => kurbo::Cap::Round,
    3 => kurbo::Cap::Square,
    _ => kurbo::Cap::Butt,
  } );
  let line_join = lottie_stroke.line_join.map_or( kurbo::Join::Miter, | j | match j as usize
  {
    1 => kurbo::Join::Miter,
    2 => kurbo::Join::Round,
    3 => kurbo::Join::Bevel,
    _ => kurbo::Join::Miter,
  } );

  Ok( Draw
  {
    stroke : Some
    ( 
      Stroke::Animated
      ( 
        animated::Stroke
        {
          width,
          join : line_join,
          miter_limit : lottie_stroke.miter_limit,
          cap : line_cap,
        } 
      ) 
    ),
    brush : Brush::Animated( animated::Brush::Solid( color ) ), // Use animated brush for consistency
    opacity : opacity.map( | v | v / 100.0 ), // Lottie opacity is 0-100
  } )
}

fn parse_repeater( lottie_repeater : LottieRepeater ) -> Result< Repeater, LottieParseError >
{
  let copies = parse_scalar( lottie_repeater.copies )?;
  let offset = parse_scalar( lottie_repeater.offset )?;
  let transform = parse_transform( lottie_repeater.transform )?;

  Ok
  ( 
    Repeater::Animated
    ( 
      animated::Repeater
      {
        copies,
        offset,
        anchor_point : transform.anchor,
        position : match transform.position
        { // Extract Point from animated::Position
          animated::Position::Value( p ) => p,
          animated::Position::SplitValues( ( x, y ) ) =>
          {
            // If split, combine them into a single point value.
            // This is a simplification; ideally, animated::Repeater::position would support split values.
            // For now, we'll make a fixed point from the first frame if animated.
            if x.is_fixed( ) && y.is_fixed( )
            {
              Value::Fixed( Point::new( x.evaluate( 0.0 ), y.evaluate( 0.0 ) ) )
            }
            else
            {
              // This case is more complex and would need to generate keyframes for the combined point
              // For simplicity, we'll just take the first value.
              Value::Fixed( Point::new( x.evaluate( 0.0 ), y.evaluate( 0.0 ) ) )
            }
          }
        },
        rotation : transform.rotation,
        scale : transform.scale,
        start_opacity : Value::Fixed( 1.0 ), // Lottie repeater doesn't have direct start/end opacity for the repeater itself, usually handled by group opacity
        end_opacity : Value::Fixed( 1.0 ),
      } 
    ) 
  )
}

fn parse_shape_content( lottie_shape_content : LottieShapeContent ) -> Result< Shape, LottieParseError >
{
  match lottie_shape_content
  {
    LottieShapeContent::Gr( group ) =>
    {
      let mut shapes = Vec::new( );
      let mut group_transform : Option< GroupTransform > = None;

      for item in group.items
      {
        if let LottieShapeContent::Tr( tr ) = item
        {
          let transform = parse_transform( tr.clone( ) )?.into_model( );
          let opacity = tr.opacity.map_or( Ok( Value::Fixed( 100.0 ) ), parse_scalar )?; // Opacity is 0-100 in Lottie
          group_transform = Some( GroupTransform { transform, opacity : opacity.map( | v | v / 100.0 ) } );
        }
        else
        {
          shapes.push( parse_shape_content( item )? );
        }
      }
      Ok( Shape::Group( shapes, group_transform ) )
    }
    LottieShapeContent::Sh( path ) => parse_geometry_from_lottie_path( path ).map( Shape::Geometry ),
    LottieShapeContent::Rc( rect ) => parse_rect( rect ).map( Shape::Geometry ),
    LottieShapeContent::El( ellipse ) => parse_ellipse( ellipse ).map( Shape::Geometry ),
    LottieShapeContent::Fl( fill ) => parse_fill( fill ).map( Shape::Draw ),
    LottieShapeContent::St( stroke ) => parse_stroke( stroke ).map( Shape::Draw ),
    LottieShapeContent::Tr( _ ) =>
    {
      // Transform for a group is handled within LottieShapeContent::Gr
      Err
      ( 
        LottieParseError::UnsupportedType
        (
          "Standalone transform in shape content ( should be in group )".to_string( ),
        ) 
      )
    }
    LottieShapeContent::Rp( repeater ) => parse_repeater( repeater ).map( Shape::Repeater ),
    LottieShapeContent::Mm( merge ) =>
    {
      // TODO: Implement merge path logic. This is complex as it requires boolean operations on paths.
      // For now, we'll skip it or return an error.
      Err
      ( 
        LottieParseError::UnsupportedType
        ( 
          format!
          (
            "Unsupported Lottie shape content type: Merge ( mode: {} )",
            merge.mode
          ) 
        ) 
      )
    }
  }
}

fn parse_layer( lottie_layer_content : LottieLayerContent ) -> Result< Layer, LottieParseError >
{
  let ( base, shapes, reference_id, time_remap ) = match lottie_layer_content
  {
    LottieLayerContent::Shape( l ) => ( l.base, l.shapes, l.reference_id, l.time_remap ),
    LottieLayerContent::PreComp( l ) => ( l.base, l.shapes, l.reference_id, l.time_remap ),
    // Handle other layer types here if they are supported by interpoli
  };

  let transform = parse_transform( base.transform )?;
  let opacity = transform.opacity.map_or( Value::Fixed( 100.0 ), parse_scalar )?; // Lottie opacity is 0-100
  let opacity = opacity.map( | v | v / 100.0 ); // Convert to 0-1 range

  let blend_mode = base.blend_mode.map
  ( 
    | bm | 
    match bm as usize
    {
      0 => BlendMode::Normal,
      1 => BlendMode::Multiply,
      2 => BlendMode::Screen,
      3 => BlendMode::Overlay,
      4 => BlendMode::Darken,
      5 => BlendMode::Lighten,
      6 => BlendMode::ColorDodge,
      7 => BlendMode::ColorBurn,
      8 => BlendMode::HardLight,
      9 => BlendMode::SoftLight,
      10 => BlendMode::Difference,
      11 => BlendMode::Exclusion,
      12 => BlendMode::Hue,
      13 => BlendMode::Saturation,
      14 => BlendMode::Color,
      15 => BlendMode::Luminosity,
      _ => BlendMode::Normal, // Default to normal for unknown
    } 
  );

  let content = if let Some( s ) = shapes
  {
    let mut interpoli_shapes = Vec::new( );
    for shape_item in s
    {
      interpoli_shapes.push( parse_shape_content( shape_item )? );
    }
    Content::Shape( interpoli_shapes )
  }
  else if let Some( ref_id ) = reference_id
  {
    Content::Instance
    {
      name : ref_id,
      time_remap : time_remap.map_or( Ok( None ), | s | parse_scalar( s ).map( Some ) )?,
    }
  }
  else
  {
    Content::None
  };

  let masks = if let Some( lottie_masks ) = base.masks_properties
  {
    let mut interpoli_masks = Vec::new( );
    for lottie_mask in lottie_masks
    {
      let mode = match lottie_mask.mode.as_str( )
      {
        "a" => BlendMode::Normal, // Add
        "s" => BlendMode::Subtract, // Subtract
        "i" => BlendMode::SourceIn, // Intersect
        // Lottie has 'l' ( Lighten ) and 'd' ( Darken ) for masks, which might not directly map to Peniko blend modes for masks.
        // For simplicity, we'll map them to Normal or SourceIn/Out.
        _ => BlendMode::Normal,
      };
      let geometry = parse_geometry_from_lottie_path( lottie_mask.path )?;
      let mask_opacity = parse_scalar( lottie_mask.opacity )?;

      interpoli_masks.push
      ( 
        Mask
        {
          mode,
          geometry,
          opacity : mask_opacity.map( | v | v / 100.0 ), // Lottie opacity is 0-100
        } 
      );
    }
    interpoli_masks
  }
  else
  {
    Vec::new( )
  };


  Ok
  ( 
    Layer
    {
      name : base.name,
      parent : base.parent_index.map( | idx | idx - 1 ), // Lottie indices are 1-based, convert to 0-based
      transform : transform.into_model( ),
      opacity,
      width : base.width.unwrap_or( 0.0 ),
      height : base.height.unwrap_or( 0.0 ),
      blend_mode,
      frames : base.in_point..base.out_point,
      stretch : base.stretch.unwrap_or( 1.0 ),
      start_frame : base.start_time.unwrap_or( 0.0 ),
      masks,
      is_mask : base.has_matte.unwrap_or( false ), // Simplified: if it has matte, it's a mask layer
      mask_layer : None, // This would require finding the actual matte layer by index
      content,
    } 
  )
}

/// Parses a Lottie JSON string into an `interpoli::Composition`.
pub fn parse_lottie_json( json_str : &str ) -> Result< Composition, LottieParseError >
{
  let lottie_file : LottieFile = serde_json::from_str( json_str )?;

  let mut composition = Composition
  {
    frames : lottie_file.in_point..lottie_file.out_point,
    frame_rate : lottie_file.frame_rate,
    width : lottie_file.width as usize,
    height : lottie_file.height as usize,
    assets : HashMap::new( ),
    layers : Vec::new( ),
  };

  if let Some( assets ) = lottie_file.assets
  {
    for asset in assets
    {
      if let Some( lottie_asset_layers ) = asset.layers
      {
        let mut interpoli_asset_layers = Vec::new( );
        for lottie_layer_content in lottie_asset_layers
        {
          interpoli_asset_layers.push( parse_layer( lottie_layer_content )? );
        }
        composition.assets.insert( asset.id, interpoli_asset_layers );
      }
    }
  }

  for lottie_layer_content in lottie_file.layers
  {
    composition.layers.push( parse_layer( lottie_layer_content )? );
  }

  Ok( composition )
}

// Helper method to convert animated::Transform to interpoli::Transform
impl animated::Transform
{
  pub fn into_model( self ) -> Transform
  {
    if self.is_fixed( )
    {
      Transform::Fixed( self.evaluate( 0.0 ) )
    }
    else
    {
      Transform::Animated( self )
    }
  }
}

// Helper method to convert animated::Repeater to interpoli::Repeater
impl animated::Repeater
{
  pub fn into_model( self ) -> Repeater
  {
    if self.is_fixed( )
    {
      Repeater::Fixed( self.evaluate( 0.0 ) )
    }
    else
    {
      Repeater::Animated( self )
    }
  }
}

// Helper method to convert animated::Stroke to interpoli::Stroke
impl animated::Stroke
{
  pub fn into_model( self ) -> Stroke
  {
    if self.is_fixed( )
    {
      Stroke::Fixed( self.evaluate( 0.0 ) )
    }
    else
    {
      Stroke::Animated( self )
    }
  }
}

// Helper method to convert animated::Brush to interpoli::Brush
impl animated::Brush
{
  pub fn into_model( self ) -> Brush
  {
    if self.is_fixed( )
    {
      Brush::Fixed( self.evaluate( 1.0, 0.0 ) )
    }
    else
    {
      Brush::Animated( self )
    }
  }
}

// Helper method to convert Value< f64 > ( 0-100 ) to Value< f64 > ( 0-1 )
impl Value< f64 >
{
  fn map< F >( self, f : F ) -> Self
  where
    F : Fn( f64 ) -> f64,
  {
    match self
    {
      Value::Fixed( v ) => Value::Fixed( f( v ) ),
      Value::Animated( animated ) =>
      {
        let new_values = animated.values.into_iter( ).map( f ).collect( );
        Value::Animated( Animated { times : animated.times, values : new_values } )
      }
    }
  }
}


// Example Usage ( in main.rs )
#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_parse_simple_lottie( )
  {
    let lottie_json = r#"
    {
      "v" : "5.7.4",
      "fr" : 30,
      "ip" : 0,
      "op" : 60,
      "w" : 500,
      "h" : 300,
      "assets" : [],
      "layers" : 
      [
        {
          "nm" : "Rectangle Layer 1",
          "ty" : 4,
          "ind" : 1,
          "parent" : null,
          "sr" : 1,
          "st" : 0,
          "ip" : 0,
          "op" : 60,
          "ks" : 
          {
            "a" : { "k" : [ 250, 150 ] },
            "p" : { "k" : [ 250, 150 ] },
            "s" : { "k" : [ 100, 100 ] },
            "r" : { "k" : 0 },
            "o" : { "k" : 100 }
          },
          "shapes": 
          [
            {
              "ty" : "rc",
              "nm" : "Rectangle",
              "p" : { "k" : [ 0, 0 ] },
              "s" : { "k" : [ 100, 100 ] },
              "r" : { "k" : 10 }
            },
            {
              "ty" : "fl",
              "nm" : "Fill 1",
              "c" : { "k" : [ 1, 0, 0, 1 ] },
              "o" : { "k" : 100 }
            }
          ]
        }
      ]
    }
    "#;

    let composition = parse_lottie_json( lottie_json ).expect( "Failed to parse Lottie JSON" );

    assert_eq!( composition.frame_rate, 30.0 );
    assert_eq!( composition.width, 500 );
    assert_eq!( composition.height, 300 );
    assert_eq!( composition.layers.len( ), 1 );

    let layer = &composition.layers[ 0 ];
    assert_eq!( layer.name, "Rectangle Layer 1" );
    assert_eq!( layer.frames, 0.0..60.0 );
    assert_eq!( layer.opacity.evaluate( 0.0 ), 1.0 ); // 100% / 100 = 1.0

    if let Content::Shape( shapes ) = &layer.content
    {
      assert_eq!( shapes.len( ), 2 );

      // Check Rectangle Geometry
      if let Shape::Geometry( Geometry::Rect( rect ) ) = &shapes[ 0 ]
      {
        assert_eq!( rect.position.evaluate( 0.0 ), Point::new( 0.0, 0.0 ) );
        assert_eq!( rect.size.evaluate( 0.0 ), Vec2::new( 100.0, 100.0 ) );
        assert_eq!( rect.corner_radius.evaluate( 0.0 ), 10.0 );
      }
      else
      {
        panic!( "Expected Rectangle Geometry" );
      }

      // Check Fill Draw
      if let Shape::Draw( draw ) = &shapes[ 1 ]
      {
        assert!( draw.stroke.is_none( ) );
        if let Brush::Animated( animated_brush ) = &draw.brush
        {
          if let animated::Brush::Solid( color_value ) = animated_brush
          {
            assert_eq!( color_value.evaluate( 0.0 ), Color::new( [ 1.0, 0.0, 0.0, 1.0 ] ) );
          }
          else
          {
            panic!( "Expected Solid Brush inside Animated Brush" );
          }
        }
        else
        {
          panic!( "Expected Animated Brush" );
        }
        assert_eq!( draw.opacity.evaluate( 0.0 ), 1.0 ); // 100% / 100 = 1.0
      }
      else
      {
        panic!( "Expected Draw ( Fill )" );
      }
    }
    else
    {
      panic!( "Expected Shape Content" );
    }
  }

  #[ test ]
  fn test_parse_animated_property( )
  {
    let lottie_json = r#"
    {
      "v" : "5.7.4",
      "fr" : 30,
      "ip" : 0,
      "op" : 60,
      "w" : 100,
      "h" : 100,
      "assets": [],
      "layers": 
      [
        {
          "nm" : "Animated Layer",
          "ty" : 4,
          "ind" : 1,
          "ip" : 0,
          "op" : 60,
          "ks" : 
          {
            "a" : { "k": [ 50, 50 ] },
            "p" : 
            {
              "k" : 
              [
                { "t" : 0, "s" : [ 50, 50 ], "e" : [ 100, 100 ] },
                { "t" : 30, "s" : [ 100, 100 ], "e" : [ 50, 50 ] }
              ]
            },
            "s" : { "k" : [ 100, 100 ] },
            "r" : { "k" : 0 },
            "o" : { "k" : 100 }
          },
          "shapes": []
        }
      ]
    }
    "#;

    let composition = parse_lottie_json( lottie_json ).expect( "Failed to parse Lottie JSON" );
    let layer = &composition.layers[ 0 ];

    if let Transform::Animated( animated_transform ) = &layer.transform
    {
      if let interpoli::animated::Position::Value( Value::Animated( animated_value ) ) = &animated_transform.position
      {
        assert_eq!( animated_value.times.len( ), 2 );
        assert_eq!( animated_value.values.len( ), 2 );
        assert_eq!( animated_value.values[ 0 ], Point::new( 50.0, 50.0 ) );
        assert_eq!( animated_value.values[ 1 ], Point::new( 100.0, 100.0 ) );
        assert_eq!( animated_value.times[ 0 ].frame, 0.0 );
        assert_eq!( animated_value.times[ 1 ].frame, 30.0 );
      }
      else
      {
        panic!( "Expected animated position" );
      }
    }
    else
    {
      panic!( "Expected animated transform" );
    }
  }
}
