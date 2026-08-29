//! Every function timed and compared against this host's libm, printed as one table.
//!
//! ```sh
//! cargo run -p deterministic_math --release --example cost_vs_libm
//! ```
//!
//! `--release` is not optional. A debug build measures the absence of inlining rather than the
//! kernel, and reports ratios several times the real ones.
//!
//! What each column means, and what it does not:
//!
//! - **det / libm ns** — the minimum over [ `ROUNDS` ] rounds of [ `SAMPLES` ] calls each, minimum
//!   rather than mean because the noise here is one-sided: a slow round is a scheduler artefact, a
//!   fast round cannot be one.
//! - **ratio** — the per-call cost of reproducibility on this host, for this domain, at this
//!   optimisation level. It is not portable and is not meant to be.
//! - **max Δulp** — the largest disagreement with the host's libm across the sampled domain, in
//!   units of last place. This is an *agreement* measure, not an accuracy one: libm is not the true
//!   value, so a nonzero entry says the two implementations differ, never which is closer.
//!   Correctly-rounded reference comparison is a different exercise and is not this one.
//!
//! The domains are stated per row because a ratio without its domain is unfalsifiable — `exp` over
//! `[ -1, 1 ]` and `exp` over `[ -700, 700 ]` are different measurements of the same function.
//!
//! Inputs come from a fixed-seed generator, so two runs on one host sample identical points and the
//! Δulp column is reproducible rather than a new draw each time.

use deterministic_math as dm;
use std::hint::black_box;
use std::time::Instant;

/// Calls per timing round.
const SAMPLES : usize = 200_000;

/// Timing rounds; the minimum across them is reported.
const ROUNDS : usize = 7;

/// A fixed-seed xorshift64*, so the sampled points are the same on every run and every host.
///
/// The kernel is not under test here — its correctness is `tests/`. This generator exists only so
/// the domain is covered evenly and identically run to run.
struct Rng( u64 );

impl Rng
{
  fn new() -> Self
  {
    Self( 0x2545_F491_4F6C_DD1D )
  }

  /// Next raw word.
  fn next_u64( &mut self ) -> u64
  {
    let mut x = self.0;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    self.0 = x;
    x.wrapping_mul( 0x2545_F491_4F6C_DD1D )
  }

  /// Next value uniform on `[ lo, hi ]`.
  fn next_in( &mut self, lo : f64, hi : f64 ) -> f64
  {
    let unit = ( self.next_u64() >> 11 ) as f64 / ( 1_u64 << 53 ) as f64;
    lo + unit * ( hi - lo )
  }
}

/// Distance between two finites in units of last place, saturating at [ `u64::MAX` ].
///
/// Monotone bit ordering only holds within one sign, so the mixed-sign case is measured as the two
/// distances to zero added together rather than by subtracting the raw patterns — which would
/// report two adjacent values straddling zero as astronomically far apart.
#[ allow( clippy::float_cmp, reason = "zero-distance fast path; `==` also folds ±0.0 together, which `to_bits` would not" ) ]
fn ulp_diff( a : f64, b : f64 ) -> u64
{
  if a == b
  {
    return 0;
  }
  if !a.is_finite() || !b.is_finite()
  {
    return if a.is_nan() && b.is_nan() { 0 } else { u64::MAX };
  }

  let key = | v : f64 |
  {
    let bits = v.to_bits();
    if v.is_sign_negative() { ( 0x8000_0000_0000_0000_u64 ).wrapping_sub( bits & 0x7FFF_FFFF_FFFF_FFFF ) } else { bits | 0x8000_0000_0000_0000 }
  };

  key( a ).abs_diff( key( b ) )
}

/// One measured row.
struct Row
{
  name : &'static str,
  domain : &'static str,
  det_ns : f64,
  libm_ns : f64,
  max_ulp : u64,
  /// Input at which `max_ulp` occurred, with both results — without it the Δulp column is a number
  /// nobody can act on. A large Δulp at a point where the result is near zero is cancellation, not
  /// disagreement about the function; the two cases are indistinguishable from the count alone and
  /// are told apart here.
  worst_at : ( f64, f64, f64, f64 ),
}

