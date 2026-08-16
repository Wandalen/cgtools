use renderer::webgl as the_module;
use mingl::math;

/// Node related tests
mod node;

/// Scene related tests
mod scene;

/// Camera related tests
mod camera;

/// PBR material tests
mod pbr_material;

/// Shadow-baking Light tests
mod shadow;

/// Color-grading white balance tint-direction tests
mod white_balance;

/// Wide-outline pass structural / uniform-wiring tests
mod wide_outline;

/// Wide-outline JFA step per-axis pixel-jump isotropy tests
mod jfa_step_size;

/// Wide-outline JFA silhouette-detection threshold tests
mod jfa_silhouette;

/// Wide-outline outline-pass JFA seed-sentinel validity tests
mod outline_seed_sentinel;
