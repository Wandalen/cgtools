use minwebgpu::{self as gl, web_sys, WebGPUError};
use rand::Rng;
pub const NUM_LIGHTS : usize = 2500;

#[ repr( C ) ]
#[derive( Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
pub struct LightRaw
{
  color : [ f32; 3 ],
  power : f32,
  position : [ f32; 3 ],
  direction : f32
}

pub struct Light
{
  pub color : gl::F32x3,
  pub power : f32,
  pub position : gl::F32x3,
  pub direction : f32
}

impl Light
{
  pub fn as_raw( &self ) -> LightRaw
  {
    LightRaw
    {
      power : self.power,
      color : self.color.to_array(),
      position : self.position.to_array(),
      direction : self.direction,
      ..Default::default()
    }
  }
}

pub struct LightState
{
  pub buffer : gl::web_sys::GpuBuffer,
  pub mesh_position_buffer : gl::web_sys::GpuBuffer,
  pub mesh_index_buffer : gl::web_sys::GpuBuffer,
  pub num_vertices : u32,
  pub num_instances : u32,
  pub num_indices : u32
}

impl LightState
{
  pub fn new( device : &web_sys::GpuDevice ) -> Result< Self, WebGPUError >
  {
    let lights = generate_lights();
    let lights_raw = lights.iter().map( | l | l.as_raw() ).collect::< Vec< LightRaw > >();

    let num_instances = NUM_LIGHTS as u32;

    let ( sphere_vertices, sphere_indices ) = build_sphere_mesh();
    let num_vertices = ( sphere_vertices.len() / 3 ) as u32;
    let num_indices = sphere_indices.len() as u32;

    let buffer = gl::BufferInitDescriptor::new
    (
      &lights_raw,
      gl::BufferUsage::STORAGE | gl::BufferUsage::VERTEX
    ).create( device )?;

    let mesh_position_buffer = gl::BufferInitDescriptor::new
    (
      &sphere_vertices,
      gl::BufferUsage::VERTEX
    ).create( device )?;

    let mesh_index_buffer = gl::BufferInitDescriptor::new
    (
      &sphere_indices,
      gl::BufferUsage::INDEX
    ).create( device )?;


    Ok
    (
      LightState
      {
        buffer,
        mesh_position_buffer,
        mesh_index_buffer,
        num_vertices,
        num_instances,
        num_indices
      }
    )
  }

  pub fn light_shading_vertex_state( shader_module : &web_sys::GpuShaderModule ) -> gl::VertexState< '_ >
  {
    gl::VertexState::new( &shader_module )
    .buffer
    ( 
      // Light mesh position
      &gl::layout::VertexBufferLayout::new()
      .stride::< [ f32; 3 ] >()
      .attribute
      (
        gl::layout::VertexAttribute::new()
        .location( 0 )
        .format( gl::GpuVertexFormat::Float32x3 )
      )
      .into()
    )
    .buffer
    ( 
      // Light position
      &gl::layout::VertexBufferLayout::new()
      .instance()
      .stride::< LightRaw >()
      .attribute
      (
        gl::layout::VertexAttribute::new()
        .location( 1 )
        .offset::< [ f32; 4 ] >()
        .format( gl::GpuVertexFormat::Float32x3 )
      )
      .attribute
      (
        gl::layout::VertexAttribute::new()
        .location( 2 )
        .offset::< [ f32; 0 ] >()
        .format( gl::GpuVertexFormat::Float32x3 )
      )
      .attribute
      (
        gl::layout::VertexAttribute::new()
        .location( 3 )
        .offset::< [ f32; 3 ] >()
        .format( gl::GpuVertexFormat::Float32 )
      )
      .into()
    )    
  }
}

pub struct LightVisualizationState
{
  pub render_pipeline : web_sys::GpuRenderPipeline
}

impl LightVisualizationState
{
  pub fn vertex_layout() -> web_sys::GpuVertexBufferLayout
  {
    gl::layout::VertexBufferLayout::new()
    .instance()
    .stride::< LightRaw >()
    .attribute
    (
      gl::layout::VertexAttribute::new()
      .location( 0 )
      .offset::< [ f32; 4 ] >()
      .format( gl::GpuVertexFormat::Float32x3 )
    )
    .attribute
    (
      gl::layout::VertexAttribute::new()
      .location( 1 )
      .format( gl::GpuVertexFormat::Float32x3 )
    )
    .into()
  }

  pub fn new( device : &web_sys::GpuDevice, format : gl::GpuTextureFormat ) -> Result< Self, WebGPUError >
  {
    let shader_module = gl::shader::create( device, include_str!( "../shaders/light.wgsl" ) );

    let render_pipeline = gl::render_pipeline::desc
    (
      gl::VertexState::new( &shader_module )
      .buffer( &Self::vertex_layout() )
    )
    .fragment
    (
      gl::FragmentState::new( &shader_module )
      .target( gl::ColorTargetState::new().format( format ) )
    )
    .primitive( gl::PrimitiveState::new().triangle_strip() )
    .depth_stencil
    ( 
      gl::DepthStencilState::new()
      .format( gl::GpuTextureFormat::Depth24plusStencil8 )
    )
    .create( device )?;

    Ok
    (
      LightVisualizationState
      {
        render_pipeline
      }
    )
  }
}

fn generate_lights() -> Vec< Light >
{
  let mut rng = rand::rng();

  let mut lights = Vec::new();
  for _i in 0..NUM_LIGHTS
  {
    let power = rng.random::< f32 >();// * 2.0 + 1.0;
    let power = ( 1.0 - power ) * 0.2 + power * 1.0;
    let color = gl::F32x3::from( [ rng.random(), rng.random(), rng.random() ] );
    let direction = if rng.random::< f32 >() < 0.5 { -1.0 } else { 1.0 };

    let mut position = gl::F32x3::from
    ([
      rng.random::< f32 >() * 2.0,
      rng.random::< f32 >(),
      rng.random::< f32 >() * 2.0
    ]) - gl::F32x3::from( [ 1.0, 0.0, 1.0 ] ) ;

    position = position * gl::F32x3::from( [ 100.0, 5.0, 100.0 ] );

    let light = Light
    {
      power,
      color,
      position,
      direction
    };

    lights.push( light );
  }

  lights
}


struct Vertex 
{
  position : [ f32; 3 ],
  normal : [ f32; 3 ],
}

fn build_sphere_mesh() -> ( Vec< f32 >, Vec< u32 > ) {
    let sphere: csgrs::mesh::Mesh< () > = csgrs::mesh::Mesh::sphere( 1.0, 32, 16, None );
    let triangulated = sphere.triangulate();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for poly in &triangulated.polygons 
    {
      let verts = &poly.vertices;
      if verts.len() < 3 
      {
          continue;
      }
      let base = vertices.len() as u32;
      for v in verts 
      {
        let p = v.pos;
        let n = v.normal;
        vertices.push
        (
          Vertex 
          {
            position: [ p.x as f32, p.y as f32, p.z as f32 ],
            normal: [ n.x as f32, n.y as f32, n.z as f32 ],
          }
        );
      }
      // Fan triangulation (polygons should already be triangles after triangulate())
      for i in 1..( verts.len() as u32 - 1 ) 
      {
        indices.push( base );
        indices.push( base + i );
        indices.push( base + i + 1 );
      }
    }

    let vertices = vertices.into_iter().flat_map( | v | v.position ).collect();

    (vertices, indices)
}