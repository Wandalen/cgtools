//! Every numeric constant the kernel evaluates against, in one place.
//!
//! Constants come from `core`, functions do not — and the asymmetry is the
//! point rather than an inconsistency. A `const` is a number the compiler bakes
//! in at compile time: `core::f64::consts::PI` is the same 64 bits on every
//! target, arrived at without executing anything, so re-exporting it costs this
//! crate nothing it was protecting. A *function* is a call into whatever libm
//! the target links, which is exactly what the rest of this crate exists to
//! avoid. Writing the digits out by hand instead would produce identical bits
//! and merely hide the provenance.
//!
//! The reduction tables below have no such source and are written out. Each is
//! the shortest decimal that round-trips to the `f64` its generating
//! computation produced, so substituting the literal is exact rather than
//! close. Every one of them is rebuilt from its own definition at run time and
//! checked — `RECIP_FACT` and the two circular series in `circular.rs`'s own
//! `mod tests`, `ATAN_B`/`ATAN_V` and the series that consume them in
//! `inverse_circular.rs`'s, the `PI/2` split in `circular.rs`'s. A mistyped
//! digit anywhere here fails a test rather than shifting an answer.

/// Ratio of a circle's circumference to its diameter.
pub const PI : f64 = core::f64::consts::PI;

/// A full turn — `2 * PI`, the period every angle wraps to.
pub const TAU : f64 = core::f64::consts::TAU;

/// A quarter turn, the width of the reduced range `sin` and `cos` evaluate on.
pub const FRAC_PI_2 : f64 = core::f64::consts::FRAC_PI_2;

/// An eighth turn, the boundary `atan`'s first reduction compares against.
pub const FRAC_PI_4 : f64 = core::f64::consts::FRAC_PI_4;

/// Natural logarithm of 2, as one double.
pub const LN_2 : f64 = core::f64::consts::LN_2;

/// Natural logarithm of 10, as one double.
pub const LN_10 : f64 = core::f64::consts::LN_10;

/// Largest argument [`crate::sin_cos`] will reduce.
///
/// Past this the three-part `PI/2` split stops carrying enough bits for the
/// residual to mean anything, and `x * INV_PIO2` approaches the `i32` the
/// quarter-turn count is held in. Any angle wrapped to `[ -PI, PI )` or bounded
/// by a solver bracket stays far inside; the bound is here for arguments
/// arriving from outside such a discipline.
pub const SIN_COS_MAX : f64 = 1.0e8;

/// Natural logarithm of 2, high half — the top 26 mantissa bits, so that
/// `k * LN2_HI` is exact for any `k` small enough to be an exponent.
pub( crate ) const LN2_HI : f64 = 0.693_147_167_563_438_4;

/// Natural logarithm of 2, low half. `LN2_HI + LN2_LO` rounds to `ln 2`, and
/// the pair together carry about 80 bits — which is what makes the Cody-Waite
/// reduction `x - k * LN2_HI - k * LN2_LO` lose nothing when `k` is large.
pub( crate ) const LN2_LO : f64 = 1.299_650_689_388_988_9e-08;

/// `PI / 2`, split into three parts for the same reason `LN2` is split into
/// two. Three rather than two because the argument may be a few thousand
/// radians, and a two-part reduction starts losing bits there.
pub( crate ) const PIO2_HI : f64 = 1.570_796_310_901_641_8;
/// `PI / 2`, middle part — see [`PIO2_HI`].
pub( crate ) const PIO2_MD : f64 = 1.589_325_477_352_819_6e-08;
/// `PI / 2`, low part — see [`PIO2_HI`].
pub( crate ) const PIO2_LO : f64 = 6.368_317_163_510_95e-25;

/// `2 / PI`, used to find how many quarter turns to strip.
pub( crate ) const INV_PIO2 : f64 = core::f64::consts::FRAC_2_PI;

/// `log2( e )`, the factor turning a natural exponent into a binary one.
pub( crate ) const LOG2_E : f64 = core::f64::consts::LOG2_E;

