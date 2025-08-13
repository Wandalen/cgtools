use wgpu::util::DeviceExt as _;

pub struct Buffer< Layout >
{
  buffer : wgpu::Buffer,
  layout : Layout,
}

impl< Layout > Buffer< Layout >
{
  pub fn buffer( &self ) -> &wgpu::Buffer
  {
    &self.buffer
  }

  pub fn layout( &self ) -> &Layout
  {
    &self.layout
  }
}

impl Buffer< () >
{
  pub fn buffer_builder( usage : wgpu::BufferUsages ) -> BufferBuilder< 'static >
  {
    BufferBuilder
    {
      data : None,
      label : None,
      size : 0,
      mapped_at_creation : false,
      usage
    }
  }
}

impl Buffer< wgpu::VertexBufferLayout< '_ > >
{
  pub fn vertex_buffer_builder() -> VertexBufferBuilder< 'static >
  {
    VertexBufferBuilder::new()
  }
}

impl< L > AsRef< wgpu::Buffer > for Buffer< L >
{
  fn as_ref( &self ) -> &wgpu::Buffer
  {
    &self.buffer
  }
}

#[ derive( Debug ) ]
pub struct BufferBuilder< 'a >
{
  data : Option< &'a [ u8 ] >,
  label : Option< &'a str >,
  size : wgpu::BufferAddress,
  mapped_at_creation : bool,
  usage : wgpu::BufferUsages,
}

impl< 'a > BufferBuilder< 'a >
{
  pub fn data< D >( mut self, value : &'a [ D ] ) -> Self
  where
    D : bytemuck::NoUninit
  {
    self.data = Some( bytemuck::cast_slice( value ) );
    self
  }

  pub fn label( mut self, value : &'a str ) -> Self
  {
    self.label = Some( value );
    self
  }

  pub fn size( mut self, value : wgpu::BufferAddress ) -> Self
  {
    self.size = value;
    self
  }

  pub fn mapped_at_creation( mut self, value : bool ) -> Self
  {
    self.mapped_at_creation = value;
    self
  }

  pub fn usage( mut self, value : wgpu::BufferUsages ) -> Self
  {
    self.usage = value;
    self
  }

  pub fn build( self, device : &wgpu::Device ) -> Buffer< () >
  {
    let Self { data, label, size, mapped_at_creation, usage } = self;
    let buffer = if let Some( data ) = data
    {
      let descriptor = wgpu::util::BufferInitDescriptor
      {
        label,
        contents : data,
        usage,
      };
      device.create_buffer_init( &descriptor )
    }
    else
    {
      let descriptor = wgpu::BufferDescriptor
      {
        label,
        size,
        usage,
        mapped_at_creation,
      };
      device.create_buffer( &descriptor )
    };
    Buffer { buffer, layout : () }
  }
}

macro_rules! impl_buffer_builder_methods
{
  ( $builder_ty:ty, $field_name:ident ) =>
  {
    impl< 'a > $builder_ty
    {
      pub fn data< D >( mut self, value : &'a [ D ] ) -> Self
      where
        D : bytemuck::NoUninit
      {
        self.$field_name.data = Some( bytemuck::cast_slice( value ) );
        self
      }

      pub fn label( mut self, value : &'a str ) -> Self
      {
        self.$field_name.label = Some( value );
        self
      }

      pub fn size( mut self, value : wgpu::BufferAddress ) -> Self
      {
        self.$field_name.size = value;
        self
      }

      pub fn mapped_at_creation( mut self, value : bool ) -> Self
      {
        self.$field_name.mapped_at_creation = value;
        self
      }

      pub fn usage( mut self, value : wgpu::BufferUsages ) -> Self
      {
        self.$field_name.usage = value;
        self
      }
    }
  };
}

#[ derive( Debug ) ]
pub struct VertexBufferBuilder< 'a >
{
  array_stride : wgpu::BufferAddress,
  step_mode : wgpu::VertexStepMode,
  attributes : &'a [ wgpu::VertexAttribute ],
  buffer_builder : BufferBuilder< 'a >
}

impl_buffer_builder_methods!( VertexBufferBuilder< 'a >, buffer_builder );

impl< 'a > VertexBufferBuilder< 'a >
{
  fn new() -> Self
  {
    Self
    {
      array_stride  : 0,
      step_mode : wgpu::VertexStepMode::Vertex,
      attributes : &[],
      buffer_builder : Buffer::buffer_builder( wgpu::BufferUsages::VERTEX )
    }
  }

  pub fn array_stride( mut self, value : wgpu::BufferAddress ) -> Self
  {
    self.array_stride = value;
    self
  }

  pub fn step_mode( mut self, value : wgpu::VertexStepMode ) -> Self
  {
    self.step_mode = value;
    self
  }

  pub fn attributes( mut self, value : &'a [ wgpu::VertexAttribute ] ) -> Self
  {
    self.attributes = value;
    self
  }

  pub fn build( self, device : &wgpu::Device ) -> Buffer< wgpu::VertexBufferLayout< 'a > >
  {
    let Self { array_stride, step_mode, attributes, buffer_builder } = self;
    let Buffer { buffer, .. } = buffer_builder.build( device );

    let layout = wgpu::VertexBufferLayout
    {
      array_stride,
      step_mode,
      attributes,
    };

    Buffer { buffer, layout }
  }
}

pub const fn attr
(
  format : wgpu::VertexFormat,
  offset : wgpu::BufferAddress,
  shader_location : wgpu::ShaderLocation
) -> wgpu::VertexAttribute
{
  wgpu::VertexAttribute
  {
    format,
    offset,
    shader_location,
  }
}
