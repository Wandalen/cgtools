# minwebgl

Minimal set of tools for concise WebGL programming.

### Implemented Features

- Attributes Uploading
- Matrices in Attributes Uploading, row-major
- Instanced Rendering
- Uniforms Buffer Objects
- Vertex Array Objects

### Installation and running
To use this library, you'll need to compile you application to wasm and then use it from javascript.  

#### Wasm-pack
You can use wasm-pack:
```bash
rustup target add wasm32-unknown-unknown  #Install wasm toolchain for rust
cargo install wasm-pack #Install wasm-pack
wasm-pack build --target web #Build your app for plain use on the web
```
`--taget web` option will allow you to load your code in a browser directly( in plain HTML ).
You can mark your main function with #[wasm_bindgen(start)]`` And then use it in html:
```html
<script type="module">
    import init from "./pkg/you_crate_name.js";
    init();
</script>
```

#### Trunk
Another way is to use trunk - a WASM web application bundler for Rust.
```bash
rustup target add wasm32-unknown-unknown  #Install wasm toolchain for rust
cargo install trunk #Install truck
trunk serve --release #Build and run your server in release mode
```

Any files you want to load in your project have to lie in a 'static' directory, relatively to the index.html file.  
When using trunk you can specify the following, if your file lie elsewhere
```html
<link data-trunk rel="copy-dir" href="assets/" data-target-path="static"/>
```