/// `log10( 2 )`, for [`crate::log10`]'s exact exponent term.
pub( crate ) const LOG10_2 : f64 = core::f64::consts::LOG10_2;

/// `log10( e )`, turning the mantissa's natural logarithm into a decimal one.
pub( crate ) const LOG10_E : f64 = core::f64::consts::LOG10_E;

/// Where [`crate::asinh`] and [`crate::acosh`] switch to `ln( 2x )`.
///
/// `2^28`. Above it `x * x + 1` rounds to `x * x` — the gap between
/// neighbouring doubles at `7.2e16` is already `16` — so the switch is an
/// identity rather than an approximation, and it is taken long before `x * x`
/// could overflow at `1.34e154`.
pub( crate ) const LARGE_ARGUMENT : f64 = 268_435_456.0;

/// Where [`crate::exp_m1`] and [`crate::ln_1p`] stop using a direct series and
/// fall back to the naive identity each of them exists to replace.
///
/// Both identities fail the same way and the threshold follows from it. Each
/// forms a quantity near `1` and then removes the `1` — `exp( x ) - 1.0`
/// subtracts it, `ln( 1.0 + x )` adds it before taking the logarithm — so each
/// carries an absolute error near half an ulp of `1`, about `1.1e-16`, into a
/// result whose true magnitude is roughly `| x |`. The relative error is
/// therefore about `1.1e-16 / | x |`, which passes one part in `1e15` around
/// `| x | = 0.11` and only improves beyond it. At `0.25` the naive form is
/// already within about 1.5 ulp, while both series still have several orders of
/// margin, so the two paths overlap comfortably rather than meeting at a seam.
///
/// The previous threshold was `1e-5`, four decades too low: across the whole
/// band from there to here the kernel returned bit-identical results to the
/// naive expression it was called to avoid, losing three to four decimal digits
/// exactly where callers had been told it would not.
pub( crate ) const SERIES_BAND : f64 = 0.25;

/// Below this magnitude [`crate::atan`] evaluates its series directly instead
/// of reducing onto a bin centre.
///
/// Half the first bin's width. The bin reduction computes `ATAN_V[ 0 ] +
/// atan_series( u )`, and for an argument near zero those two terms are equal
/// and opposite — the reduction that makes the series converge fast for a
/// mid-range argument is precisely what destroys a small one. Under this
/// threshold the series is already inside its own documented range, so
/// bypassing the reduction costs nothing and keeps every digit.
pub( crate ) const ATAN_DIRECT : f64 = 0.0625;

/// Below this magnitude [`crate::asin`] returns [`crate::atan`] of its argument
/// rather than building the companion side first.
///
/// `2^-27`. `asin` is `atan( c / sqrt( 1 - c² ) )`, and the two functions differ
/// by `c³ / 2` — which under this threshold is below half the last place of `c`,
/// so they are the same double and the companion contributes nothing but
/// arithmetic.
///
/// Arithmetic that costs a last place, at that. `1 - c` and `1 + c` each round,
/// and their product lands a step either side of `1` even though the exact
/// product rounds *to* `1` — so the companion is `1 ± 2^-52` rather than `1`, and
/// that step is the entire last place of the answer. Measured: `asin( 3.7e-12 )`
/// came back one ULP above `3.7e-12`, where the correct result is the argument
/// itself.
///
/// [`crate::acos`] needs no such bypass and has none. It is built from the same
/// two pieces, but its result near zero is `PI/2` rather than something small, so
/// a last-place wobble in the companion is a last-place wobble in `1.57` — 8
/// orders of magnitude less significant. That asymmetry is visible in the
/// measurements: before the reduction work, `asin` disagreed with the platform's
/// libm by 22 268 ULP and `acos`, built the same way, by 69.
pub( crate ) const ASIN_DIRECT : f64 = 7.450_580_596_923_828e-9;