/// Time `f` over `inputs`, returning the minimum per-call nanoseconds across [ `ROUNDS` ].
fn time< F >( inputs : &[ ( f64, f64 ) ], mut f : F ) -> f64
where
  F : FnMut( f64, f64 ) -> f64,
{
  let mut best = f64::INFINITY;
  for _ in 0..ROUNDS
  {
    let start = Instant::now();
    let mut acc = 0.0_f64;
    for &( x, y ) in inputs
    {
      acc += black_box( f( black_box( x ), black_box( y ) ) );
    }
    black_box( acc );
    let ns = start.elapsed().as_secs_f64() * 1.0e9 / inputs.len() as f64;
    if ns < best
    {
      best = ns;
    }
  }
  best
}

/// Measure one function both ways over a freshly drawn domain.
///
/// The two argument ranges arrive as pairs rather than as four scalars so the signature stays under
/// the workspace's argument-count lint, and so a call site cannot silently transpose a bound.
fn measure< P, L >( name : &'static str, domain : &'static str, xr : ( f64, f64 ), yr : ( f64, f64 ), mut p : P, mut l : L ) -> Row
where
  P : FnMut( f64, f64 ) -> f64,
  L : FnMut( f64, f64 ) -> f64,
{
  let mut rng = Rng::new();
  let inputs : Vec< ( f64, f64 ) > = ( 0..SAMPLES ).map( | _ | ( rng.next_in( xr.0, xr.1 ), rng.next_in( yr.0, yr.1 ) ) ).collect();

  let mut max_ulp = 0_u64;
  let mut worst_at = ( 0.0, 0.0, 0.0, 0.0 );
  for &( x, y ) in &inputs
  {
    let ( pv, lv ) = ( p( x, y ), l( x, y ) );
    let d = ulp_diff( pv, lv );
    if d > max_ulp
    {
      max_ulp = d;
      worst_at = ( x, y, pv, lv );
    }
  }

  Row
  {
    name,
    domain,
    det_ns : time( &inputs, &mut p ),
    libm_ns : time( &inputs, &mut l ),
    max_ulp,
    worst_at,
  }
}

/// Measure a one-argument function; the second sampled column is ignored.
fn measure1< P, L >( name : &'static str, domain : &'static str, lo : f64, hi : f64, mut p : P, mut l : L ) -> Row
where
  P : FnMut( f64 ) -> f64,
  L : FnMut( f64 ) -> f64,
{
  measure( name, domain, ( lo, hi ), ( 0.0, 1.0 ), move | x, _ | p( x ), move | x, _ | l( x ) )
}

/// `sin_cos` needs its own measurement because it returns a pair.
///
/// Timing sums the two halves so neither is optimised away, but the Δulp is taken as the worse of
/// the two halves compared *separately*. Scoring the sum instead would measure this harness rather
/// than the kernel: `s + c` passes through zero wherever `tan x = -1`, and the resulting
/// cancellation reported 131 072 Δulp for a pair whose halves each agree to 2.
fn measure_sin_cos() -> Row
{
  let mut rng = Rng::new();
  let inputs : Vec< ( f64, f64 ) > = ( 0..SAMPLES ).map( | _ | ( rng.next_in( -100.0, 100.0 ), 0.0 ) ).collect();

  let mut max_ulp = 0_u64;
  let mut worst_at = ( 0.0, 0.0, 0.0, 0.0 );
  for &( x, _ ) in &inputs
  {
    let ( ps, pc ) = dm::sin_cos( x );
    let ( ls, lc ) = x.sin_cos();
    for ( pv, lv ) in [ ( ps, ls ), ( pc, lc ) ]
    {
      let d = ulp_diff( pv, lv );
      if d > max_ulp
      {
        max_ulp = d;
        worst_at = ( x, 0.0, pv, lv );
      }
    }
  }

  Row
  {
    name : "sin_cos",
    domain : "[ -100, 100 ]",
    det_ns : time( &inputs, | x, _ | { let ( s, c ) = dm::sin_cos( x ); s + c } ),
    libm_ns : time( &inputs, | x, _ | { let ( s, c ) = x.sin_cos(); s + c } ),
    max_ulp,
    worst_at,
  }
}

