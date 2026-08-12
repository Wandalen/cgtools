# action

Curated action scripts for the cgtools workspace.

| File | Responsibility |
|------|----------------|
| `run` | Build and run any example or binary by partial match against its path |
| `browser_serve` | Shared trunk-serve + browsee-launch logic for browser examples' `verb/run` |
| `gallery` | Regenerate examples/index.html + index.md from `run`'s discovery and each example's readme.md |
| `build_site` | Assemble the static GitHub Pages output (`_site/`): gallery, showcase images, per-example trunk builds |
