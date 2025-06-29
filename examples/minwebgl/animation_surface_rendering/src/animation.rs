// Copyright 2024 the Interpoli Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use interpoli::{ fixed, Composition, Content, Draw, Geometry, GroupTransform, Layer, Shape };
use kurbo::{ Affine, PathEl, Rect };
use peniko::{ Fill, Mix };
use std::ops::Range;

#[ derive( Default ) ]
pub struct Animator
{
  batch : Batch,
  frame_geometry : Vec< geometry_generation::PrimitiveData >,
  colors : Vec< minwebgl::math::F32x4 >,
  mask_elements : Vec< PathEl >,
}

impl Animator
{
  /// Creates a new animator.
  pub fn new() -> Self
  {
    Self::default()
  }

  /// Generates the animation at a given frame
  pub fn generate
  (
    &mut self,
    animation : &Composition,
    frame : f64,
    transform : Affine,
    alpha : f64,
  ) -> Vec< geometry_generation::PrimitiveData >
  {
    self.frame_geometry.clear();
    self.colors.clear();
    self.batch.clear();
    for layer in animation.layers.iter().rev()
    {
      if layer.is_mask
      {
        continue;
      }
      self.generate_layer
      (
        animation,
        &animation.layers,
        layer,
        transform,
        alpha,
        frame,
      );
    }
    
    ( self.frame_geometry.clone(), self.colors.clone() )
  }

  fn generate_layer
  (
    &mut self,
    animation : &Composition,
    layer_set : &[ Layer ],
    layer : &Layer,
    transform : Affine,
    alpha : f64,
    frame : f64,
  )
  {
    if !layer.frames.contains( &frame )
    {
      return;
    }
    let parent_transform = transform;
    let transform = self.compute_transform( layer_set, layer, parent_transform, frame );
    let full_rect = Rect::new( 0.0, 0.0, animation.width as f64, animation.height as f64 );
    if let Some( ( mode, mask_index ) ) = layer.mask_layer
    {
      if let Some( mask ) = layer_set.get( mask_index )
      {
        self.generate_layer
        (
          animation,
          layer_set,
          mask,
          parent_transform,
          alpha,
          frame,
        );
      }
    }
    let alpha = alpha * layer.opacity.evaluate( frame ) / 100.0;
    for mask in &layer.masks
    {
      let alpha = mask.opacity.evaluate( frame ) / 100.0;
      mask.geometry.evaluate( frame, &mut self.mask_elements );
      self.mask_elements.clear();
    }
    match &layer.content
    {
      Content::None => {},
      Content::Instance
      {
        name,
        time_remap : _,
      } =>
      {
        if let Some( asset_layers ) = animation.assets.get( name )
        {
          let frame = frame / layer.stretch;
          let frame_delta = -layer.start_frame / layer.stretch;
          for asset_layer in asset_layers.iter().rev()
          {
            if asset_layer.is_mask
            {
              continue;
            }
            self.generate_layer
            (
              animation,
              asset_layers,
              asset_layer,
              transform,
              alpha,
              frame + frame_delta,
            );
          }
        }
      }
      Content::Shape( shapes ) =>
      {
        self.generate_shapes( shapes, transform, alpha, frame );
        let ( geometry, colors ) = self.batch.generate();
        self.frame_geometry.extend( geometry );
        self.colors.extend( colors );
        self.batch.clear();
      }
    }
  }

  fn generate_shapes( &mut self, shapes : &[ Shape ], transform : Affine, alpha : f64, frame : f64 )
  {
    // Keep track of our local top of the geometry stack. Any subsequent
    // draws are bounded by this.
    let geometry_start = self.batch.geometries.len();
    // Also keep track of top of draw stack for repeater evaluation.
    let draw_start = self.batch.draws.len();
    // Top to bottom, collect geometries and draws.
    for shape in shapes
    {
      match shape
      {
        Shape::Group( shapes, group_transform ) =>
        {
          let ( group_transform, group_alpha ) =
            if let Some( GroupTransform { transform, opacity } ) = group_transform
            {
              (
                transform.evaluate( frame ).into_owned(),
                opacity.evaluate( frame ) / 100.0,
              )
            } 
            else
            {
              ( Affine::IDENTITY, 1.0 )
            };
          self.generate_shapes
          (
            shapes,
            transform * group_transform,
            alpha * group_alpha,
            frame,
          );
        }
        Shape::Geometry( geometry ) =>
        {
          self.batch.push_geometry( geometry, transform, frame );
        }
        Shape::Draw( draw ) =>
        {
          self.batch.push_draw( draw, alpha, geometry_start, frame );
        }
        Shape::Repeater( repeater ) =>
        {
          let repeater = repeater.evaluate( frame );
          self.batch
          .repeat( repeater.as_ref(), geometry_start, draw_start );
        }
      }
    }
  }

  /// Computes the transform for a single layer. This currently chases the
  /// full transform chain each time. If it becomes a bottleneck, we can
  /// implement caching.
  fn compute_transform
  (
    &self,
    layer_set : &[ Layer ],
    layer : &Layer,
    global_transform : Affine,
    frame : f64,
  ) -> Affine
  {
    let mut transform = layer.transform.evaluate( frame ).into_owned();
    let mut parent_index = layer.parent;
    let mut count = 0_usize;
    while let Some( index ) = parent_index
    {
      // We don't check for cycles at import time, so this heuristic
      // prevents infinite loops.
      if count >= layer_set.len()
      {
        break;
      }
      if let Some( parent ) = layer_set.get( index )
      {
        parent_index = parent.parent;
        transform = parent.transform.evaluate( frame ).into_owned() * transform;
        count += 1;
      } 
      else
      {
        break;
      }
    }
    global_transform * transform
  }
}

