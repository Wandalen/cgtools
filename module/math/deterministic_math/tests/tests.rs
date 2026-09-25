//! Integration tests for `deterministic_math`, one module per function family.

// Exact float equality is the subject of this suite, not a mistake in it. The
// crate's whole claim is that a given input yields a given bit pattern, so an
// assertion that two `f64`s are *equal* is the strongest form of the claim and
// an epsilon would weaken it into something that passes when the claim fails.
//
// This allow is scoped to the test crate deliberately, and it covers the
// assertions only — not the ruler they are measured with. `ulp_diff` is library
// code (`deterministic_math::measure`), re-exported by `inc` rather than
// reimplemented here, and it carries no suppression of its own: it compares bit
// patterns, never values. The library as a whole carries none either — the one
// place it compared against a non-zero constant, `atanh`'s domain boundary, was
// restructured to use `>=` instead, so a genuine approximate-comparison bug in
// library code would still be caught.
#![ allow( clippy::float_cmp, reason = "bit-exact equality is what this suite asserts" ) ]

use deterministic_math as the_module;

mod inc;
