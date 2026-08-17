use renderer::webgl as the_module;
use mingl::math;

/// Node related tests
mod node;

/// Mesh clone independence tests
mod mesh;

/// Scene related tests
mod scene;

/// Camera related tests
mod camera;

/// PBR material tests
mod pbr_material;

/// Shadow-baking Light tests
mod shadow;

/// G-buffer attachment metadata (shader `#define` names, vertex-attribute descriptors) tests
mod gbuffer;

/// Color-grading white balance tint-direction tests
mod white_balance;

/// Color-grading vibrance relative-saturation-boost weighting tests
mod vibrance;

/// Wide-outline pass structural / uniform-wiring tests
mod wide_outline;

/// Wide-outline JFA step per-axis pixel-jump isotropy tests
mod jfa_step_size;

/// Wide-outline JFA silhouette-detection threshold tests
mod jfa_silhouette;

/// Wide-outline JFA final-result ping-pong buffer selection tests
mod jfa_buffer_selection;

/// Wide-outline outline-pass JFA seed-sentinel validity tests
mod outline_seed_sentinel;

/// Skeleton morph-target displacement-texture sizing tests
mod displacement_texture_size;

/// Renderer per-material shader-program cache invalidation tests
mod program_needs_recompile;

/// `SwapFramebuffer::new` doc-comment-vs-body renderbuffer claim consistency tests
mod pass;

/// IBL loader texture-parameter / mip-range wiring tests
mod ibl;
