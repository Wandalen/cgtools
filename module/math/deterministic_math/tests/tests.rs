//! Integration tests for `deterministic_math`, one module per function family.

// Exact float equality is the subject of this suite, not a mistake in it. The
// crate's whole claim is that a given input yields a given bit pattern, so an
// assertion that two `f64`s are *equal* is the strongest form of the claim and
// an epsilon would weaken it into something that passes when the claim fails.
// `ulp_diff` uses the same comparison as its zero-distance fast path.
//
// This allow is scoped to the test crate deliberately. The library itself
// carries no `float_cmp` suppression — the one place it compared against a
// non-zero constant, `atanh`'s domain boundary, was restructured to use `>=`
// instead, so a genuine approximate-comparison bug in library code would still
// be caught.
#![ allow( clippy::float_cmp, reason = "bit-exact equality is what this suite asserts" ) ]

use deterministic_math as the_module;

mod inc;