/// Where the hyperbolic functions stop being a difference of two exponentials
/// and become one.
///
/// At `| x | = 20`, `e^-x` is `2e-9` times `e^x`, so `e^-2x` — the relative
/// weight of the second term in `sinh`, `cosh` and `tanh` alike — is `4e-18`,
/// below half an ulp of the first. Past this point the smaller exponential
/// cannot change a single bit of the answer, so evaluating it is pure cost, and
/// on `tanh` it is worse than cost: the quotient is `1.0` exactly, and forming
/// it from two large numbers risks an overflow that returns `NaN` for an
/// argument whose answer is `1`.
///
/// Deliberately far below where `exp` overflows, so the branch is chosen for
/// being an identity rather than for dodging an edge case.
pub( crate ) const HYPERBOLIC_SATURATION : f64 = 20.0;

/// Bin centres [`crate::atan`] reduces to, and the exact `atan` of each.
///
/// Reducing `t` to the nearest of these four leaves a residual no larger than
/// about `0.125`, where the arctangent series converges fast enough to reach
/// full precision in eleven terms. Without the reduction the same series needs
/// hundreds of terms near `t = 1` and never gets there.
pub( crate ) const ATAN_B : [ f64; 4 ] = [ 0.125, 0.375, 0.625, 0.875 ];

/// `atan` of each entry of [`ATAN_B`], to full double precision.
pub( crate ) const ATAN_V : [ f64; 4 ] =
[
  0.124_354_994_546_761_44,
  0.358_770_670_270_572_25,
  0.558_599_315_343_562_4,
  0.718_829_999_621_624_5,
];

/// `1 / n!` for `n` in `0 ..= 19` — every coefficient the Taylor series in this
/// crate need, in one table because they need overlapping halves of it: the
/// exponential series takes all of it, the sine series the odd entries and the
/// cosine series the even ones.
///
/// Written out rather than computed. Each series once rebuilt its own
/// coefficients inside its evaluation loop — a factorial loop and a division
/// per term, on every call — which cost roughly ten times the polynomial that
/// used them. Both intermediate forms were worse than they look: a `const fn`
/// builder is invisible to a coverage gate, because nothing executes it at run
/// time, and separate per-series tables would write `1 / 7!` twice and invite
/// the two copies to drift.
///
/// Three series and not five: the arctangent and logarithm series build their
/// own coefficients inline too, and measurement says leave them alone. Theirs
/// is a single division on a loop-constant index, which the optimiser folds to
/// a literal — tabulating them was tried over four million arguments and moved
/// nothing (`10.21ns` to `10.19ns`). What defeated the optimiser here was a
/// *loop* inside the loop, not arithmetic inside it, and only these three had
/// one.
pub( crate ) const RECIP_FACT : [ f64; 20 ] =
[
  1.0,                          // 1 / 0!
  1.0,                          // 1 / 1!
  0.5,                          // 1 / 2!
  0.166_666_666_666_666_66,     // 1 / 3!
  0.041_666_666_666_666_664,    // 1 / 4!
  0.008_333_333_333_333_333,    // 1 / 5!
  0.001_388_888_888_888_889,    // 1 / 6!
  0.000_198_412_698_412_698_4,  // 1 / 7!
  2.480_158_730_158_73e-5,      // 1 / 8!
  2.755_731_922_398_589_3e-6,   // 1 / 9!
  2.755_731_922_398_589e-7,     // 1 / 10!
  2.505_210_838_544_172e-8,     // 1 / 11!
  2.087_675_698_786_81e-9,      // 1 / 12!
  1.605_904_383_682_161_3e-10,  // 1 / 13!
  1.147_074_559_772_972_5e-11,  // 1 / 14!
  7.647_163_731_819_816e-13,    // 1 / 15!
  4.779_477_332_387_385e-14,    // 1 / 16!
  2.811_457_254_345_520_6e-15,  // 1 / 17!
  1.561_920_696_858_622_5e-16,  // 1 / 18!
  8.220_635_246_624_33e-18,     // 1 / 19!
];
