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

/// Every numeric constant the kernel evaluates against.
pub mod constant;

mod algebraic;
mod circular;
mod exponential;
mod hyperbolic;
mod inverse_circular;

pub use constant::{ FRAC_PI_2, FRAC_PI_4, LN_10, LN_2, PI, SIN_COS_MAX, TAU };

pub use algebraic::{ hypot, mul_add, powi, sqrt };
pub use circular::{ cos, sin, sin_cos, tan };
pub use exponential::{ cbrt, exp, exp2, exp_m1, ln, ln_1p, log, log2, log10, powf };
pub use hyperbolic::{ acosh, asinh, atanh, cosh, sinh, tanh };
pub use inverse_circular::{ acos, asin, atan, atan2 };
