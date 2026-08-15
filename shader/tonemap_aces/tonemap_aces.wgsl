//@ name: tonemap_aces
//@ description: ACES filmic tone map from HDR to [0, 1], the Hill fit with three.js-style pre-exposure.
//@ tags: category:color
//@ depends_on:
//@ export: fn tonemap_aces(hdr: vec3f) -> vec3f
//@ export: fn tonemap_aces_preview(p: vec2f) -> vec3f

fn tonemap_aces( hdr : vec3f ) -> vec3f
{
  // Stephen Hill's ACES fit — constants match the workspace's GLSL
  // `renderer` implementation ( webgl tonemapping shader ) verbatim, so the
  // WebGPU and WebGL2 paths grade identically.
  let m1 = mat3x3f
  (
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777
  );
  let m2 = mat3x3f
  (
    1.60475, -0.10208, -0.00327,
    -0.53108,  1.10813, -0.07276,
    -0.07367, -0.00605,  1.07602
  );
  // Pre-exposure RRT scaling, matching three.js ACESFilmicToneMapping.
  let v = m1 * ( hdr / 0.6 );
  let a = v * ( v + 0.0245786 ) - 0.000090537;
  let b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;
  return clamp( m2 * ( a / b ), vec3f( 0.0 ), vec3f( 1.0 ) );
}

fn tonemap_aces_preview( p : vec2f ) -> vec3f
{
  let hdr = vec3f( p.x );
  if ( p.y > 0.0 )
  {
    return clamp( hdr, vec3f( 0.0 ), vec3f( 1.0 ) );
  }
  return tonemap_aces( hdr );
}
