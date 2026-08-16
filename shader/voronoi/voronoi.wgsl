//@ name: voronoi
//@ description: Cellular (Worley) F1 distance and cell id at a 2D point.
//@ tags: category:noise, technique:cellular
//@ depends_on: hash22
//@ export: fn voronoi(p: vec2f, jitter: f32) -> vec2f
//@ export: fn voronoi_preview(p: vec2f, jitter: f32) -> f32
//@ param: jitter argument f32 range(0.0, 1.0)

fn voronoi( p : vec2f, jitter : f32 ) -> vec2f
{
  let i = floor( p );
  let f = fract( p );
  // ( squared distance to nearest feature point, that cell's id ).
  var best = vec2f( 8.0, 0.0 );
  for( var y : i32 = -1; y <= 1; y++ )
  {
    for( var x : i32 = -1; x <= 1; x++ )
    {
      let cell = vec2f( f32( x ), f32( y ) );
      let rnd = hash22( i + cell );
      // rnd is in [0, 1) -- jitter is capped at 1.0 so the offset feature
      // point never leaves the fixed 3x3 neighbor search below.
      let delta = cell + jitter * rnd - f;
      let dist = dot( delta, delta );
      if( dist < best.x )
      {
        best = vec2f( dist, rnd.x );
      }
    }
  }
  return vec2f( sqrt( best.x ), best.y );
}

fn voronoi_preview( p : vec2f, jitter : f32 ) -> f32
{
  return voronoi( p, jitter ).x;
}
