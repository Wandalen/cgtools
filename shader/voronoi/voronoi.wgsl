//@ name: voronoi
//@ description: Cellular (Worley) F1 distance and cell id at a 2D point.
//@ tags: category:noise, technique:cellular
//@ depends_on: hash22
//@ export: fn voronoi(p: vec2f) -> vec2f

fn voronoi( p : vec2f ) -> vec2f
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
      let jitter = hash22( i + cell );
      let delta = cell + jitter - f;
      let dist = dot( delta, delta );
      if( dist < best.x )
      {
        best = vec2f( dist, jitter.x );
      }
    }
  }
  return vec2f( sqrt( best.x ), best.y );
}
