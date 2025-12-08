
#![ allow(dead_code ) ]

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

  /// Creates a fixed `Value` that holds a single, constant value.
  ///
  /// This is a convenience function for constructing a non-animated `Value`.
  ///
  /// # Arguments
  ///
  /// * `value` - The value to be held. It must implement the `interpoli::Tween` trait.
  ///
  /// # Returns
  ///
  /// A `Value::Fixed` containing the provided value.
  pub fn fixed< T : interpoli::Tween >( value : T ) -> Value< T >
  {
    Value::Fixed( value )
  }

  /// Creates an animated `Value` from a vector of keyframe data.
  ///
  /// Each tuple in the input vector represents a keyframe with its frame time,
  /// optional in/out tangents for easing, a hold flag, and the keyframe value itself.
  ///
  /// # Arguments
  ///
  /// * `values` - A vector of tuples, each defining a keyframe: `( frame, [ in_tangent, out_tangent ], hold, value )`.
  ///
  /// # Returns
  ///
  /// A `Value::Animated` containing the animated data.
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

  /// Represents the transform of an element, including position, rotation, and scale.
  ///
  /// This struct uses `Value<T>` for each property, allowing for both fixed and animated transforms.
  #[ derive( Debug, Clone, Former ) ]
  pub struct Transform
  {
    /// The anchor point of the transform.
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    pub anchor : Value< kurbo::Point >,
    /// The position of the element.
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    pub position : Value< Point >,
    /// The rotation of the element in radians.
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    pub rotation : Value< f64 >,
    /// The scale of the element.
    #[ former( default = Value::Fixed( kurbo::Vec2::new( 100.0, 100.0 ) ) ) ]
    pub scale : Value< Vec2 >,
    /// The skew of the element.
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    pub skew : Value< f64 >,
    /// The skew angle of the element.
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    pub skew_angle : Value< f64 >,
  }

  /// Converts a `Transform` into an `interpoli::animated::Transform`.
  impl Into< interpoli::animated::Transform > for Transform
  {
    fn into( self ) -> interpoli::animated::Transform
    {
      let Transform
      {
        anchor,
        position,
        rotation,
        scale,
        skew,
        skew_angle,
      }
      = self;

      interpoli::animated::Transform
      {
        anchor,
        position : interpoli::animated::Position::Value( position ),
        rotation,
        scale,
        skew,
        skew_angle
      }
    }
  }

  /// Defines a repeater, which duplicates a layer's content.
  ///
  /// This struct controls the number of copies and their properties like offset, transform, and opacity.
  #[ derive( Debug, Clone, Former ) ]
  pub struct Repeater
  {
    /// The number of copies to create.
    #[ former( default = 0.0 ) ]
    pub copies: f64,
    /// The offset of each copy.
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    pub offset: Value< f64 >,
    /// The anchor point for the repeater's transform.
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    pub anchor_point: Value< Point >,
    /// The position of each copy.
    #[ former( default = Value::Fixed( kurbo::Point::new( 0.0, 0.0 ) ) ) ]
    pub position: Value< Point >,
    /// The rotation of each copy.
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    pub rotation: Value< f64 >,
    /// The scale of each copy.
    #[ former( default = Value::Fixed( kurbo::Vec2::new( 100.0, 100.0 ) ) ) ]
    pub scale: Value< Vec2 >,
    /// The starting opacity of the copies.
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    pub start_opacity: Value< f64 >,
    /// The ending opacity of the copies.
    #[ former( default = Value::Fixed( 0.0 ) ) ]
    pub end_opacity: Value< f64 >,
  }

  /// Converts a `Repeater` into an `interpoli::animated::Repeater`.
  impl Into< interpoli::animated::Repeater > for Repeater
  {
    fn into( self ) -> interpoli::animated::Repeater
    {
      interpoli::animated::Repeater
      {
        copies : Value::Fixed( self.copies ),
        offset : self.offset.clone(),
        anchor_point : self.anchor_point.clone(),
        position : self.position.clone(),
        rotation : self.rotation.clone(),
        scale : self.scale.clone(),
        start_opacity : self.start_opacity.clone(),
        end_opacity : self.end_opacity.clone()
      }
    }
  }

  /// Represents a color, which can be either fixed or animated.
  #[ allow( dead_code ) ]
  #[ derive( Debug, Clone ) ]
  pub enum Color
  {
    /// A fixed color value as a 4-element f32 array (RGBA).
    Fixed( [ f32; 4 ] ),
    /// An animated color value.
    Animated( Value< peniko::Color > )
  }

  /// Represents a shape, which can be a stroke, color, geometry, spline, or repeater.
  #[ allow( dead_code ) ]
  #[ derive( Debug, Clone ) ]
  pub enum Shape
  {
    /// The stroke width of the shape.
    Stroke( Option< f64 > ),
    /// The color of the shape.
    Color( Color ),
    /// A path defined by a series of points.
    Geometry( Vec< [ f32; 2 ] > ),
    /// A smooth curve defined by a series of points.
    Spline
    {
      /// The points that define the spline.
      path : Vec< [ f32; 2 ] >,
      /// Whether the spline should form a closed loop.
      is_closed : bool
    },
    /// A repeater that duplicates the previous shapes.
    Repeater( interpoli::Repeater )
  }

  impl Shape
  {
    /// Converts a vector of `Shape` enum variants into an `interpoli::Content` object.
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

  /// Represents a single layer in a composition, with properties like parent, frame range, transform, and content.
  #[ derive( Debug, Clone, Former ) ]
  pub struct Layer
  {
    /// The index of the parent layer. A value of -1 indicates no parent.
    #[ former( default = -1_isize ) ]
    pub parent : isize,
    /// The frame range during which the layer is active.
    #[ former( default = 0.0..0.0 ) ]
    pub frames : Range< f64 >,
    /// The starting frame of the layer.
    #[ former( default = 0.0 ) ]
    pub start_frame : f64,
    /// The layer's transform.
    #[ former( default = interpoli::Transform::Fixed( kurbo::Affine::IDENTITY ) ) ]
    pub transform : interpoli::Transform,
    /// The content of the layer, defined by a vector of `Shape`s.
    #[ subform_collection ]
    #[ former( default = vec![] ) ]
    pub content : Vec< Shape >
  }

  /// Converts a `Layer` into an `interpoli::Layer`.
  impl Into< interpoli::Layer > for Layer
  {
    fn into( self ) -> interpoli::Layer
    {
      let parent = if self.parent == -1
      {
        None
      }
      else
      {
        Some( self.parent as usize )
      };

      interpoli::Layer
      {
        name : String::new(),
        parent,
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

  /// Represents the entire composition, including its dimensions, frame range, and layers.
  #[ derive( Debug, Former ) ]
  pub struct Model
  {
    /// The width of the composition.
    #[ former( default = 1920_usize ) ]
    pub width : usize,
    /// The height of the composition.
    #[ former( default = 1080_usize ) ]
    pub height : usize,
    /// The frame range of the composition.
    #[ former( default = 0.0..0.0 ) ]
    pub frames : Range< f64 >,
    /// A collection of layers that make up the composition.
    #[ former( default = vec![] ) ]
    #[ subform_collection ]
    pub layers : Vec< Layer >,
  }

  /// Converts a `Model` into an `interpoli::Composition`.
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
