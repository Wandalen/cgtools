//@ name: voronoi
//@ description: Cellular (Worley) F1 distance and cell id at a 2D point.
//@ tags: category:noise, technique:cellular
//@ depends_on: hash22
//@ export: fn voronoi(p: vec2f, jitter: f32, metric: f32, seed: f32) -> vec2f
//@ export: fn voronoi_preview(p: vec2f, jitter: f32, metric: f32, seed: f32) -> f32
//@ param: jitter argument f32 range(0.0, 2.0)
//@ param: metric argument f32 range(0.0, 4.0)
//@ param: seed argument f32 range(-50.0, 50.0)

// metric rounds to a discrete selector spanning the Minkowski Lp family,
// ordered by increasing p ( decreasing "pointiness" ): 0 = manhattan (p=1,
// diamond cells), 1 = p=1.5 (softened diamond), 2 = euclidean (p=2, round
// cells -- this chunk's original behavior), 3 = p=4 (softened square), 4 =
// chebyshev (p=infinity, square cells -- unreachable via pow(), computed
// directly). Euclidean sits at the range's midpoint deliberately -- sliders
// default to their midpoint, and euclidean is this chunk's original,
// pre-parameter look.
fn voronoi_metric_power( metric : f32 ) -> f32
{
  let m = u32( round( metric ) );
  let powers = array<f32, 4>( 1.0, 1.5, 2.0, 4.0 );
  // m == 4 ( chebyshev ) never reads this -- clamped so the lookup stays
  // in-bounds regardless, since WGSL's select() evaluates both branches.
  return powers[ min( m, 3u ) ];
}

fn voronoi_distance( delta : vec2f, m : u32, metric_power : f32 ) -> f32
{
  if( m == 4u )
  {
    return max( abs( delta.x ), abs( delta.y ) );
  }
  // Sum of |axis|^p, NOT yet rooted -- cheaper per-candidate than the true
  // distance and monotonic ( same argmin ), rooted once outside the search
  // loop in voronoi() below. At p=1 ( manhattan ) this is already the true
  // distance, since pow(x,1)+pow(y,1) needs no root.
  return pow( abs( delta.x ), metric_power ) + pow( abs( delta.y ), metric_power );
}

fn voronoi( p : vec2f, jitter : f32, metric : f32, seed : f32 ) -> vec2f
{
  let i = floor( p );
  let f = fract( p );
  // Search radius must grow with jitter: past unit jitter, a neighbor cell
  // `k` steps away can reach into `f`'s own [0, 1) span once jitter >= k
  // ( its point's nearest reachable coordinate is k - jitter ), so a fixed
  // 3x3 window silently misses the true nearest point above jitter = 1.
  // ceil(jitter) is the smallest radius that's always correct -- verified
  // by explicit construction ( jitter = 1.5: radius 1 provably misses a
  // real nearest point 2 cells out; radius 2 = ceil(1.5) never does ), not
  // just formula-matching. Holds for every metric above since each one is
  // >= chebyshev distance for the same delta, and the proof bounds the
  // ( metric-agnostic ) per-axis reach. jitter's own range caps at 2.0 with
  // its midpoint ( 1.0, this chunk's original hardcoded jitter ) as the
  // default, so this evaluates to the original fixed radius 1 there.
  let radius = i32( ceil( jitter ) );
  let m = u32( round( metric ) );
  let metric_power = voronoi_metric_power( metric );
  // Sentinel upper-bounds the true nearest-neighbor distance ( in whichever
  // surrogate units voronoi_distance above returns for this metric ), so
  // the first real candidate always beats it. Derived from cell 0's own
  // point alone ( always in-window ): its offset from `p` is strictly less
  // than `max( 1, jitter )` per axis ( hash/fract outputs are in [0, 1) ),
  // times a small margin against floating-point edge cases at that open
  // boundary. Sum-of-powers metrics ( manhattan through p=4 ) double it,
  // since both axes could hit their bound simultaneously; chebyshev takes
  // only the single ( larger ) axis, with no second term to add.
  let axis_bound = max( 1.0, jitter ) * 1.01;
  let sentinel = select( 2.0 * pow( axis_bound, metric_power ), axis_bound, m == 4u );
  var best = vec2f( sentinel, 0.0 );
  for( var y : i32 = -radius; y <= radius; y++ )
  {
    for( var x : i32 = -radius; x <= radius; x++ )
    {
      let cell = vec2f( f32( x ), f32( y ) );
      // `seed` offsets the integer lattice coordinate fed into the hash,
      // not `p` itself -- panning `p` by an integer amount just relabels
      // the same cell -> hash mapping across cells, whereas offsetting the
      // coordinate that's actually hashed reshuffles it, since hash22 has
      // no smoothness to preserve. seed = 0 reproduces this chunk's
      // original, unseeded pattern exactly, matching its range's midpoint.
      let rnd = hash22( i + cell + seed );
      let delta = cell + jitter * rnd - f;
      let dist = voronoi_distance( delta, m, metric_power );
      if( dist < best.x )
      {
        best = vec2f( dist, rnd.x );
      }
    }
  }
  // Undo the sum-of-powers surrogate from voronoi_distance above; chebyshev
  // never used one, so it's already a true distance.
  let true_dist = select( pow( best.x, 1.0 / metric_power ), best.x, m == 4u );
  return vec2f( true_dist, best.y );
}

fn voronoi_preview( p : vec2f, jitter : f32, metric : f32, seed : f32 ) -> f32
{
  return voronoi( p, jitter, metric, seed ).x;
}
