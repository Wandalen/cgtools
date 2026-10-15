# gl_uniforms

Program-scoped WebGL uniform upload wrapper for `minwebgl`-based renderers.

Collapses the boilerplate repeated at every uniform call site -- `gl.get_uniform_location(
program, name )`, then `gl::uniform::(matrix_)upload( ... )`, then `.expect( "uniform
upload should not fail" )` -- into a single `.upload( name, &value )` /
`.matrix_upload( name, &value, column_major )` call. `ProgramUniforms` binds a `GL`
context and a linked `WebGlProgram` once, so each call site only needs a name and a value.

## Responsibility Table

| File | Responsibility |
|------|-----------------|
| `src/lib.rs` | `ProgramUniforms` wrapper -- `.upload()`/`.matrix_upload()` over `minwebgl::uniform`'s `UniformUpload`/`UniformMatrixUpload` traits |
| `tests/readme.md` | Live-context tests for `ProgramUniforms` upload/matrix_upload |
