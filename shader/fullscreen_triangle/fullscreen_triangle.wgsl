//@ name: fullscreen_triangle
//@ description: Fullscreen-triangle vertex stage: 3 vertices, no vertex buffer, vertex_index alone picks the corner.
//@ tags: category:vertex
//@ stage: vertex
//@ depends_on:
//@ export: struct VertexOutput { position: vec4f, uv: vec2f }
//@ export: fn vs_main(vertex_index: u32) -> VertexOutput

struct VertexOutput
{
  @builtin( position ) position : vec4f,
  @location( 0 ) uv : vec2f,
}

@vertex
fn vs_main( @builtin( vertex_index ) vertex_index : u32 ) -> VertexOutput
{
  // Big-triangle trick: 3 vertices, no buffer, vertex_index picks the corner.
  // The triangle overshoots clip space; only the visible unit square of uv
  // ( bottom-left = (0,0), top-right = (1,1) ) is ever rasterized to pixels.
  let x = i32( vertex_index ) & 1;
  let y = i32( vertex_index ) / 2;
  let uv = vec2f( f32( x ) * 2.0, f32( y ) * 2.0 );

  var out : VertexOutput;
  out.uv = uv;
  out.position = vec4f( uv * 2.0 - 1.0, 0.0, 1.0 );
  return out;
}
