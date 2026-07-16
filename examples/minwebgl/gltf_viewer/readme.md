# glTF Viewer

**Keywords:** glTF, PBR, 3D Models, WebGL2

This demo is a comprehensive glTF 2.0 viewer implementing physically-based rendering (PBR) in WebGL2. It loads and displays 3D models with industry-standard materials, including metallic-roughness workflow, normal mapping, and image-based lighting.

glTF is the standard format for 3D content on the web. This example provides a production-ready viewer with full PBR material support, suitable for e-commerce, education, and 3D asset preview.

![image](./showcase.webp)

**[How to run](../how_to_run.md)**

## KTX2 compressed textures

Build with the `ktx2` feature to load a KTX2 / Basis Universal model (`KHR_texture_basisu`) instead of the default. The textures are decoded in pure Rust and transcoded at load time to whichever compressed format your GPU supports (ASTC 4×4, BC7 or ETC2), falling back to uncompressed RGBA8:

```sh
trunk serve --release --features ktx2
```

Only **UASTC** payloads are supported — ETC1S / BasisLZ is rejected with an actionable error. To produce a UASTC-encoded `.glb` from an ordinary one, use [gltf-transform](https://gltf-transform.dev), which shells out to KTX-Software's `ktx` CLI (it must be on your `PATH`):

```sh
gltf-transform uastc in.glb out.glb
```

**References:**

* [Real Shading in Unreal Engine 4] - Industry standard PBR implementation
* [Background: Physics and Math of Shading] - Mathematical foundations
* [Moving Frostbite to PBR 2.0] - Production pipeline insights
* [Understanding Masking-Shadowing] - Geometry function theory
* [GGX Importance Sampling Part 1] - Monte Carlo integration
* [GGX Importance Sampling Part 2] - Advanced sampling techniques
* [Sampling Microfacet BRDF] - Practical BRDF implementation
* [Normal Mapping Without Tangents] - Alternative normal mapping approach
* [Vulkan glTF PBR Reference] - Complete reference implementation

[Real Shading in Unreal Engine 4]: https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
[Background: Physics and Math of Shading]: https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf
[Moving Frostbite to PBR 2.0]: https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf
[Understanding Masking-Shadowing]: https://inria.hal.science/hal-00942452v1/document
[GGX Importance Sampling Part 1]: https://schuttejoe.github.io/post/ggximportancesamplingpart1/
[GGX Importance Sampling Part 2]: https://schuttejoe.github.io/post/ggximportancesamplingpart2/
[Sampling Microfacet BRDF]: https://agraphicsguynotes.com/posts/sample_microfacet_brdf/
[Normal Mapping Without Tangents]: http://www.thetenthplanet.de/archives/1180
[Vulkan glTF PBR Reference]: https://github.com/SaschaWillems/Vulkan-glTF-PBR
