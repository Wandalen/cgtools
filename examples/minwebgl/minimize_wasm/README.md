# Compilation config part for minimize wasm size

Should be in main workspace file

```toml
[profile.release]
lto = true
opt-level = 'z'
strip = true  # Automatically strip symbols from the binary.
codegen-units = 1
#panic = "abort"  # Commented out because it increases size
```

# Size of `simple_pbr` optimization results:

Debug size: 304659  
Release before any size optimizations: 84592

Release with optimization: 49170  
Command `wasm-strip <file.wasm>`: 49026  
Command `wasm-opt -Os -o <output.wasm> <input.wasm>`: 47289  
Command `wasm-opt -Oz -o <output.wasm> <input.wasm>`: 47292

File `run.sh` compiles and packs wasm file, splits and fixes javascript code in `index.html` to make it load splited parts, join and use as single wasm file
