mod private
{
  use gpu_hal::
  {
    Device,
    Buffer,
    BufferUsage,
    Error,
    VertexAttribute,
    VertexBufferLayout,
    VertexFormat
  };

  /// GPU-resident mesh: one vertex buffer per attribute + optional index buffer.
  ///
  /// Attribute slots match the canonical opaque shader's vertex inputs:
  /// 0 — position ( vec3 ), 1 — normal ( vec3 ), 2 — uv ( vec2 ),
  /// 3 — color ( vec4 ).
  pub struct Geometry
  {
    /// Vertex buffers in attribute-slot order: position, normal, uv, color.
    pub vertex_buffers : [ Buffer; 4 ],
    /// Index buffer ( `u32` indices ), if the mesh is indexed.
    pub index_buffer : Option< Buffer >,
    /// Number of vertices.
    pub vertex_count : u32,
    /// Number of indices; zero for non-indexed meshes.
    pub index_count : u32
  }

  impl Geometry
  {
    /// Uploads raw attribute data. `positions` and `normals` are xyz triples,
    /// `uvs` are uv pairs, `colors` are rgba quadruples — all per vertex, with
    /// the same vertex count.
    ///
    /// # Errors
    ///
    /// Returns an error when any vertex or index buffer allocation fails on the device.
    pub fn new
    (
      device : &Device,
      positions : &[ f32 ],
      normals : &[ f32 ],
      uvs : &[ f32 ],
      colors : &[ f32 ],
      indices : Option< Vec< u32 > >
    ) -> Result< Self, Error >
    {
      let vertex_count = ( positions.len() / 3 ) as u32;
      let position_buffer = device.create_buffer_init( bytemuck::cast_slice( positions ), BufferUsage::VERTEX )?;
      let normal_buffer = device.create_buffer_init( bytemuck::cast_slice( normals ), BufferUsage::VERTEX )?;
      let uv_buffer = device.create_buffer_init( bytemuck::cast_slice( uvs ), BufferUsage::VERTEX )?;
      let color_buffer = device.create_buffer_init( bytemuck::cast_slice( colors ), BufferUsage::VERTEX )?;

      let mut index_count = 0;
      let index_buffer = match indices
      {
        Some( data ) =>
        {
          index_count = data.len() as u32;
          Some( device.create_buffer_init( bytemuck::cast_slice( &data ), BufferUsage::INDEX )? )
        }
        None => None
      };

      Ok
      (
        Self
        {
          vertex_buffers : [ position_buffer, normal_buffer, uv_buffer, color_buffer ],
          index_buffer,
          vertex_count,
          index_count
        }
      )
    }

    /// Vertex buffer layouts matching the canonical opaque shader's inputs,
    /// in attribute-slot order.
    #[ must_use ]
    pub fn vertex_layouts() -> [ VertexBufferLayout; 4 ]
    {
      [
        VertexBufferLayout
        {
          stride : 12,
          attributes : vec!
          [
            VertexAttribute { location : 0, format : VertexFormat::Float32x3, offset : 0 }
          ]
        },
        VertexBufferLayout
        {
          stride : 12,
          attributes : vec!
          [
            VertexAttribute { location : 1, format : VertexFormat::Float32x3, offset : 0 }
          ]
        },
        VertexBufferLayout
        {
          stride : 8,
          attributes : vec!
          [
            VertexAttribute { location : 2, format : VertexFormat::Float32x2, offset : 0 }
          ]
        },
        VertexBufferLayout
        {
          stride : 16,
          attributes : vec!
          [
            VertexAttribute { location : 3, format : VertexFormat::Float32x4, offset : 0 }
          ]
        }
      ]
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Geometry
  };
}
