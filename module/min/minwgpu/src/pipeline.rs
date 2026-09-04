//! This module provides render pipeline construction : a general fluent builder covering
//! vertex buffers, instancing, bind groups, blending and depth, plus the bufferless-triangle
//! shortcut used by shader-driven offscreen examples.

use mingl::mod_interface;

mod private
{
  /// Creates a new `RenderPipelineBuilder`.
  ///
  /// Entry point for building a render pipeline of any shape. Defaults match
  /// [`fullscreen`] — `vs_main`/`fs_main` entry points, [`wgpu::BlendState::REPLACE`],
  /// `wgpu` default primitive/multisample state, no depth, no vertex buffers — so the
  /// shortcut and the general form stay consistent.
  #[ inline ]
  #[ must_use ]
  pub fn render_pipeline() -> RenderPipelineBuilder< 'static >
  {
    RenderPipelineBuilder
    {
      label : None,
      wgsl : "",
      vertex_entry : "vs_main",
      fragment_entry : "fs_main",
      buffers : Vec::new(),
      layouts : Vec::new(),
      format : None,
      blend : Some( wgpu::BlendState::REPLACE ),
      write_mask : wgpu::ColorWrites::ALL,
      primitive : wgpu::PrimitiveState::default(),
      multisample : wgpu::MultisampleState::default(),
      depth_stencil : None,
    }
  }

  /// A fluent builder for `wgpu::RenderPipeline`.
  ///
  /// Accepts WGSL source rather than a prebuilt `wgpu::ShaderModule`, matching
  /// [`fullscreen`]'s existing convention — the module is created during [`RenderPipelineBuilder::build`].
  #[ derive( Debug ) ]
  pub struct RenderPipelineBuilder< 'a >
  {
    pub( super ) label : Option< &'a str >,
    pub( super ) wgsl : &'a str,
    pub( super ) vertex_entry : &'a str,
    pub( super ) fragment_entry : &'a str,
    pub( super ) buffers : Vec< wgpu::VertexBufferLayout< 'a > >,
    pub( super ) layouts : Vec< &'a wgpu::BindGroupLayout >,
    pub( super ) format : Option< wgpu::TextureFormat >,
    pub( super ) blend : Option< wgpu::BlendState >,
    pub( super ) write_mask : wgpu::ColorWrites,
    pub( super ) primitive : wgpu::PrimitiveState,
    pub( super ) multisample : wgpu::MultisampleState,
    pub( super ) depth_stencil : Option< wgpu::DepthStencilState >,
  }

