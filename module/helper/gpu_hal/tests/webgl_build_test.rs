//! `webgl_build` tests : WGSL→GLSL ES 300 translation and the uniform
//! block / texture-sampler renaming that lets gpu_hal's WebGL introspection
//! ( `webgl_bindings_introspect`, `device.rs` ) find bindings by a fixed
//! `ub_{group}_{binding}` / `tex_{group}_{binding}` name — never a
//! naga-generated one. Runs on any host : the `webgl-glsl-build` feature is
//! independent of `webgl`'s wasm32-only deps.
#![ cfg( feature = "webgl-glsl-build" ) ]

use gpu_hal::webgl_build::wgsl_to_webgl_glsl;

/// Mirrors `renderer`'s real `main.wgsl` binding shape : `camera` shared by
/// both stages, `model` referenced only by the vertex stage, `material` and
/// the base-color texture/sampler pair referenced only by the fragment
/// stage — the exact layout that exposed the original "fragment-only
/// rename" defect ( `model` would never be renamed, breaking the transform
/// at runtime ).
const MIXED_BINDINGS_WGSL : &str = "
struct Camera { view_matrix : mat4x4f }
struct Model { world_matrix : mat4x4f }
struct Material { base_color_factor : vec4f }

@group( 0 ) @binding( 0 ) var< uniform > camera : Camera;
@group( 1 ) @binding( 0 ) var< uniform > material : Material;
@group( 1 ) @binding( 1 ) var base_color_texture : texture_2d< f32 >;
@group( 1 ) @binding( 2 ) var base_color_sampler : sampler;
@group( 2 ) @binding( 0 ) var< uniform > model : Model;

struct VertexOutput
{
  @builtin( position ) clip_position : vec4f,
  @location( 0 ) uv : vec2f,
}

@vertex
fn vs_main( @location( 0 ) position : vec3f, @location( 1 ) uv : vec2f ) -> VertexOutput
{
  var out : VertexOutput;
  out.clip_position = camera.view_matrix * model.world_matrix * vec4f( position, 1.0 );
  out.uv = uv;
  return out;
}

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  let sampled = textureSample( base_color_texture, base_color_sampler, in.uv );
  let exposure = camera.view_matrix[ 0 ].x;
  return sampled * material.base_color_factor * exposure;
}
";

#[ test ]
fn vertex_and_fragment_uniform_blocks_renamed_independently_per_stage()
{
  let source = wgsl_to_webgl_glsl( MIXED_BINDINGS_WGSL, "vs_main", "fs_main" ).unwrap();

  assert!( source.vertex.contains( "ub_0_0" ), "vertex output missing shared camera block :\n{}", source.vertex );
  assert!( source.vertex.contains( "ub_2_0" ), "vertex output missing vertex-only model block :\n{}", source.vertex );
  assert!( !source.vertex.contains( "ub_1_0" ), "vertex output must not carry the fragment-only material block :\n{}", source.vertex );

  assert!( source.fragment.contains( "ub_0_0" ), "fragment output missing shared camera block :\n{}", source.fragment );
  assert!( source.fragment.contains( "ub_1_0" ), "fragment output missing material block :\n{}", source.fragment );
  assert!( !source.fragment.contains( "ub_2_0" ), "fragment output must not carry the vertex-only model block :\n{}", source.fragment );
}

#[ test ]
fn texture_sampler_pair_renamed_to_texture_binding()
{
  let source = wgsl_to_webgl_glsl( MIXED_BINDINGS_WGSL, "vs_main", "fs_main" ).unwrap();

  assert!( source.fragment.contains( "tex_1_1" ), "fragment output missing renamed base-color sampler :\n{}", source.fragment );
}

/// Mirrors `renderer`'s real `tonemap.wgsl` : a `texelFetch`-style texture
/// with no paired WGSL `sampler`, and a vertex stage with zero bindings at
/// all ( the "big triangle" trick, driven only by `vertex_index` ).
const TEXTURE_ONLY_WGSL : &str = "
@group( 0 ) @binding( 0 ) var source_texture : texture_2d< f32 >;

@vertex
fn vs_main( @builtin( vertex_index ) vertex_index : u32 ) -> @builtin( position ) vec4f
{
  let x = f32( vertex_index / 2u );
  let y = f32( vertex_index % 2u );
  return vec4f( x * 4.0 - 1.0, y * 4.0 - 1.0, 0.0, 1.0 );
}

@fragment
fn fs_main( @builtin( position ) frag_coord : vec4f ) -> @location( 0 ) vec4f
{
  return textureLoad( source_texture, vec2i( floor( frag_coord.xy ) ), 0 );
}
";

#[ test ]
fn texture_without_sampler_renamed_and_bindingless_stage_translates()
{
  let source = wgsl_to_webgl_glsl( TEXTURE_ONLY_WGSL, "vs_main", "fs_main" ).unwrap();

  assert!( source.fragment.contains( "tex_0_0" ), "fragment output missing renamed sampler-less texture :\n{}", source.fragment );
  assert!( !source.vertex.trim().is_empty(), "vertex stage with zero bindings must still translate" );
}

#[ test ]
fn multiple_textures_do_not_collide()
{
  let wgsl = "
  @group( 1 ) @binding( 1 ) var base_color_texture : texture_2d< f32 >;
  @group( 1 ) @binding( 2 ) var base_color_sampler : sampler;
  @group( 1 ) @binding( 3 ) var mr_texture : texture_2d< f32 >;
  @group( 1 ) @binding( 4 ) var mr_sampler : sampler;

  @vertex
  fn vs_main( @location( 0 ) position : vec3f ) -> @builtin( position ) vec4f
  {
    return vec4f( position, 1.0 );
  }

  @fragment
  fn fs_main( @location( 0 ) uv : vec2f ) -> @location( 0 ) vec4f
  {
    let base = textureSample( base_color_texture, base_color_sampler, uv );
    let mr = textureSample( mr_texture, mr_sampler, uv );
    return base + mr;
  }
  ";

  let source = wgsl_to_webgl_glsl( wgsl, "vs_main", "fs_main" ).unwrap();

  assert!( source.fragment.contains( "tex_1_1" ), "missing first texture rename :\n{}", source.fragment );
  assert!( source.fragment.contains( "tex_1_3" ), "missing second texture rename :\n{}", source.fragment );
}

#[ test ]
fn invalid_wgsl_syntax_returns_err()
{
  let result = wgsl_to_webgl_glsl( "this is not valid wgsl {{{", "vs_main", "fs_main" );
  assert!( result.is_err() );
}

#[ test ]
fn invalid_wgsl_semantics_returns_err()
{
  let wgsl = "
  @vertex
  fn vs_main() -> @builtin( position ) vec4f
  {
    let mismatched : f32 = vec3f( 1.0, 2.0, 3.0 );
    return vec4f( mismatched, 0.0, 0.0, 1.0 );
  }

  @fragment
  fn fs_main() -> @location( 0 ) vec4f
  {
    return vec4f( 1.0 );
  }
  ";

  let result = wgsl_to_webgl_glsl( wgsl, "vs_main", "fs_main" );
  assert!( result.is_err() );
}

#[ test ]
fn unknown_entry_point_returns_err()
{
  let result = wgsl_to_webgl_glsl( MIXED_BINDINGS_WGSL, "does_not_exist", "fs_main" );
  assert!( result.is_err() );
}