/// The whole surface, one row each.
fn all_rows() -> Vec< Row >
{
  vec!
  [
    // Algebraic — already pinned by IEEE-754, measured to show they cost nothing.
    measure1( "sqrt",    "[ 0, 1e6 ]",       0.0,    1.0e6,  dm::sqrt,  f64::sqrt ),
    measure( "mul_add",  "x,y ∈ [ -100, 100 ], c = 1", ( -100.0, 100.0 ), ( -100.0, 100.0 ), | x, y | dm::mul_add( x, y, 1.0 ), | x, y | x.mul_add( y, 1.0 ) ),
    measure( "hypot",    "x,y ∈ [ -1e6, 1e6 ]", ( -1.0e6, 1.0e6 ), ( -1.0e6, 1.0e6 ), dm::hypot, f64::hypot ),
    measure1( "powi",    "[ -10, 10 ], n = 7", -10.0, 10.0,  | x | dm::powi( x, 7 ), | x : f64 | x.powi( 7 ) ),

    // Exponential and logarithmic.
    measure1( "exp",     "[ -20, 20 ]",      -20.0,  20.0,   dm::exp,    f64::exp ),
    measure1( "exp2",    "[ -20, 20 ]",      -20.0,  20.0,   dm::exp2,   f64::exp2 ),
    measure1( "exp_m1",  "[ -1, 1 ]",        -1.0,   1.0,    dm::exp_m1, f64::exp_m1 ),
    measure1( "ln",      "[ 1e-6, 1e6 ]",    1.0e-6, 1.0e6,  dm::ln,     f64::ln ),
    measure1( "ln_1p",   "[ -0.5, 1 ]",      -0.5,   1.0,    dm::ln_1p,  f64::ln_1p ),
    measure1( "log2",    "[ 1e-6, 1e6 ]",    1.0e-6, 1.0e6,  dm::log2,   f64::log2 ),
    measure1( "log10",   "[ 1e-6, 1e6 ]",    1.0e-6, 1.0e6,  dm::log10,  f64::log10 ),
    measure( "log",      "x ∈ [ 1e-6, 1e6 ], base ∈ [ 2, 10 ]", ( 1.0e-6, 1.0e6 ), ( 2.0, 10.0 ), dm::log, f64::log ),
    measure( "powf",     "x ∈ [ 0.1, 10 ], y ∈ [ -5, 5 ]", ( 0.1, 10.0 ), ( -5.0, 5.0 ), dm::powf, f64::powf ),
    measure1( "cbrt",    "[ -1e6, 1e6 ]",    -1.0e6, 1.0e6,  dm::cbrt,   f64::cbrt ),

    // Circular.
    measure1( "sin",     "[ -100, 100 ]",    -100.0, 100.0,  dm::sin,    f64::sin ),
    measure1( "cos",     "[ -100, 100 ]",    -100.0, 100.0,  dm::cos,    f64::cos ),
    measure1( "tan",     "[ -1.5, 1.5 ]",    -1.5,   1.5,    dm::tan,    f64::tan ),
    measure_sin_cos(),

    // Inverse circular.
    measure1( "atan",    "[ -100, 100 ]",    -100.0, 100.0,  dm::atan,   f64::atan ),
    measure( "atan2",    "y,x ∈ [ -100, 100 ]", ( -100.0, 100.0 ), ( -100.0, 100.0 ), dm::atan2, f64::atan2 ),
    measure1( "asin",    "[ -1, 1 ]",        -1.0,   1.0,    dm::asin,   f64::asin ),
    measure1( "acos",    "[ -1, 1 ]",        -1.0,   1.0,    dm::acos,   f64::acos ),

    // Hyperbolic.
    measure1( "sinh",    "[ -20, 20 ]",      -20.0,  20.0,   dm::sinh,   f64::sinh ),
    measure1( "cosh",    "[ -20, 20 ]",      -20.0,  20.0,   dm::cosh,   f64::cosh ),
    measure1( "tanh",    "[ -20, 20 ]",      -20.0,  20.0,   dm::tanh,   f64::tanh ),
    measure1( "asinh",   "[ -100, 100 ]",    -100.0, 100.0,  dm::asinh,  f64::asinh ),
    measure1( "acosh",   "[ 1, 100 ]",       1.0,    100.0,  dm::acosh,  f64::acosh ),
    measure1( "atanh",   "[ -0.99, 0.99 ]",  -0.99,  0.99,   dm::atanh,  f64::atanh ),
  ]
}