#[ derive( Clone, Debug ) ]
struct DrawData
{
  stroke : Option< fixed::Stroke >,
  brush : fixed::Brush,
  alpha : f64,
  /// Range into `ShapeBatch::geometries`
  geometry : Range< usize >,
}

impl DrawData
{
  fn new( draw : &Draw, alpha : f64, geometry : Range< usize >, frame : f64 ) -> Self
  {
    Self
    {
      stroke : draw
      .stroke
      .as_ref()
      .map( | stroke | stroke.evaluate( frame ).into_owned() ),
      brush : draw.brush.evaluate( 1.0, frame ).into_owned(),
      alpha : alpha * draw.opacity.evaluate( frame ) / 100.0,
      geometry,
    }
  }
}

#[ derive( Clone, Debug ) ]
struct GeometryData
{
  elements : Range< usize >,
  transform : Affine,
}

#[ derive( Default ) ]
struct Batch
{
  elements : Vec< PathEl >,
  geometries : Vec< GeometryData >,
  draws : Vec< DrawData >,
  repeat_geometries : Vec< GeometryData >,
  repeat_draws : Vec< DrawData >,
  drawn_geometry : usize,
}

impl Batch
{
  fn push_geometry( &mut self, geometry : &Geometry, transform : Affine, frame : f64 )
  {
    // Merge with the previous geometry if possible. There are two
    // conditions:
    // 1. The previous geometry has not yet been referenced by a draw
    // 2. The geometries have the same transform
    if self.drawn_geometry < self.geometries.len()
      && self.geometries.last().map( | last | last.transform ) == Some( transform )
    {
      geometry.evaluate( frame, &mut self.elements );
      self.geometries.last_mut().unwrap().elements.end = self.elements.len();
    } 
    else
    {
      let start = self.elements.len();
      geometry.evaluate( frame, &mut self.elements );
      let end = self.elements.len();
      self.geometries.push
      ( 
        GeometryData
        {
          elements : start..end,
          transform,
        }
      );
    }
  }

  fn push_draw( &mut self, draw : &Draw, alpha : f64, geometry_start : usize, frame : f64 )
  {
    self.draws.push( DrawData::new
    (
      draw,
      alpha,
      geometry_start..self.geometries.len(),
      frame,
    ));
    self.drawn_geometry = self.geometries.len();
  }

  fn repeat( &mut self, repeater : &fixed::Repeater, geometry_start : usize, draw_start : usize )
  {
    // First move the relevant ranges of geometries and draws into side
    // buffers
    self.repeat_geometries
    .extend( self.geometries.drain( geometry_start.. ) );
    self.repeat_draws.extend( self.draws.drain( draw_start.. ) );
    // Next, repeat the geometries and apply the offset transform
    for geometry in self.repeat_geometries.iter()
    {
      for i in 0..repeater.copies
      {
        let transform = repeater.transform( i );
        let mut geometry = geometry.clone();
        geometry.transform *= transform;
        self.geometries.push( geometry );
      }
    }
    // Finally, repeat the draws, taking into account opacity and the
    // modified newly repeated geometry ranges
    let start_alpha = repeater.start_opacity / 100.0;
    let end_alpha = repeater.end_opacity / 100.0;
    let delta_alpha = if repeater.copies > 1
    {
      // See note in Skottie: AE does not cover the full opacity range
      ( end_alpha - start_alpha ) / repeater.copies as f64
    } 
    else
    {
      0.0
    };
    for i in 0..repeater.copies
    {
      let alpha = start_alpha + delta_alpha * i as f64;
      if alpha <= 0.0
      {
        continue;
      }
      for mut draw in self.repeat_draws.iter().cloned()
      {
        draw.alpha *= alpha;
        let count = draw.geometry.end - draw.geometry.start;
        draw.geometry.start =
        geometry_start + ( draw.geometry.start - geometry_start ) * repeater.copies;
        draw.geometry.end = draw.geometry.start + count * repeater.copies;
        self.draws.push( draw );
      }
    }
    // Clear the side buffers
    self.repeat_geometries.clear();
    self.repeat_draws.clear();
    // Prevent merging until new geometries are pushed
    self.drawn_geometry = self.geometries.len();
  }

  fn generate( &self ) -> ( Vec< geometry_generation::PrimitiveData >, Vec< minwebgl::math::F32x4 > )
  {
    // Process all draws in reverse
    for draw in self.draws.iter().rev()
    {
      // Some nastiness to avoid cloning the brush if unnecessary
      let modified_brush = if draw.alpha != 1.0
      {
        Some( draw.brush.clone().multiply_alpha( draw.alpha as f32 ) )
      } 
      else
      {
        None
      };
      let brush = modified_brush.as_ref().unwrap_or( &draw.brush );
      for geometry in self.geometries[ draw.geometry.clone() ].iter()
      {
        let path = &self.elements[ geometry.elements.clone() ];
        let transform = geometry.transform;
        if let Some( stroke ) = draw.stroke.as_ref()
        {
          //scene.stroke( stroke, transform, brush, None, &path );
        } 
        else
        {
          //scene.fill( Fill::NonZero, transform, brush, None, &path );
        }
      }
    }
  }

  fn clear( &mut self )
  {
    self.elements.clear();
    self.geometries.clear();
    self.draws.clear();
    self.repeat_geometries.clear();
    self.repeat_draws.clear();
    self.drawn_geometry = 0;
  }
}