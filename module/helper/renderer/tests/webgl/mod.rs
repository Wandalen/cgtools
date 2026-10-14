use renderer::webgl as the_module;
use mingl::math;

// Browser, not Node: `web_sys::window()` is `None` under Node, so every test in
// this binary that touches a live WebGL2 context fails there at *runtime* with a
// misleading "Failed to get window" rather than at compile time. The call is
// per-binary, not per-file, and every file below is part of this one binary
// (`tests/tests.rs` -> `mod webgl`) — so it belongs here, at the binary's own
// root, rather than being inherited from whichever leaf module happens to carry
// one. `ibl.rs` and `wide_outline.rs` each carry their own besides; duplicates
// within a binary are legal and those two predate this one.
#[ cfg( all( test, target_arch = "wasm32" ) ) ]
wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

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

// The GL-resource-lifecycle suites below were inline `#[cfg(test)] mod tests`
// blocks under `src/webgl/**` until `rulebook.md § Test placement` was changed
// to put every test in `tests/`. Each covers one type's teardown contract:
// what a `Drop` or `gl_resources_free` must release, and what it must leave
// alone. All are BUG-432..440 reproducers.
//
// Both gates are load-bearing. `target_arch = "wasm32"` because every one of
// these needs a live `WebGl2RenderingContext` to construct its subject at all.
// `feature = "test_internals"` because a teardown assertion has to hold the
// handles across the drop, and those are private state — the feature is what
// makes the `_for_test` accessors that hand them over exist. Run them with
// `--features test_internals`; without it these modules compile away entirely
// and the suites above still run on their own.
/// `IBL` texture teardown and clone/double-free behaviour
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod ibl_gl_lifecycle;

/// `Renderer::gl_resources_free` and resize buffer replacement
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod renderer_gl_lifecycle;

/// `TransformsData` / `DisplacementsData` texture teardown
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod skeleton_gl_lifecycle;

/// `ShadowBaker` framebuffer teardown and cull-face restoration
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod shadow_gl_lifecycle;

/// `GBuffer` VAO / framebuffer / depth buffer / texture teardown
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod gbuffer_gl_lifecycle;

/// `BlendPass::gl_resources_free` program teardown
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod blend_gl_lifecycle;

/// `UnrealBloomPass` drop-without-explicit-free teardown
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod unreal_bloom_gl_lifecycle;

/// `WideOutlinePass::gl_resources_free` owned-vs-borrowed teardown
#[ cfg( all( target_arch = "wasm32", feature = "test_internals" ) ) ]
mod wide_outline_gl_lifecycle;
