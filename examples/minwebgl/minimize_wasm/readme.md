# Minimized WASM Build

**Keywords:** WASM, Optimization, Binary Size, WebGL2

This demo showcases techniques for minimizing WebAssembly binary size in cgtools applications.
<!-- Fix(BUG-329): named an unconfigured link-time-optimization technique this crate never actually
     enables (no [profile.release] exists anywhere reachable from this crate or the workspace).
     Root cause: aspirational wording never checked against the actual build pipeline.
     Pitfall: a demo whose purpose IS showcasing techniques is exactly where a wrong named
     technique goes unnoticed, since the demo still visibly "works" either way. -->
It demonstrates optimization strategies including a minimal global allocator (`wee_alloc`),
post-build size optimization (`wasm-opt -Os`), and debug-info stripping (`wasm-strip`).

An alternative minimal allocator is `lol_alloc`'s `LeakingPageAllocator` — smaller and
simpler than `wee_alloc` since it never reclaims memory (bump-allocates from pages and leaks
on "free"), which is a reasonable trade for short-lived, one-shot WASM modules but unsuitable
for long-running applications that repeatedly allocate and deallocate.

Small WASM binaries improve load times and user experience. This example serves as a reference for production builds requiring minimal download size.

![image](./showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [Shrinking .wasm Code Size]

[Shrinking .wasm Code Size]: https://rustwasm.github.io/book/reference/code-size.html
