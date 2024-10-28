### General viewer for models in obj format
To load a model, put it in the "assets" folder, and then in the main function supply the following information:
```Rust
let mtl_path = "lost-empire";
let texture_path = "lost-empire";
let obl_path = "lost-empire/lost_empire.obj";
```
Then you can run `trunk serve --realese` in the directory to start the server. Be sure to use `--release` flag, as it may take up to a minute to load and parse some the larger models in debug mode.

The viewere is far from perfect. It is meant to generalize all models and display them using a simple PBR if possible. Not all models can be loaded and not all material properties are supported.  
If you experience problems with viewing your model, you can try using `diagnosic` module to determine the problem.

### Useful links
- [PBR in opengl]
- [Normal mapping]
- [Intro to PBR]
- [Microfacet BRDF]
- [An Introduction to Physically Based Rendering]
- [Physically Based Rendering for Artists]

[PBR in opengl]:  https://learnopengl.com/PBR/Theory
[Normal mapping]:  https://learnopengl.com/Advanced-Lighting/Normal-Mapping
[Intro to PBR]: https://www.youtube.com/watch?v=gya7x9H3mV0&list=PLeb33PCuqDdesjTOgWXXAF4-gjknPPhBm&index=7
[Microfacet BRDF]: https://simonstechblog.blogspot.com/2011/12/microfacet-brdf.html
[Physically Based Rendering for Artists]: https://www.youtube.com/watch?v=LNwMJeWFr0U
[An Introduction to Physically Based Rendering]: https://typhomnt.github.io/teaching/ray_tracing/pbr_intro/

![Object viewer example](./showcase.jpg)