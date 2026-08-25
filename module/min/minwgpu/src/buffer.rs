//! This module provides a fluent builder pattern for creating `wgpu` buffers,
//! simplifying the setup process for vertex buffers and other buffer types.
//! It offers a clear and chained method for configuring and building `wgpu::Buffer` instances.

use mingl::mod_interface;

mod private
{
  use wgpu::util::DeviceExt as _;

  /// Creates a new `BufferBuilder` to construct a generic `wgpu::Buffer`.
  ///
  /// This is the entry point for building buffers with custom usage flags.
  #[ inline ]
  #[ must_use ]
  pub fn buffer( usage : wgpu::BufferUsages ) -> BufferBuilder< 'static >
  {
    BufferBuilder
    {
      inner : BufferBuilderInner
      {
        data : None,
        label : None,
        size : 0,
        mapped_at_creation : false,
        usage
      }
    }
  }

  /// Creates a new `VertexBufferBuilder` to construct a `VertexBuffer`.
  ///
  /// This is the specialized entry point for building buffers intended for use as vertex buffers.
  #[ inline ]
  #[ must_use ]
  pub fn vertex_buffer() -> VertexBufferBuilder< 'static >
  {
    VertexBufferBuilder::new()
  }

  /// A container for a `wgpu::Buffer` and its corresponding `wgpu::VertexBufferLayout`.
  ///
  /// This struct pairs a buffer with the layout description required by a render pipeline.
  #[ non_exhaustive ]
  #[ derive( Debug ) ]
  pub struct VertexBuffer< 'a >
  {
    /// The raw `wgpu` buffer that stores the vertex data on the GPU.
    pub buffer : wgpu::Buffer,
    /// The layout that describes how the vertex data in the buffer is structured.
    pub layout : wgpu::VertexBufferLayout< 'a >,
  }

  impl< 'a > VertexBuffer< 'a >
  {
    /// Returns a reference to the underlying `wgpu::Buffer`.
    #[ inline ]
    #[ must_use ]
    pub fn buffer_get( &self ) -> &wgpu::Buffer
    {
      &self.buffer
    }

    /// Returns a reference to the `wgpu::VertexBufferLayout`.
    #[ inline ]
    #[ must_use ]
    pub fn layout_get( &self ) -> &wgpu::VertexBufferLayout< 'a >
    {
      &self.layout
    }
  }

  impl AsRef< wgpu::Buffer > for VertexBuffer< '_ >
  {
    #[ inline ]
    fn as_ref( &self ) -> &wgpu::Buffer
    {
      &self.buffer
    }
  }

  /// Internal struct holding the common configuration for any buffer builder.
  #[ derive( Debug ) ]
  pub( super ) struct BufferBuilderInner< 'a >
  {
    pub( super ) data : Option< &'a [ u8 ] >,
    pub( super ) label : Option< &'a str >,
    pub( super ) size : wgpu::BufferAddress,
    pub( super ) mapped_at_creation : bool,
    pub( super ) usage : wgpu::BufferUsages,
  }

