//@ name: gaussian_weight
//@ description: Unnormalized 1D Gaussian weight for an offset at the given sigma.
//@ tags: category:filter
//@ depends_on:
//@ export: fn gaussian_weight(offset: f32, sigma: f32) -> f32
//@ export: fn gaussian_weight_preview(p: vec2f, sigma: f32) -> f32
//@ param: sigma argument f32 range(0.1, 2.0)

fn gaussian_weight( offset : f32, sigma : f32 ) -> f32
{
  // Unnormalized bell curve : callers accumulate the weight sum over their
  // taps and divide once, matching the workspace's GLSL separable blur.
  let x = offset / sigma;
  return exp( -0.5 * x * x );
}

fn gaussian_weight_preview( p : vec2f, sigma : f32 ) -> f32
{
  return gaussian_weight( length( p ), sigma );
}
