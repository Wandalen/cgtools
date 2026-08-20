//! Tests for `RenderPipelineBuilder`'s state accumulation — setters pinned through the
//! `*_get` getters — runnable without a GPU, following the same pattern as
//! `buffer_test.rs` ( only `build` needs a live `wgpu::Device` ).
//!
//! `bind_group_layout` is deliberately uncovered here : a `wgpu::BindGroupLayout` can only
//! be produced by a real device, so its accumulation is exercised through
//! `pipeline::fullscreen` in the live suite rather than headlessly.

use minwgpu::pipeline::render_pipeline;

#[ test ]
fn defaults_match_the_fullscreen_shortcut()
{
  let builder = render_pipeline();
  assert_eq!( builder.vertex_entry_get(), "vs_main" );
  assert_eq!( builder.fragment_entry_get(), "fs_main" );
  assert_eq!( builder.blend_get(), Some( wgpu::BlendState::REPLACE ) );
  assert_eq!( builder.write_mask_get(), wgpu::ColorWrites::ALL );
  assert!( builder.buffer_layouts_get().is_empty() );
  assert!( builder.bind_group_layouts_get().is_empty() );
  assert!( builder.depth_stencil_get().is_none() );
  assert!( builder.format_get().is_none(), "format has no default -- build must be told the target" );
}

#[ test ]
fn sets_label_and_wgsl()
{
  let builder = render_pipeline().label( "circles" ).wgsl( "@vertex fn vs_main() {}" );
  assert_eq!( builder.label_get(), Some( "circles" ) );
  assert_eq!( builder.wgsl_get(), "@vertex fn vs_main() {}" );
}

#[ test ]
fn sets_entry_points()
{
  let builder = render_pipeline().vertex_entry( "vertex" ).fragment_entry( "fragment" );
  assert_eq!( builder.vertex_entry_get(), "vertex" );
  assert_eq!( builder.fragment_entry_get(), "fragment" );
}

#[ test ]
fn sets_color_target_format()
{
  let builder = render_pipeline().format( wgpu::TextureFormat::Bgra8UnormSrgb );
  assert_eq!( builder.format_get(), Some( wgpu::TextureFormat::Bgra8UnormSrgb ) );
}

#[ test ]
fn sets_and_clears_blend()
{
  let builder = render_pipeline().blend( wgpu::BlendState::ALPHA_BLENDING );
  assert_eq!( builder.blend_get(), Some( wgpu::BlendState::ALPHA_BLENDING ) );
  assert_eq!( builder.blend_none().blend_get(), None );
}

#[ test ]
fn sets_write_mask()
{
  let builder = render_pipeline().write_mask( wgpu::ColorWrites::RED );
  assert_eq!( builder.write_mask_get(), wgpu::ColorWrites::RED );
}

/// Vertex buffer layouts must accumulate in call order — the pipeline binds them to
/// consecutive slots, so a reordering here would silently mis-bind every attribute.
#[ test ]
fn buffer_layouts_accumulate_in_slot_order()
{
  const QUAD : [ wgpu::VertexAttribute; 1 ] =
    [ minwgpu::helper::attr( wgpu::VertexFormat::Float32x2, 0, 0 ) ];
  const INSTANCE : [ wgpu::VertexAttribute; 1 ] =
    [ minwgpu::helper::attr( wgpu::VertexFormat::Float32x3, 0, 1 ) ];

  let quad = wgpu::VertexBufferLayout
  {
    array_stride : 8,
    step_mode : wgpu::VertexStepMode::Vertex,
    attributes : &QUAD,
  };
  let instance = wgpu::VertexBufferLayout
  {
    array_stride : 12,
    step_mode : wgpu::VertexStepMode::Instance,
    attributes : &INSTANCE,
  };

  let builder = render_pipeline().buffer_layout( quad ).buffer_layout( instance );
  let layouts = builder.buffer_layouts_get();

  assert_eq!( layouts.len(), 2 );
  assert_eq!( layouts[ 0 ].array_stride, 8 );
  assert_eq!( layouts[ 0 ].step_mode, wgpu::VertexStepMode::Vertex );
  assert_eq!( layouts[ 1 ].array_stride, 12 );
  assert_eq!
  (
    layouts[ 1 ].step_mode,
    wgpu::VertexStepMode::Instance,
    "the instance-stepped layout is what makes a draw instanced"
  );
}

#[ test ]
fn sets_primitive_and_multisample()
{
  let primitive = wgpu::PrimitiveState
  {
    topology : wgpu::PrimitiveTopology::LineList,
    ..wgpu::PrimitiveState::default()
  };
  let multisample = wgpu::MultisampleState { count : 4, ..wgpu::MultisampleState::default() };

  let builder = render_pipeline().primitive( primitive ).multisample( multisample );
  assert_eq!( builder.primitive_get().topology, wgpu::PrimitiveTopology::LineList );
  assert_eq!( builder.multisample_get().count, 4 );
}

#[ test ]
fn sets_depth_stencil()
{
  let depth = wgpu::DepthStencilState
  {
    format : wgpu::TextureFormat::Depth32Float,
    depth_write_enabled : Some( true ),
    depth_compare : Some( wgpu::CompareFunction::Less ),
    stencil : wgpu::StencilState::default(),
    bias : wgpu::DepthBiasState::default(),
  };

  let builder = render_pipeline().depth_stencil( depth );
  let stored = builder.depth_stencil_get().expect( "depth state was just set" );
  assert_eq!( stored.format, wgpu::TextureFormat::Depth32Float );
  assert_eq!( stored.depth_write_enabled, Some( true ) );
  assert_eq!( stored.depth_compare, Some( wgpu::CompareFunction::Less ) );
}
