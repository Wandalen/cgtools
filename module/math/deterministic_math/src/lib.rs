#![ doc( html_root_url = "https://docs.rs/deterministic_math/latest/deterministic_math/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Bit-reproducible transcendental functions built only from the operations IEEE-754 pins down" ) ]

//! # Why a crate ships its own `sin`
//!
//! IEEE-754 fixes the result of `+`, `-`, `*`, `/`, `sqrt` and `fma` exactly:
//! given the same inputs and rounding mode, every conforming machine produces
//! the same bits. It says nothing of the sort about `sin`, `cos`, `atan`, `exp`
//! or `ln`. For those, correct rounding is *recommended* and not required —
//! proving the last bit right in every case runs into the table-maker's
//! dilemma, since the exact result can sit arbitrarily close to a rounding
//! boundary and no bounded working precision decides every input. Conforming
//! implementations are therefore free to differ in the final place, and they
//! do: between glibc and musl, between two glibc releases, and between an
//! x86-64 build and an aarch64 one compiled from identical source.
//!
//! For rendering that is invisible. For a simulation whose clients must agree —
//! lockstep multiplayer, deterministic replay, a recorded run that has to
//! reproduce after a libc upgrade — it is fatal, and it fails in a
//! characteristic way. The drift is not the problem; the first *branch* is. Two
//! machines evaluating the same threshold against the same state return `true`
//! and `false`, and from that instant they are simulating different worlds
//! rather than the same world with a rounding error in it. An iterative solver
//! reaches that point quickly, because each iteration multiplies the input
//! difference it was given.
//!
//! Everything here is therefore built from the exact operations alone. The
//! coefficients and reduction constants travel with this crate, so they are the
//! same on every target by construction rather than by the coincidence of two
//! vendors having chosen alike.
//!
//! # Reproducible is not the same as accurate
//!
//! This crate guarantees exactly one property: **the same input produces the
//! same bits on every target**. It does not claim to be correctly rounded, and
//! for several functions it is measurably further from the true value than a
//! good libm is. Those are separate axes, and only the first is on offer here.
//! A function reproducibly one ulp off on every machine satisfies the whole
//! contract; a function nearer the truth on one machine and nearer it
//! differently on another satisfies none of it.
//!
//! `examples/cost_vs_libm.rs` measures both axes for every function and prints
//! the table. Run it before assuming either number.
//!
//! # Using it
//!
//! Call these by path, so the choice is legible at the call site:
//!
//! ```rust
//! let ( s, c ) = deterministic_math::sin_cos( 0.5 );
//! assert!( ( s * s + c * c - 1.0 ).abs() < 1.0e-15 );
//! ```
//!
//! Writing `x.sin()` instead silently reintroduces the platform's libm, which
//! is the entire thing this crate exists to avoid. The distinction is not
//! visible in a diff unless you are looking for it, so a project relying on
//! this guarantee is best served by a lint that greps for the bare method form.
//!
//! Vector and matrix algebra needs no such treatment: it is built from `+`, `-`
//! and `*`, which IEEE-754 already pins, and Rust never contracts `a * b + c`
//! into an FMA on its own.
//!
//! # What is deliberately absent
//!
//! `abs`, `floor`, `ceil`, `round`, `trunc`, `fract`, `signum`, `recip`,
//! `clamp`, `min`, `max`, `to_degrees`, `to_radians`, `rem_euclid` and their
//! neighbours are **not** here and do not need to be. Each is either exact by
//! construction or a single pinned operation, so `f64`'s own method is already
//! bit-identical on every target and wrapping it would add a call and no
//! guarantee. Their absence is not a gap.
//!
//! What *is* here is every function whose `f64` method is free to differ
//! between implementations — plus [`sqrt`] and [`mul_add`], which are pinned
//! but are included anyway so that one crate holds every arithmetic entry point
//! a reproducibility audit has to look at.
//!
//! The one public item that is not arithmetic,
//! [`measure::ulp_diff`], stays behind its module path for that same reason:
//! the root is the list of pinned functions, and it is only useful as a list if
//! everything on it belongs to the same kind.

mod constant;

/// Measuring the distance between two results, for grading this surface rather
/// than extending it.
///
/// Deliberately not re-exported flat: [`ulp_diff`](measure::ulp_diff) is not a
/// pinned function, and the crate root is the list of pinned functions.
pub mod measure;

mod algebraic;
mod circular;
mod exponential;
mod hyperbolic;
mod inverse_circular;

/// The reduction steps, series and coefficient tables the surface is built
/// from, exposed only under the `test_internals` feature.
///
/// Not part of the surface. With the feature off — which is the default, and
/// what any dependent gets — this module does not exist, and the crate exports
/// exactly the 28 functions, 7 constants and [`measure::ulp_diff`] that
/// `docs/api/001_function_surface.md` documents.
///
/// It exists because tests live in `tests/`, which is a separate crate and so
/// cannot reach a private item the way an inline `mod tests` could. The
/// alternative — asserting only on composed results — is materially weaker:
/// a wrong coefficient table and a wrong reduction can cancel, and
/// `the_bin_table_holds_the_arctangent_of_each_centre` exists precisely to
/// catch the case where they do.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
pub mod internal
{
  pub use crate::algebraic::{ atan_series, atanh_series, round_half_away, scale2 };
  pub use crate::circular::{ cos_reduced, sin_reduced };
  pub use crate::exponential::{ exp_reduced, ln_mantissa, mantissa_exponent };
  pub use crate::inverse_circular::clamp_unit;
  pub use crate::constant::
  {
    ASIN_DIRECT, ATAN_B, ATAN_DIRECT, ATAN_V, HYPERBOLIC_SATURATION,
    LOG10_2, LOG10_E, LOG2_E, PIO2_HI, PIO2_LO, PIO2_MD, SERIES_BAND,
  };
}

pub use constant::{ FRAC_PI_2, FRAC_PI_4, LN_10, LN_2, PI, SIN_COS_MAX, TAU };

pub use algebraic::{ hypot, mul_add, powi, sqrt };
pub use circular::{ cos, sin, sin_cos, tan };
pub use exponential::{ cbrt, exp, exp2, exp_m1, ln, ln_1p, log, log2, log10, powf };
pub use hyperbolic::{ acosh, asinh, atanh, cosh, sinh, tanh };
pub use inverse_circular::{ acos, asin, atan, atan2 };