fn main()
{
  let rows = all_rows();

  println!( "deterministic_math vs host libm — {SAMPLES} samples, best of {ROUNDS} rounds" );
  println!( "{} functions measured", rows.len() );
  println!();
  println!( "| function | domain | det ns | libm ns | ratio | max Δulp |" );
  println!( "|----------|--------|--------|---------|-------|----------|" );
  for r in &rows
  {
    println!
    (
      "| `{}` | `{}` | {:.2} | {:.2} | {:.2}x | {} |",
      r.name, r.domain, r.det_ns, r.libm_ns, r.det_ns / r.libm_ns, r.max_ulp
    );
  }

  let worst = rows.iter().max_by( | a, b | ( a.det_ns / a.libm_ns ).total_cmp( &( b.det_ns / b.libm_ns ) ) );
  if let Some( w ) = worst
  {
    println!();
    println!( "worst ratio: {} at {:.2}x", w.name, w.det_ns / w.libm_ns );
  }

  println!();
  println!( "where each max Δulp occurred — `rel` is the relative difference, which is the number" );
  println!( "that says whether a large Δulp is disagreement or cancellation near a zero:" );
  println!();
  println!( "| function | max Δulp | input | det | libm | rel |" );
  println!( "|----------|----------|-------|-----|------|-----|" );
  for r in rows.iter().filter( | r | r.max_ulp > 0 )
  {
    let ( x, y, pv, lv ) = r.worst_at;
    let rel = if lv == 0.0 { f64::INFINITY } else { ( ( pv - lv ) / lv ).abs() };
    println!( "| `{}` | {} | {:.6e}, {:.6e} | {:.17e} | {:.17e} | {:.2e} |", r.name, r.max_ulp, x, y, pv, lv, rel );
  }

  small_argument_sweep();
}