  impl< 'a > RenderPipelineBuilder< 'a >
  {
    /// Sets a debug label, used for the pipeline, its layout and its shader module.
    #[ inline ]
    #[ must_use ]
    pub fn label( mut self, value : &'a str ) -> Self
    {
      self.label = Some( value );
      self
    }

    /// Sets the WGSL source defining both entry points.
    #[ inline ]
    #[ must_use ]
    pub fn wgsl( mut self, value : &'a str ) -> Self
    {
      self.wgsl = value;
      self
    }

    /// Sets the vertex entry point name ( default `vs_main` ).
    #[ inline ]
    #[ must_use ]
    pub fn vertex_entry( mut self, value : &'a str ) -> Self
    {
      self.vertex_entry = value;
      self
    }

    /// Sets the fragment entry point name ( default `fs_main` ).
    #[ inline ]
    #[ must_use ]
    pub fn fragment_entry( mut self, value : &'a str ) -> Self
    {
      self.fragment_entry = value;
      self
    }

    /// Appends one vertex buffer layout, in the slot order the pipeline will expect.
    ///
    /// Call once per bound vertex buffer. A layout whose `step_mode` is
    /// [`wgpu::VertexStepMode::Instance`] is what makes a draw instanced — pair it with a
    /// non-trivial instance range on [`crate::pass::Draw`].
    #[ inline ]
    #[ must_use ]
    pub fn buffer_layout( mut self, value : wgpu::VertexBufferLayout< 'a > ) -> Self
    {
      self.buffers.push( value );
      self
    }

    /// Appends one bind group layout, in the group order the shader will expect.
    #[ inline ]
    #[ must_use ]
    pub fn bind_group_layout( mut self, value : &'a wgpu::BindGroupLayout ) -> Self
    {
      self.layouts.push( value );
      self
    }

    /// Sets the color target format. Required — [`RenderPipelineBuilder::build`] panics without it.
    ///
    /// For a windowed pipeline this must be the surface's own format, available as
    /// `Windowed::format`.
    #[ inline ]
    #[ must_use ]
    pub fn format( mut self, value : wgpu::TextureFormat ) -> Self
    {
      self.format = Some( value );
      self
    }

    /// Sets the color blend state ( default [`wgpu::BlendState::REPLACE`] ).
    #[ inline ]
    #[ must_use ]
    pub fn blend( mut self, value : wgpu::BlendState ) -> Self
    {
      self.blend = Some( value );
      self
    }

    /// Disables blending entirely, writing fragment output directly to the target.
    #[ inline ]
    #[ must_use ]
    pub fn blend_none( mut self ) -> Self
    {
      self.blend = None;
      self
    }

    /// Sets which color channels are written ( default [`wgpu::ColorWrites::ALL`] ).
    #[ inline ]
    #[ must_use ]
    pub fn write_mask( mut self, value : wgpu::ColorWrites ) -> Self
    {
      self.write_mask = value;
      self
    }

    /// Sets the primitive state — topology, culling, winding.
    #[ inline ]
    #[ must_use ]
    pub fn primitive( mut self, value : wgpu::PrimitiveState ) -> Self
    {
      self.primitive = value;
      self
    }

    /// Sets the multisample state.
    #[ inline ]
    #[ must_use ]
    pub fn multisample( mut self, value : wgpu::MultisampleState ) -> Self
    {
      self.multisample = value;
      self
    }

    /// Sets the depth/stencil state ( default : none ).
    #[ inline ]
    #[ must_use ]
    pub fn depth_stencil( mut self, value : wgpu::DepthStencilState ) -> Self
    {
      self.depth_stencil = Some( value );
      self
    }

    /// Returns the configured debug label, if any.
    #[ inline ]
    #[ must_use ]
    pub fn label_get( &self ) -> Option< &'a str >
    {
      self.label
    }

    /// Returns the configured WGSL source.
    #[ inline ]
    #[ must_use ]
    pub fn wgsl_get( &self ) -> &'a str
    {
      self.wgsl
    }

    /// Returns the configured vertex entry point name.
    #[ inline ]
    #[ must_use ]
    pub fn vertex_entry_get( &self ) -> &'a str
    {
      self.vertex_entry
    }

    /// Returns the configured fragment entry point name.
    #[ inline ]
    #[ must_use ]
    pub fn fragment_entry_get( &self ) -> &'a str
    {
      self.fragment_entry
    }

    /// Returns the accumulated vertex buffer layouts, in slot order.
    #[ inline ]
    #[ must_use ]
    pub fn buffer_layouts_get( &self ) -> &[ wgpu::VertexBufferLayout< 'a > ]
    {
      &self.buffers
    }

    /// Returns the accumulated bind group layouts, in group order.
    #[ inline ]
    #[ must_use ]
    pub fn bind_group_layouts_get( &self ) -> &[ &'a wgpu::BindGroupLayout ]
    {
      &self.layouts
    }

    /// Returns the configured color target format, if set.
    #[ inline ]
    #[ must_use ]
    pub fn format_get( &self ) -> Option< wgpu::TextureFormat >
    {
      self.format
    }

    /// Returns the configured blend state, if any.
    #[ inline ]
    #[ must_use ]
    pub fn blend_get( &self ) -> Option< wgpu::BlendState >
    {
      self.blend
    }

    /// Returns the configured color write mask.
    #[ inline ]
    #[ must_use ]
    pub fn write_mask_get( &self ) -> wgpu::ColorWrites
    {
      self.write_mask
    }

    /// Returns the configured primitive state.
    #[ inline ]
    #[ must_use ]
    pub fn primitive_get( &self ) -> &wgpu::PrimitiveState
    {
      &self.primitive
    }

    /// Returns the configured multisample state.
    #[ inline ]
    #[ must_use ]
    pub fn multisample_get( &self ) -> &wgpu::MultisampleState
    {
      &self.multisample
    }

    /// Returns the configured depth/stencil state, if any.
    #[ inline ]
    #[ must_use ]
    pub fn depth_stencil_get( &self ) -> Option< &wgpu::DepthStencilState >
    {
      self.depth_stencil.as_ref()
    }

    /// Consumes the builder and creates the configured `wgpu::RenderPipeline`.
    ///
    /// # Panics
    /// Panics if no color target format was set via [`RenderPipelineBuilder::format`]. A
    /// pipeline has no meaningful default target format — it must match the surface or
    /// texture it renders into, which only the caller knows.
    #[ must_use ]
    pub fn build( self, device : &wgpu::Device ) -> wgpu::RenderPipeline
    {
      let Self
      {
        label, wgsl, vertex_entry, fragment_entry, buffers, layouts,
        format, blend, write_mask, primitive, multisample, depth_stencil,
      } = self;

      let format = format.expect( "a render pipeline requires a color target format -- call `format`" );

      let shader = device.create_shader_module
      (
        wgpu::ShaderModuleDescriptor
        {
          label,
          source : wgpu::ShaderSource::Wgsl( wgsl.into() ),
        }
      );

      // wgpu 30 : bind group layout entries, and vertex buffer layout entries, are both
      // `Option`al ( a `None` leaves that slot unbound ), and push constants became
      // `immediate_size`. This builder appends dense slots, so every entry is `Some`.
      let buffers : Vec< Option< wgpu::VertexBufferLayout< '_ > > > = buffers.into_iter().map( Some ).collect();
      let layouts : Vec< Option< &wgpu::BindGroupLayout > > = layouts.into_iter().map( Some ).collect();
      let layout = device.create_pipeline_layout
      (
        &wgpu::PipelineLayoutDescriptor
        {
          label,
          bind_group_layouts : &layouts,
          immediate_size : 0
        }
      );

      device.create_render_pipeline
      (
        &wgpu::RenderPipelineDescriptor
        {
          label,
          layout : Some( &layout ),
          vertex : wgpu::VertexState
          {
            module : &shader,
            entry_point : Some( vertex_entry ),
            compilation_options : wgpu::PipelineCompilationOptions::default(),
            buffers : &buffers
          },
          primitive,
          depth_stencil,
          multisample,
          fragment : Some
          (
            wgpu::FragmentState
            {
              module : &shader,
              entry_point : Some( fragment_entry ),
              compilation_options : wgpu::PipelineCompilationOptions::default(),
              targets : &[ Some( wgpu::ColorTargetState { format, blend, write_mask } ) ]
            }
          ),
          multiview_mask : None,
          cache : None
        }
      )
    }
  }

  /// Creates a render pipeline for the bufferless-triangle pattern : `vs_main` derives
  /// positions from the vertex index ( no vertex buffers ), `fs_main` shades into a
  /// single color target of the given `format` with [`wgpu::BlendState::REPLACE`].
  ///
  /// A named shortcut over [`render_pipeline`] for the common shader-driven offscreen case;
  /// reach for the builder directly when vertex buffers, instancing, blending or depth are
  /// needed. Pair with [`crate::pass::draw_fullscreen`].
  ///
  /// The WGSL source must define `vs_main` and `fs_main` entry points. Bind group
  /// layouts in `layouts` become the pipeline layout; pass an empty slice when the
  /// shader binds nothing. Primitive, multisample and depth state are the `wgpu`
  /// defaults ( triangle list, no multisampling, no depth ).
  #[ must_use ]
  pub fn fullscreen
  (
    device : &wgpu::Device,
    wgsl : &str,
    format : wgpu::TextureFormat,
    layouts : &[ &wgpu::BindGroupLayout ]
  )
  -> wgpu::RenderPipeline
  {
    layouts.iter().copied().fold
    (
      render_pipeline().label( "fullscreen_pipeline" ).wgsl( wgsl ).format( format ),
      RenderPipelineBuilder::bind_group_layout,
    )
    .build( device )
  }
}

mod_interface!
{
  own use render_pipeline;
  own use RenderPipelineBuilder;
  own use fullscreen;
}
