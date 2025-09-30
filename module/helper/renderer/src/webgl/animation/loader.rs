mod private
{
  use std::
  {
    cell::RefCell,
    collections::HashMap,
    rc::Rc
  };
  use crate::webgl::
  {
    Node,
    animation::
    {
      base::
      {
        TRANSLATION_PREFIX,
        ROTATION_PREFIX,
        SCALE_PREFIX,
        MORPH_TARGET_PREFIX
      },
      Animation
    }
  };
  use animation::
  {
    Sequence,
    Sequencer,
    Tween,
    easing::
    {
      EasingBuilder,
      EasingFunction,
      Squad,
      Step,
      Linear,
      cubic::CubicHermite
    }
  };
  use gltf::
  {
    animation::
    {
      util::ReadOutputs, Channel, Interpolation, Property
    },
    Gltf
  };
  use mingl::{ F64x3, QuatF64 };

  async fn decode_channel< 'a >
  (
    channel : Channel< '_ >,
    buffers : &'a [ Vec< u8 > ],
  ) -> Option< ( usize, Vec< f64 >, ReadOutputs< 'a > ) >
  {
    let sampler = channel.sampler();
    let reader = channel.reader
    (
      | buffer | Some( buffers[ buffer.index() ].as_slice() )
    );

    let Some( times ) = reader.read_inputs()
    else
    {
      return None;
    };

    let Some( values ) = reader.read_outputs()
    else
    {
      return None;
    };

    let components = if let Interpolation::CubicSpline = sampler.interpolation()
    {
      3
    }
    else
    {
      1
    };

    Some
    (
      (
        components,
        times.map( | t | t as f64 ).collect::< Vec< _ > >(),
        values
      )
    )
  }

  async fn quat_sequence
  (
    channel : Channel< '_ >,
    buffers : &[ Vec< u8 > ],
  ) -> Option< Sequence< Tween< QuatF64 > > >
  {
    let Some
    (
      ( components, times, values )
    )
    = decode_channel( channel.clone(), buffers ).await
    else
    {
      return None;
    };

    let ReadOutputs::Rotations( rotations ) = values
    else
    {
      return None;
    };
    let rotations = rotations.into_f32().collect::< Vec< _ > >();

    let iter = times.into_iter()
    .zip( rotations.chunks( components ) );

    let mut tweens = vec![];
    let mut last_time = None;
    let mut last_value = None;

    for ( t2, v ) in iter
    {
      let mut items_iter = v.iter();

      let mut in_tangent = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _in_tangent ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        in_tangent = Some( _in_tangent.map( | v | v as f64 ) );
      }

      let Some( value ) = items_iter.next().cloned()
      else
      {
        continue;
      };

      let mut out_tangent = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _out_tangent ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        out_tangent = Some( _out_tangent.map( | v | v as f64 ) );
      }

      let r2 = QuatF64::from( value.map( | v | v as f64 ) );

      let r1 = last_value.unwrap_or( r2 );
      let t1 = last_time.unwrap_or( t2 );

      let easing : Box< dyn EasingFunction< AnimatableType = QuatF64 > > = match channel.sampler().interpolation()
      {
        Interpolation::Linear => Linear::new(),
        Interpolation::Step => Box::new( Step::new( 1.0 ) ),
        Interpolation::CubicSpline =>
        {
          let in_tangent = QuatF64::from( in_tangent.unwrap() );
          let out_tangent = QuatF64::from( out_tangent.unwrap() );
          Box::new( Squad::new( in_tangent, out_tangent ) )
        },
      };

      last_time = Some( t2 );
      last_value = Some( r2 );
      let duration = t2 - t1;
      let delay = t1;

      let tween = Tween::new( r1, r2, duration.into(), easing )
      .with_delay( delay.into() );
      tweens.push( tween );
    }

    Sequence::new( tweens ).ok()
  }

  async fn vec3_sequence
  (
    channel : Channel< '_ >,
    buffers : &[ Vec< u8 > ],
  ) -> Option< Sequence< Tween< F64x3 > > >
  {
    let Some
    (
      ( components, times, values )
    )
    = decode_channel( channel.clone(), buffers ).await
    else
    {
      return None;
    };

    let values = match values
    {
      ReadOutputs::Translations( v ) |
      ReadOutputs::Scales( v ) =>
      {
        v.collect::< Vec< _ > >()
      }
      _ => { return None; }
    };

    let iter = times.into_iter()
    .zip( values.chunks( components ) );

    let mut tweens = vec![];
    let mut last_time = None;
    let mut last_value = None;

    for ( t2, v ) in iter
    {
      let mut items_iter = v.iter();

      let mut m1 = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _m1 ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        m1 = Some( _m1.map( | v | v as f64 ) );
      }

      let Some( v2 ) = items_iter.next().cloned()
      else
      {
        continue;
      };

      let mut m2 = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _m2 ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        m2 = Some( _m2.map( | v | v as f64 ) );
      }

      let v2 = F64x3::from_array( v2.map( | v | v as f64 ) );
      let t1 = last_time.unwrap_or( t2 );
      let v1 = last_value
      .unwrap_or( v2 );

      let easing : Box< dyn EasingFunction< AnimatableType = F64x3 > > = match channel.sampler().interpolation()
      {
        Interpolation::Linear => Linear::new(),
        Interpolation::Step => Box::new( Step::new( 1.0 ) ),
        Interpolation::CubicSpline =>
        {
          let m1 = F64x3::from_array( m1.unwrap() );
          let m2 = F64x3::from_array( m2.unwrap() );
          Box::new( CubicHermite::new( m1, m2 ) )
        },
      };

      last_time = Some( t2 );
      last_value = Some( v2 );
      let duration = t2 - t1;
      let delay = t1;
      let tween = Tween::new( v1, v2, duration.into(), easing )
      .with_delay( delay.into() );
      tweens.push( tween );
    }

    Sequence::new( tweens ).ok()
  }

  async fn weights_sequence
  (
    channel : Channel< '_ >,
    buffers : &[ Vec< u8 > ],
  )
  -> Option< Sequence< Tween< f64 > >>
  {
    let Some
    (
      ( components, times, values )
    )
    = decode_channel( channel.clone(), buffers ).await
    else
    {
      return None;
    };

    let ReadOutputs::MorphTargetWeights( weights ) = values
    else
    {
      return None;
    };

    let weights = weights.into_f32().collect::< Vec< _ > >();

    let iter = times.into_iter()
    .zip( weights.chunks( components ) );

    let mut tweens = vec![];
    let mut last_time = None;
    let mut last_value: Option< f64 > = None;

    for ( t2, v ) in iter
    {
      let mut items_iter = v.iter();

      let mut m1 = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _m1 ) = items_iter.next().cloned()
        else
        {
          continue
        };
        m1 = Some( _m1 as f64 );
      }

      let Some( v2 ) = items_iter.next().cloned().map( | v | v as f64 )
      else
      {
        continue;
      };

      let mut m2 = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _m2 ) = items_iter.next().cloned()
        else
        {
          continue
        };
        m2 = Some( _m2 );
      }

      let v1 = last_value.clone().unwrap_or( v2 );
      let t1 = last_time.unwrap_or( t2 );

      let easing : Box< dyn EasingFunction< AnimatableType = f64 > > = match channel.sampler().interpolation()
      {
        Interpolation::Linear => Linear::new(),
        Interpolation::Step => Box::new( Step::new( 1.0 ) ),
        Interpolation::CubicSpline => Box::new( CubicHermite::new( m1.unwrap(), m2.unwrap() ) )
      };

      last_time = Some( t2 );
      last_value = Some( v2.clone() );
      let duration = t2 - t1;
      let delay = t1;

      let tween = Tween::new( v1, v2, duration.into(), easing )
      .with_delay( delay.into() );
      tweens.push( tween );
    }

    Sequence::new( tweens ).ok()
  }

  /// Load all animations from [`Gltf`] file
  pub async fn load
  (
    gltf_file : &Gltf,
    buffers : &[ Vec< u8 > ],
    nodes : &[ Rc< RefCell< Node > > ]
  )
  -> Vec< Animation >
  {
    let mut animations = Vec::new();
    for animation in gltf_file.animations()
    {
      let mut animated_nodes = HashMap::new();
      let mut sequencer = Sequencer::new();

      for channel in animation.channels()
      {
        let node = nodes[ channel.target().node().index() ].clone();
        let Some( name ) = node.borrow().get_name()
        else
        {
          continue;
        };

        animated_nodes.insert( name.clone(), node );

        match channel.target().property()
        {
          Property::Translation =>
          {
            let Some( sequence ) = vec3_sequence( channel, buffers ).await
            else
            {
              continue;
            };
            sequencer.add( &format!( "{}{}", name, TRANSLATION_PREFIX ), sequence );
          },
          Property::Scale =>
          {
            let Some( sequence ) = vec3_sequence( channel, buffers ).await
            else
            {
              continue;
            };
            sequencer.add( &format!( "{}{}", name, SCALE_PREFIX ), sequence );
          }
          Property::Rotation =>
          {
            let Some( sequence ) = quat_sequence( channel, buffers ).await
            else
            {
              continue;
            };
            sequencer.add( &format!( "{}{}", name, ROTATION_PREFIX ), sequence );
          },
          Property::MorphTargetWeights =>
          {
            let Some( sequence ) = weights_sequence( channel, buffers ).await
            else
            {
              continue;
            };
            sequencer.add( &format!( "{}{}", name, MORPH_TARGET_PREFIX ), sequence );
          }
        };
      }

      let animation = Animation::new
      (
        animation.name().map( | s | s.into() ),
        Box::new( sequencer ),
        animated_nodes
      );

      animations.push( animation );
    }

    animations
  }
}

crate::mod_interface!
{
  own use
  {
    load
  };
}
