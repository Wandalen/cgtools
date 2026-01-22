use minwebgpu::
{
  self as gl,
  web_sys
};
pub const NUM_MODELS : usize = 9;

#[ repr( C ) ]
#[derive( Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
pub struct InstanceRaw
{
  position : [ f32; 3 ]
}

pub struct Instance
{
  position : gl::F32x3
}

impl Instance
{
  pub fn as_raw( &self ) -> InstanceRaw
  {
    InstanceRaw
    {
      position : self.position.to_array(),
      ..Default::default()
    }
  }
}

pub struct ModelState
{
  pub pos_buffer : web_sys::GpuBuffer,
  pub normal_buffer : web_sys::GpuBuffer,
  pub uv_buffer : web_sys::GpuBuffer,
  pub index_buffer : web_sys::GpuBuffer,
  pub index_length : u32,
  pub instance_buffer : web_sys::GpuBuffer
}

impl ModelState
{
  pub async fn new( device : &web_sys::GpuDevice ) -> Result< ModelState, gl::WebGPUError >
  {
    // Load models, create buffer and initialize buffer with the data
    let model = gl::file::load( "bunny.obj" ).await.expect( "Failed to fetch the model" );
    let ( models, _ ) = gl::model::obj::load_model_from_slice( &model, "", &tobj::GPU_LOAD_OPTIONS ).await.unwrap();

    let model = &models[ 0 ];
    let mesh = &model.mesh;

    // Here we create and initialize buffers from the model information
    let pos_buffer = gl::BufferInitDescriptor::new( &mesh.positions, gl::BufferUsage::VERTEX ).create( device )?;
    let normal_buffer = gl::BufferInitDescriptor::new( &mesh.normals, gl::BufferUsage::VERTEX ).create( device )?;
    let uv_buffer = gl::BufferInitDescriptor::new( &mesh.texcoords, gl::BufferUsage::VERTEX ).create( device )?;
    let index_buffer = gl::BufferInitDescriptor::new( &mesh.indices, gl::BufferUsage::INDEX ).create( device )?;
    let index_length = mesh.indices.len() as u32;

    let instances = generate_instances();
    let instances_raw = instances.iter().map( | i | i.as_raw() ).collect::< Vec< InstanceRaw > >();
    let instance_buffer = gl::BufferInitDescriptor::new( &instances_raw, gl::BufferUsage::VERTEX ).create( device )?;

    Ok
    (
      ModelState
      {
        pos_buffer,
        normal_buffer,
        uv_buffer,
        index_buffer,
        index_length,
        instance_buffer
      }
    )
  }

  pub fn vertex_layout() -> [ web_sys::GpuVertexBufferLayout; 3 ]
  {
    // Step mode defaults to `Vertex`
    // Vertex Attribute offset defaults to 0.0
    // Vertex Attribute format defaults to Float32x3
    // If stride is not specified, it is computed from the attributes
    let pos_buffer_layout = gl::VertexBufferLayout::new()
    .attribute
    (
      gl::VertexAttribute::new()
      .location( 0 )
    );

    let normal_buffer_layout = gl::VertexBufferLayout::new()
    .attribute
    (
      gl::VertexAttribute::new()
      .location( 1 )
    );

    let uv_buffer_layout = gl::VertexBufferLayout::new()
    .attribute
    (
      gl::VertexAttribute::new()
      .location( 2 )
      .format( gl::GpuVertexFormat::Float32x2 )
    );

    [ pos_buffer_layout.into(), normal_buffer_layout.into(), uv_buffer_layout.into() ]
  }

  pub fn instance_layout() -> web_sys::GpuVertexBufferLayout
  {
    let buffer_layout = gl::VertexBufferLayout::new()
    .instance()
    .attribute
    (
      gl::VertexAttribute::new()
      .location( 5 )
    );

    buffer_layout.into()
  }
}

fn generate_instances() -> Vec< Instance >
{
  let mut instances = Vec::new();

  let spacing = 20.0;
  let rows = ( NUM_MODELS as f32 ).sqrt().ceil() as usize;
  let cols = NUM_MODELS.div_ceil( rows );

  let mid_c = if cols % 2 == 0 { cols / 2 - 1 } else { cols / 2 };
  let mid_r = if rows % 2 == 0 { rows / 2 - 1 } else { rows / 2 };

  let start_pos = gl::F32x3::from( [ -( mid_r as f32 ) * spacing, 0.0, -( mid_c as f32 ) * spacing ] );
  for r in 0..rows
  {
    for c in 0..cols
    {
      let position = start_pos + gl::F32x3::from( [ ( r as f32 ) * spacing, 0.0, ( c as f32 ) * spacing ] );

      let instance = Instance
      {
        position,
      };

      instances.push( instance );
    }
  }

  instances
}