/// Relative disagreement with libm as the argument shrinks, for the functions that tend to the
/// identity at zero.
///
/// This sweep is the regression guard for the small-argument work. Seven of these functions once
/// disagreed with libm by 1e-11 to 1e-13 relative below `x ≈ 1e-4`, fanning out steadily as the
/// argument shrank — a mechanism rather than an unlucky sample, which is exactly what a decade
/// sweep can show and a single worst-case number cannot. Every column should now read at or near
/// `1e-16`. A column that starts fanning out to the right again means a reduction or a series band
/// has regressed.
fn small_argument_sweep()
{
  /// One `f64 -> f64` entry, named so the sweep's table below stays under the complexity lint.
  type Unary = fn( f64 ) -> f64;

  let fns : [ ( &str, Unary, Unary ) ; 8 ] =
  [
    ( "exp_m1", dm::exp_m1, f64::exp_m1 ),
    ( "ln_1p",  dm::ln_1p,  f64::ln_1p ),
    ( "asin",   dm::asin,   f64::asin ),
    ( "atan",   dm::atan,   f64::atan ),
    ( "sinh",   dm::sinh,   f64::sinh ),
    ( "tanh",   dm::tanh,   f64::tanh ),
    ( "asinh",  dm::asinh,  f64::asinh ),
    ( "atanh",  dm::atanh,  f64::atanh ),
  ];

  println!();
  println!( "relative disagreement with libm by argument size — `x` chosen off a decade so the" );
  println!( "sample is not a value either implementation special-cases:" );
  println!();
  print!( "| function |" );
  for e in 1..=8 { print!( " x≈1e-{e} |" ); }
  println!();
  print!( "|----------|" );
  for _ in 1..=8 { print!( "---------|" ); }
  println!();

  for ( name, p, l ) in fns
  {
    print!( "| `{name}` |" );
    for e in 1..=8
    {
      let x = 3.7_f64 * 10.0_f64.powi( -e );
      let ( pv, lv ) = ( p( x ), l( x ) );
      let rel = if lv == 0.0 { 0.0 } else { ( ( pv - lv ) / lv ).abs() };
      print!( " {rel:.0e} |" );
    }
    println!();
  }

  // The two functions whose whole reason for existing is small-argument precision are also measured
  // against the naive substitute they replaced, since "better than naive" and "as good as libm" are
  // different claims and the crate makes both.
  println!();
  println!( "`exp_m1` and `ln_1p` against the naive substitutes they exist to replace:" );
  println!();
  println!( "| x | dm::exp_m1 rel | naive exp(x)-1 rel | dm::ln_1p rel | naive ln(1+x) rel |" );
  println!( "|---|----------------|--------------------|---------------|-------------------|" );
  for e in 2..=8
  {
    let x = 3.7_f64 * 10.0_f64.powi( -e );
    let rel = | got : f64, want : f64 | ( ( got - want ) / want ).abs();
    println!
    (
      "| 3.7e-{} | {:.0e} | {:.0e} | {:.0e} | {:.0e} |",
      e,
      rel( dm::exp_m1( x ), x.exp_m1() ),
      rel( dm::exp( x ) - 1.0, x.exp_m1() ),
      rel( dm::ln_1p( x ), x.ln_1p() ),
      rel( dm::ln( 1.0 + x ), x.ln_1p() ),
    );
  }

  amplification();
}

/// What a small-argument shortfall costs when a caller subtracts one of these functions from its own
/// argument.
///
/// `( sinh( s ) - s ) / s³` has the limit `1/6` at zero. The subtraction cancels the leading term,
/// so whatever relative error `sinh` carries is divided by a vanishing quantity: an error that looks
/// harmless at the function's own output scale dominates the result here, and the amplification is
/// roughly `1 / s²`. This is the shape of expression — a Stumpff function, a series remainder, a
/// finite-difference derivative — where the difference between 1e-16 and 1e-12 stops being academic.
///
/// It is also why a caller who writes this expression should band it: evaluate the closed form only
/// where `s` is large enough, and a Taylor series below. This table is the evidence for where that
/// band belongs.
///
/// Scored against `1/6 + s²/120 + s⁴/5040`, the accurate evaluation for small `s`, so both
/// implementations are compared to the mathematics rather than to each other.
fn amplification()
{
  println!();
  println!( "`( sinh(s) - s ) / s³` — relative error against the series. The subtraction amplifies" );
  println!( "whatever error `sinh` carries by about `1 / s²`, which is why the small-argument fix" );
  println!( "matters beyond its own output scale:" );
  println!();
  println!( "| s | series | via dm::sinh | rel | via libm sinh | rel |" );
  println!( "|---|--------|--------------|-----|---------------|-----|" );
  for e in 1..=6
  {
    let s = 10.0_f64.powi( -e );
    let series = 1.0 / 6.0 + s * s / 120.0 + s * s * s * s / 5040.0;
    let via_det = ( dm::sinh( s ) - s ) / ( s * s * s );
    let via_libm = ( s.sinh() - s ) / ( s * s * s );
    println!
    (
      "| 1e-{} | {:.12} | {:.12} | {:.0e} | {:.12} | {:.0e} |",
      e, series, via_det, ( ( via_det - series ) / series ).abs(),
      via_libm, ( ( via_libm - series ) / series ).abs()
    );
  }
}
