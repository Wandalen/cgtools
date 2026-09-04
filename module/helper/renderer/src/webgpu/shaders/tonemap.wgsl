// Fullscreen ACES tone mapping + sRGB encode (WGSL).
//
// Port of `../../webgl/shaders/tonemapping/aces.frag` (three.js-matching
// ACESFilmic fit) and `post_processing/to_srgb.frag`, fused into one pass:
// the WebGL chain runs them as two composer steps, but this slice has no
// display-referred passes between them, so one draw does both. The vertex
// stage is the `big_triangle.vert` trick: one oversized triangle from
// `vertex_index`, no vertex buffers.

@group( 0 ) @binding( 0 ) var source_texture : texture_2d< f32 >;

struct VertexOutput
{
  @builtin( position ) clip_position : vec4f,
}

@vertex
fn vs_main( @builtin( vertex_index ) vertex_index : u32 ) -> VertexOutput
{
  let x = f32( vertex_index / 2u );
  let y = f32( vertex_index % 2u );

  var out : VertexOutput;
  out.clip_position = vec4f( x * 4.0 - 1.0, y * 4.0 - 1.0, 0.0, 1.0 );
  return out;
}

fn aces_tone_map( hdr : vec3f ) -> vec3f
{
  let m1 = mat3x3f
  (
    vec3f( 0.59719, 0.07600, 0.02840 ),
    vec3f( 0.35458, 0.90834, 0.13383 ),
    vec3f( 0.04823, 0.01566, 0.83777 )
  );
  let m2 = mat3x3f
  (
    vec3f( 1.60475, -0.10208, -0.00327 ),
    vec3f( -0.53108, 1.10813, -0.07276 ),
    vec3f( -0.07367, -0.00605, 1.07602 )
  );

  // Pre-exposure RRT scaling, matching three.js ACESFilmicToneMapping.
  let v = m1 * ( hdr / 0.6 );
  let a = v * ( v + 0.0245786 ) - 0.000090537;
  let b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;

  return clamp( m2 * ( a / b ), vec3f( 0.0 ), vec3f( 1.0 ) );
}

fn linear_to_srgb( color : vec3f ) -> vec3f
{
  let more = pow( color, vec3f( 0.41666 ) ) * 1.055 - vec3f( 0.055 );
  let less = color * 12.92;

  return select( more, less, color <= vec3f( 0.0031308 ) );
}

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  let src = textureLoad( source_texture, vec2i( floor( in.clip_position.xy ) ), 0 );

  // Background pixels are cleared with alpha = 0 and bypass tone mapping
  // (as the clear color does in three.js); geometry writes alpha = 1.
  let mapped = mix( src.rgb, aces_tone_map( src.rgb ), src.a );

  return vec4f( linear_to_srgb( mapped ), 1.0 );
}
