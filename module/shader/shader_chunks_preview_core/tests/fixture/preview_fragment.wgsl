//@ name: preview_fragment
//@ description: Warped fbm3 noise field with live-tunable frequency, domain-warp strength, and brightness.
//@ tags: category:scene
//@ stage: fragment
//@ depends_on: hash21, fbm3, fullscreen_triangle
//@ export: fn fs_main(in: VertexOutput) -> @location(0) vec4f
//@ param: noise_scale uniform f32 range(0.5, 20.0)
//@ param: warp_strength uniform f32 range(0.0, 2.0)
//@ param: brightness uniform f32 range(0.0, 3.0)

// Renders shader_chunks_core's noise stack ( hash21 -> value_noise -> fbm3 )
// as a fullscreen domain-warped fbm field: fbm3 is sampled once to build a
// 2D warp offset, then sampled again at the warped point -- the classic
// "warped fbm" look -- so every tunable below visibly changes the picture
// rather than sitting inert. This is shader_chunk_preview's only local
// chunk; the three chunks it depends on come from shader_chunks_core
// unmodified -- see src/shader_source.rs's PREVIEW_CHUNKS for the composed
// set.

struct Params
{
  time : f32,
  noise_scale : f32,
  warp_strength : f32,
  brightness : f32,
  resolution : vec4f, // .xy = physical pixels, .zw unused
}

@group( 0 ) @binding( 0 ) var< uniform > params : Params;

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  let aspect = params.resolution.x / max( params.resolution.y, 1.0 );
  let center = vec2f( 0.5, 0.5 );
  let q = ( in.uv - center ) * vec2f( aspect, 1.0 );
  let p = q * params.noise_scale;
  let drift = params.time * 0.05;

  // Domain warp: offset the sample point by a second, independently-phased
  // fbm3 field before the final sample -- without this, `warp_strength`
  // would have nothing to multiply and the picture would just be plain fbm.
  let warp = vec2f
  (
    fbm3( p + drift ),
    fbm3( p + vec2f( 5.2, 1.3 ) + drift ),
  );
  let n = fbm3( p + ( warp - 0.4375 ) * params.warp_strength * 4.0 );

  let color = vec3f( n ) * params.brightness;
  return vec4f( color, 1.0 );
}
