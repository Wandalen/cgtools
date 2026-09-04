//! ## Root Cause
//! `shader.frag`'s `getRefractionColor` computes a per-bounce travel distance `r` that
//! feeds a Beer-Lambert absorption term (`attenuationFactor *= exp( -r * ( 1.0 -
//! colorAbsorption ) )`), meant to be scaled by the `absorptionFactor` uniform. The
//! multiplication was commented out, so `absorptionFactor` — declared as a uniform and
//! uploaded a value from `main.rs` — had no effect whatsoever on the rendered image.
//!
//! ## Why Not Caught
//! The shader still compiled and linked fine (an unused uniform is not a compile error),
//! and the visual difference between "absorption scaled by 0.8" and "absorption scaled by
//! an implicit 1.0" is subtle for many scenes — nothing crashed or produced an obviously
//! wrong result. The crate has no lib target or native test target to unit-test shader
//! behavior directly, only this structural source parse.
//!
//! ## Fix Applied (BUG-322)
//! Restored the multiplication in `examples/minwebgl/diamond/shaders/shader.frag`'s `r`
//! computation: `length( rayOrigin - oldOrigin ) * absorptionFactor`.
//!
//! ## Prevention
//! Asserts the active (non-comment) code computing `r` includes `absorptionFactor`, and
//! more generally that the uniform is used somewhere outside its own declaration line and
//! outside a comment — catching either a regression to the exact old commented-out line,
//! or any other future dead-uniform mistake.
//!
//! ## Pitfall
//! Don't assert on the shader's exact formatting/whitespace — only that the multiplication
//! is present in the active code, so harmless future reformatting doesn't break this test.

const SHADER_FRAG : &str = include_str!( "../shaders/shader.frag" );

/// Strips a `//` line comment (if any) from a single line of GLSL source.
fn active_code( line : &str ) -> &str
{
  line.split( "//" ).next().unwrap_or( "" )
}

#[ test ]
fn r_computation_multiplies_by_absorption_factor()
{
  let line = SHADER_FRAG.lines()
  .find( | l | l.contains( "length( rayOrigin - oldOrigin )" ) )
  .expect( "r computation line not found in shader.frag" );

  assert!(
    active_code( line ).contains( "absorptionFactor" ),
    "the active (non-comment) code computing `r` must multiply by absorptionFactor: {line:?}"
  );
}

#[ test ]
fn absorption_factor_uniform_is_not_dead()
{
  let active_uses = SHADER_FRAG.lines()
  .filter( | l | !l.trim_start().starts_with( "uniform" ) )
  .filter( | l | active_code( l ).contains( "absorptionFactor" ) )
  .count();

  assert!(
    active_uses >= 1,
    "absorptionFactor uniform must be used in at least one active, non-declaration line of shader.frag"
  );
}
