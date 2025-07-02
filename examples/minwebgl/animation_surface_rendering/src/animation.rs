// Copyright 2024 the Interpoli Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use interpoli::{ fixed, Composition, Content, Draw, Geometry, GroupTransform, Layer, Shape, Stroke, Brush };
use kurbo::{ Affine, PathEl };
use renderer::webgl::loaders::gltf::GLTF;
use std::ops::Range;
use minwebgl::{ F32x4, GL };

use renderer::webgl::
{
  Scene,
  Node
};
use geometry_generation::AttributesData;
use color::{ AlphaColor, Rgba8 };

fn merge_gltfs( gltfs : Vec< GLTF > ) -> GLTF
{
  let scenes = vec![];
  let nodes = vec![];
  let gl_buffers = vec![];
  let images = vec![];
  let textures = vec![];
  let materials = vec![];
  let meshes = vec![];

  for gltf in gltfs 
  {
    scenes.extend( gltf.scenes );
    nodes.extend( gltf.nodes );
    gl_buffers.extend( gltf.gl_buffers );
    images.extend( gltf.images.borrow() );
    textures.extend( gltf.textures );
    materials.extend( gltf.materials );
    meshes.extend( gltf.meshes );
  }

  GLTF
  {
    scenes,
    nodes,
    gl_buffers,
    images : Rc::new( RefCell::new( images ) ),
    textures,
    materials,
    meshes,
}
}

#[ derive( Clone ) ]
pub struct PrimitiveData 
{
  pub attributes : Rc< RefCell< AttributesData > >,
  pub parent : Option< usize >,
  pub color : F32x4,
  pub transform : Transform
}

pub fn primitives_data_to_gltf
( 
  gl : &WebGl2RenderingContext,
  primitives_data : Vec< PrimitiveData >
) -> GLTF
{
  let mut scenes = vec![];
  let mut nodes = vec![];
  let mut gl_buffers = vec![]; 
  let mut meshes = vec![];

  let material = Rc::new( RefCell::new( Material::default() ) );
  let materials = vec![ material.clone() ];

  scenes.push( Rc::new( RefCell::new( Scene::new() ) ) );

  let position_buffer = gl.create_buffer().unwrap();

  gl_buffers.push( position_buffer.clone() );

  let attribute_infos = 
  [
    ( 
      "positions", 
      geometry_generation::make_buffer_attibute_info( 
        &position_buffer, 
        BufferDescriptor::new::< [ f32; 3 ] >(),
        0, 
        3, 
        0, 
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap() 
    ),
  ];

  let index_buffer = gl.create_buffer().unwrap();
  gl_buffers.push( index_buffer.clone() );

  let mut index_info = IndexInfo
  {
    buffer : index_buffer.clone(),
    count : 0,
    offset : 0,
    data_type : GL::UNSIGNED_INT
  };

  let mut positions = vec![];
  let mut indices = vec![];

  for primitive_data in primitives_data
  {
    let last_positions_count = positions.len() as u32;
    positions.extend( primitive_data.attributes.borrow().positions.clone() );
    let primitive_indices = primitive_data.attributes.borrow().indices.iter()
    .map( | i | i + last_positions_count )
    .collect::< Vec< _ > >();
    let offset = indices.len() as u32 * 4;
    indices.extend( primitive_indices );

    index_info.offset = offset;
    index_info.count = primitive_data.attributes.borrow().indices.len() as u32;

    let Ok( mut geometry ) = Geometry::new( gl ) else
    {
      panic!( "Can't create new Geometry struct" );
    };

    for ( name, info ) in &attribute_infos
    {
      geometry.add_attribute( gl, *name, info.clone(), false ).unwrap();
    }

    geometry.add_index( gl, index_info.clone() ).unwrap();
    geometry.vertex_count = primitive_data.attributes.borrow().positions.len() as u32;

    let primitive = Primitive
    {
      geometry : Rc::new( RefCell::new( geometry ) ),
      material : material.clone()
    };

    let mesh = Rc::new( RefCell::new( Mesh::new() ) );
    mesh.borrow_mut().add_primitive( Rc::new( RefCell::new( primitive ) ) );

    let node = Rc::new( RefCell::new( Node::new() ) );
    node.borrow_mut().object = Object3D::Mesh( mesh.clone() );
    primitive_data.transform.set_node_transform( node.clone() );

    nodes.push( node.clone() );
    meshes.push( mesh );
    scenes[ 0 ].borrow_mut().children.push( node );
  }

  gl::buffer::upload( &gl, &position_buffer, &positions, GL::STATIC_DRAW );
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );
  
  GLTF
  {
    scenes,
    nodes,
    gl_buffers,
    images : Rc::new( RefCell::new( vec![] ) ),
    textures : vec![],
    materials,
    meshes
  }
}

struct Animation
{
  gltf : GLTF,
  composition : Composition
}

impl Animation
{ 
  fn new( gl : &GL, composition : Composition ) -> Self
  {
    let mut colors = vec![];
    let mut gltfs = vec![];

    for layer in composition.layers
    {
      let mut primitives_data = vec![];

      layer.parent
      layer.transform

      let Content::Shape( mut shapes ) = layer.content 
      else
      {
        continue;
      };

      let mut color = F32x4::from_array( [ 0.0, 0.0, 0.0, 1.0 ] );
      let mut stroke_width = None;
      let mut i = 0;
      while i < shapes.len() 
      {
        match shape
        {
          Shape::Group( shapes, group_transform ) => 
          {

          },
          Shape::Geometry( Geometry::Fixed( path ) ) => 
          {

          },
          Shape::Draw
          ( 
            Draw
            {
              stroke,
              brush : Brush::Fixed( peniko::Brush::Solid( color ) ),
              opacity,
            } 
          ) => 
          {
            if let Some( Stroke::Fixed( stroke ) ) = stroke
            {
              stroke_width = Some( stroke.width );
            }

            color.to_rgba8()
          },
          _ => continue
        }

        i += 1;
      }

      let gltf = primitives_data_to_gltf( gl, primitives_data );
      gltfs.push( gltf ); 
    }

    let gltf = merge_gltfs( gltfs );

    Self
    {
      gltf,
      composition
    }
  }
}

#[ derive( Default ) ]
pub struct Animator
{
  batch : Batch,
  frame_geometry : Vec< geometry_generation::PrimitiveData >,
  colors : Vec< F32x4 >,
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
    animation : &mut Animation,
    frame : f64,
    transform : Affine,
    alpha : f64,
  ) -> ( Vec< geometry_generation::PrimitiveData >, Vec< F32x4 > )
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
    if let Some( ( _, mask_index ) ) = layer.mask_layer
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
    let mut alpha = alpha * layer.opacity.evaluate( frame ) / 100.0;
    for mask in &layer.masks
    {
      alpha = mask.opacity.evaluate( frame ) / 100.0;
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

  fn generate( &self ) -> ( Vec< geometry_generation::PrimitiveData >, Vec< F32x4 > )
  {
    let mut primitives = Vec::< geometry_generation::PrimitiveData >::new();
    let mut colors = Vec::< F32x4 >::new();

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
          primitives.push();
          colors.push();
          //scene.stroke( stroke, transform, brush, None, &path );
        } 
        else
        {
          primitives.push();
          colors.push();
          //scene.fill( Fill::NonZero, transform, brush, None, &path );
        }
      }
    }

    ( primitives, colors )
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