  impl BufferBuilderInner< '_ >
  {
    /// Finalizes the configuration and creates the `wgpu::Buffer`.
    #[ inline ]
    #[ must_use ]
    pub fn build( self, device : &wgpu::Device ) -> wgpu::Buffer
    {
      let Self { data, label, size, mapped_at_creation, usage } = self;

      if let Some( data ) = data
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
      }
    }
  }

  macro_rules! impl_buffer_builder_methods
  {
    ( $builder_ty:ty, $field_name:ident ) =>
    {
      impl< 'a > $builder_ty
      {
        /// Sets the initial data for the buffer from a slice of any plain old data type.
        /// If data is provided then buffer will be created with `wgpu::util::BufferInitDescriptor`.
        #[ inline ]
        #[ must_use ]
        pub fn data< D >( mut self, value : &'a [ D ] ) -> Self
        where
          D : asbytes::NoUninit
        {
          self.$field_name.data = Some( asbytes::cast_slice( value ) );
          self
        }

        /// Sets a debug label for the buffer.
        #[ inline ]
        #[ must_use ]
        pub fn label( mut self, value : &'a str ) -> Self
        {
          self.$field_name.label = Some( value );
          self
        }

        /// Sets the buffer size based on the size of a generic type `T`.
        /// If data is not provided then buffer will be created with `wgpu::BufferDescriptor` of specified size.
        #[ inline ]
        #[ must_use ]
        pub fn size< T >( mut self ) -> Self
        {
          self.$field_name.size = core::mem::size_of::< T >() as u64;
          self
        }

        /// Sets the buffer size from the in-memory size of a given variable.
        #[ inline ]
        #[ must_use ]
        pub fn size_from_var< T : ?Sized >( mut self, var : &T ) -> Self
        {
          self.$field_name.size = core::mem::size_of_val( var ) as u64;
          self
        }

        /// Sets the buffer size directly from a provided value.
        #[ inline ]
        #[ must_use ]
        pub fn size_from_value( mut self, value : wgpu::BufferAddress ) -> Self
        {
          self.$field_name.size = value;
          self
        }

        /// Specifies whether the buffer should be mapped at creation.
        #[ inline ]
        #[ must_use ]
        pub fn mapped_at_creation( mut self, value : bool ) -> Self
        {
          self.$field_name.mapped_at_creation = value;
          self
        }

        /// Sets the usage flags for the buffer.
        #[ inline ]
        #[ must_use ]
        pub fn usage( mut self, value : wgpu::BufferUsages ) -> Self
        {
          self.$field_name.usage = value;
          self
        }

        /// A convenience method to set the buffer usage, often used for vertex buffers.
        #[ inline ]
        #[ must_use ]
        pub fn vertex_usage( mut self, value : wgpu::BufferUsages ) -> Self
        {
          self.$field_name.usage = value;
          self
        }

        /// Returns the configured initial data as raw bytes, if any.
        #[ inline ]
        #[ must_use ]
        pub fn data_get( &self ) -> Option< &'a [ u8 ] >
        {
          self.$field_name.data
        }

        /// Returns the configured debug label, if any.
        #[ inline ]
        #[ must_use ]
        pub fn label_get( &self ) -> Option< &'a str >
        {
          self.$field_name.label
        }

        /// Returns the configured buffer size in bytes.
        #[ inline ]
        #[ must_use ]
        pub fn size_get( &self ) -> wgpu::BufferAddress
        {
          self.$field_name.size
        }

        /// Returns whether the buffer is to be mapped at creation.
        #[ inline ]
        #[ must_use ]
        pub fn mapped_at_creation_get( &self ) -> bool
        {
          self.$field_name.mapped_at_creation
        }

        /// Returns the configured usage flags.
        #[ inline ]
        #[ must_use ]
        pub fn usage_get( &self ) -> wgpu::BufferUsages
        {
          self.$field_name.usage
        }
      }
    };
  }

  /// A fluent builder for creating `wgpu::Buffer` instances.
  #[ derive( Debug ) ]
  pub struct BufferBuilder< 'a >
  {
    pub( super ) inner : BufferBuilderInner< 'a >
  }

  impl_buffer_builder_methods!( BufferBuilder< 'a >, inner );

  impl BufferBuilder< '_ >
  {
    /// Consumes the builder and creates the configured `wgpu::Buffer`.
    #[ inline ]
    #[ must_use ]
    pub fn build( self, device : &wgpu::Device ) -> wgpu::Buffer
    {
      self.inner.build( device )
    }
  }

  /// A fluent builder for creating `VertexBuffer` instances.
  ///
  /// This builder configures both the buffer's data and its memory layout description.
  #[ derive( Debug ) ]
  pub struct VertexBufferBuilder< 'a >
  {
    pub( super ) array_stride : wgpu::BufferAddress,
    pub( super ) step_mode : wgpu::VertexStepMode,
    pub( super ) attributes : &'a [ wgpu::VertexAttribute ],
    pub( super ) buffer_builder : BufferBuilderInner< 'a >
  }

  impl_buffer_builder_methods!( VertexBufferBuilder< 'a >, buffer_builder );

  impl< 'a > VertexBufferBuilder< 'a >
  {
    /// Creates a new `VertexBufferBuilder` with default settings.
    fn new() -> Self
    {
      Self
      {
        array_stride  : 0,
        step_mode : wgpu::VertexStepMode::Vertex,
        attributes : &[],
        buffer_builder : buffer( wgpu::BufferUsages::VERTEX ).inner
      }
    }

    /// Sets the byte distance between consecutive elements in the buffer.
    #[ inline ]
    #[ must_use ]
    pub fn array_stride( mut self, value : wgpu::BufferAddress ) -> Self
    {
      self.array_stride = value;
      self
    }

    /// Sets how often the vertex buffer is stepped forward.
    #[ inline ]
    #[ must_use ]
    pub fn step_mode( mut self, value : wgpu::VertexStepMode ) -> Self
    {
      self.step_mode = value;
      self
    }

    /// Sets the vertex attributes that describe how to interpret the buffer data.
    #[ inline ]
    #[ must_use ]
    pub fn attributes( mut self, value : &'a [ wgpu::VertexAttribute ] ) -> Self
    {
      self.attributes = value;
      self
    }

    /// Returns the configured byte distance between consecutive elements.
    #[ inline ]
    #[ must_use ]
    pub fn array_stride_get( &self ) -> wgpu::BufferAddress
    {
      self.array_stride
    }

    /// Returns the configured step mode.
    #[ inline ]
    #[ must_use ]
    pub fn step_mode_get( &self ) -> wgpu::VertexStepMode
    {
      self.step_mode
    }

    /// Returns the configured vertex attributes.
    #[ inline ]
    #[ must_use ]
    pub fn attributes_get( &self ) -> &'a [ wgpu::VertexAttribute ]
    {
      self.attributes
    }

    /// Consumes the builder and creates the configured `VertexBuffer`.
    #[ inline ]
    #[ must_use ]
    pub fn build( self, device : &wgpu::Device ) -> VertexBuffer< 'a >
    {
      let Self { array_stride, step_mode, attributes, buffer_builder } = self;
      let buffer = buffer_builder.build( device );

      let layout = wgpu::VertexBufferLayout
      {
        array_stride,
        step_mode,
        attributes,
      };

      VertexBuffer { buffer, layout }
    }
  }
}

mod_interface!
{
  own use buffer;
  own use vertex_buffer;
  own use VertexBuffer;
  own use BufferBuilder;
  own use VertexBufferBuilder;
}
