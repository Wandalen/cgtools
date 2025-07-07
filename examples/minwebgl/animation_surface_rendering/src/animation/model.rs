mod private
{
  use former::Former;
  use interpoli::
  {
    Composition,
    Value,
    EasingHandle,
    Content,
    Stroke
  };
  use kurbo::
  {
    Point,
    Vec2
  };
  use crate::primitive::points_to_path;
  use std::ops::Range;

  pub fn fixed< T : interpoli::Tween >( value : T ) -> Value< T >
  {
    Value::Fixed( value )
  }

  pub fn animated< T : interpoli::Tween >
  ( 
    values : Vec< ( f64, [ Option< [ f64; 2 ] >; 2 ], bool, T ) > 
  ) -> Value< T >
  {
    let _values = values.iter()
    .map( | ( _, _, _, v ) | v )
    .cloned()
    .collect::< Vec< _ > >();

    let times = values.into_iter()
    .map
    ( 
      | ( frame, [ in_tangent, out_tangent ], hold, _ ) | 
      {
        interpoli::Time
        {
          frame,
          in_tangent : in_tangent.map( | [ x1, y1 ] | EasingHandle{ x : x1, y : y1 } ), 
          out_tangent : out_tangent.map( | [ x2, y2 ] | EasingHandle{ x : x2, y : y2 } ),
          hold
        }
      } 
    )
    .collect::< Vec< _ > >();

    Value::Animated
    ( 
      interpoli::Animated
      {
        times,
        values : _values
      }
    )
  }

  #[ derive( Debug, Clone, Former ) ]
  pub struct Transform
  {
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    anchor : Value< kurbo::Point >,
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    position : Value< Point >,
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    rotation : Value< f64 >,
    #[ former( default = Value::Fixed( kurbo::Vec2::new( 1.0, 1.0 ) ) ) ]
    scale : Value< Vec2 >,
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    skew : Value< f64 >,
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    skew_angle : Value< f64 >,
  }

  impl Into< interpoli::animated::Transform > for Transform 
  {
    fn into( self ) -> interpoli::animated::Transform 
    {
      interpoli::animated::Transform
      {
        anchor : self.anchor.clone(),
        position : interpoli::animated::Position::Value( self.position.clone() ),
        rotation : self.rotation.clone(),
        scale : self.scale.clone(),
        skew : self.skew.clone(),
        skew_angle : self.skew_angle.clone()
      }
    }
  }

  #[ derive( Debug, Clone, Former ) ]
  pub struct Repeater 
  {
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    copies: Value< f64 >,
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    offset: Value< f64 >,
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    anchor_point: Value< Point >,
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    position: Value< Point >,
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    rotation: Value< f64 >,
    #[ former( default = Value::Fixed( kurbo::Vec2::new( 0.0, 0.0 ) ) ) ]
    scale: Value< Vec2 >,
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    start_opacity: Value< f64 >,
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    end_opacity: Value< f64 >,
  }

  impl Into< interpoli::animated::Repeater > for Repeater 
  {
    fn into( self ) -> interpoli::animated::Repeater 
    {
      interpoli::animated::Repeater
      {
        copies : self.copies.clone(),
        offset : self.offset.clone(),
        anchor_point : self.anchor_point.clone(),
        position : self.position.clone(),
        rotation : self.rotation.clone(),
        scale : self.scale.clone(),
        start_opacity: self.start_opacity.clone(),
        end_opacity: self.end_opacity.clone()
      }
    }
  }
  
  #[ derive( Debug, Clone ) ]
  pub enum Color 
  {
    Fixed( [ f32; 4 ] ),
    Animated( Value< peniko::Color > )
  }

  #[ derive( Debug, Clone ) ]
  pub enum Shape
  {
    Stroke( Option< f64 > ),
    Color( Color ),
    Geometry( Vec< [ f32; 2 ] > ),
    Spline
    {
      path : Vec< [ f32; 2 ] >,
      is_closed : bool
    },
    Repeater( interpoli::Repeater )
  }

  impl Shape
  {
    fn into_content( shapes : Vec< Shape > ) -> Content
    {
      let mut _shapes = vec![];

      let mut stroke = None;
      let mut color = Color::Fixed( [ 0.0; 4 ] );

      for shape in shapes
      {
        let shape : interpoli::Shape = match shape
        {
          Shape::Stroke( _stroke ) => 
          {
            stroke = _stroke;

            let brush = match color
            {
              Color::Fixed( fixed ) =>
              {
                interpoli::Brush::Fixed
                ( 
                  peniko::Brush::Solid
                  ( 
                    color::AlphaColor::< color::Srgb >::new( fixed )
                  ) 
                )
              },
              Color::Animated( ref animated ) =>
              {
                interpoli::Brush::Animated( interpoli::animated::Brush::Solid( animated.clone() ) )
              }
            };

            interpoli::Shape::Draw
            ( 
              interpoli::Draw
              { 
                stroke : stroke.map( | v | Stroke::Fixed( kurbo::Stroke::new( v ) ) ), 
                brush, 
                opacity : interpoli::Value::Fixed( 1.0 ) 
              } 
            )
          },
          Shape::Color( _color ) =>
          {
            color = _color;

            let brush = match color
            {
              Color::Fixed( fixed ) =>
              {
                interpoli::Brush::Fixed
                ( 
                  peniko::Brush::Solid
                  ( 
                    color::AlphaColor::< color::Srgb >::new( fixed )
                  ) 
                )
              },
              Color::Animated( ref animated ) =>
              {
                interpoli::Brush::Animated( interpoli::animated::Brush::Solid( animated.clone() ) )
              }
            };

            interpoli::Shape::Draw
            ( 
              interpoli::Draw
              { 
                stroke : stroke.map( | v | Stroke::Fixed( kurbo::Stroke::new( v ) ) ), 
                brush, 
                opacity : interpoli::Value::Fixed( 1.0 ) 
              } 
            )
          },
          Shape::Geometry( contour ) => 
          {
            interpoli::Shape::Geometry
            ( 
              interpoli::Geometry::Fixed( points_to_path( contour ) )
            )
          },
          Shape::Spline { path, is_closed } => 
          {
            interpoli::Shape::Geometry
            ( 
              interpoli::Geometry::Spline
              ( 
                interpoli::animated::Spline
                {
                  is_closed,
                  times : vec![],
                  values : 
                  vec!
                  [ 
                    path.into_iter()
                    .map( | p | kurbo::Point::new( p[ 0 ].into(), p[ 1 ].into() ) )
                    .collect::< Vec< _ > >() 
                  ]
                }
              )
            )
          },
          Shape::Repeater( repeater ) => interpoli::Shape::Repeater( repeater.into() ),
        };

        _shapes.push( shape );
      }

      Content::Shape( _shapes )
    }
  }
  
  #[ derive( Debug, Clone, Former ) ]
  pub struct Layer
  {
    #[ former( default = usize::MAX ) ]
    parent : Option< usize >,
    #[ former( default = 0.0..0.0 ) ]
    frames : Range< f64 >,
    #[ former( default = 0.0 ) ]
    start_frame : f64,
    #[ former( default = interpoli::Transform::Fixed( kurbo::Affine::IDENTITY ) ) ]
    transform : interpoli::Transform,
    #[ subform_collection ]
    #[ former( default = vec![] ) ]
    content : Vec< Shape >
  }

  impl Into< interpoli::Layer > for Layer 
  {
    fn into( self ) -> interpoli::Layer 
    {
      interpoli::Layer 
      {
        name : String::new(),
        parent : self.parent,
        transform : self.transform,
        opacity : Value::Fixed( 1.0 ),
        width : 0.0,
        height : 0.0,
        blend_mode : None,
        frames : self.frames.clone(),
        stretch : 1.0,
        start_frame : self.start_frame,
        masks : vec![],
        is_mask : false,
        mask_layer : None,
        content : Shape::into_content( self.content.clone() )
      }
    }
  }

  #[ derive( Debug, Former ) ]
  pub struct Model
  {
    #[ former( default = 1920_usize ) ]
    width : usize, 
    #[ former( default = 1080_usize ) ]
    height : usize, 
    #[ former( default = 0.0..0.0 ) ]
    frames : Range< f64 >,
    #[ former( default = vec![] ) ]
    #[ subform_collection ]
    layers : Vec< Layer >,
  }

  impl Into< Composition > for Model 
  {
    fn into( self ) -> Composition 
    {
      Composition
      {
        frames : self.frames.clone(),
        frame_rate : 60.0,
        width : self.width,
        height : self.height,
        assets : Default::default(),
        layers : self.layers.clone()
        .into_iter()
        .map( | l | l.into() )
        .collect::< Vec< interpoli::Layer > >()
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    fixed,
    animated,
    Transform,
    Color,
    Repeater,
    Shape,
    Layer,
    Model,
  };
}