### GLTF viewer

Some description

### Features
#### GLTf parsing
✅ Buffers  
✅ Images   
✅ Camera      
✅ Scene  
✅ Material  
✅ Mesh  
❌ Tangents
❌ Sparse accessors  
❌ Animations  
❌ Skins and bones  
❌ Morph targets
#### Basic shader
🔄 PBR  
&emsp;✅ Base color texture  
&emsp;✅ Metallic texture  
&emsp;✅ Roughness texture  
&emsp;🔄 IBL  
✅ Normal texture   
✅ Occlusion texture  
❌ Emission texture  
❌ Skinning  
❌ HDR and tonemapping
#### Rendering
❌ Multisampling  
✅ Support for multiple scenes  
❌ Support for multiple cameras  
#### Extensions
❌ None are supported
#### UI
❌ Switching between scenes  
❌ Switching between cameras   
❌ Changing the tonemapping method and value  
❌ GPU Stats  


### Links

#### PBR
- [Real Shading in Unreal Engine 4]
- [Background: Physics and Math of Shading]
- [Moving Frostbite to Physically Based Rendering 2.0]
- [Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs]
- [Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 1]
- [Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 2]
- [Microfacet Models for Refraction through Rough Surfaces]
- [PBR Diffuse Lighting for GGX+Smith Microsurfaces]
- [Sampling Microfacet BRDF]
- [Notes on importance sampling]
- [Article - Physically Based Rendering - Cook–Torrance]

#### Normal mapping
- [Normals and the Inverse Transpose, Part 1: Grassmann Algebra]
- [Normals and the Inverse Transpose, Part 2: Dual Spaces]
- [Normal Mapping Without Precomputed Tangents]

[Real Shading in Unreal Engine 4]: https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
[Background: Physics and Math of Shading]: https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf
[Moving Frostbite to Physically Based Rendering 2.0]: https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf
[Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs]: https://inria.hal.science/hal-00942452v1/document
[Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 1]: https://schuttejoe.github.io/post/ggximportancesamplingpart1/
[Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 2]: https://schuttejoe.github.io/post/ggximportancesamplingpart2/
[Microfacet Models for Refraction through Rough Surfaces]: https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf
[PBR Diffuse Lighting for GGX+Smith Microsurfaces]: https://ubm-twvideo01.s3.amazonaws.com/o1/vault/gdc2017/Presentations/Hammon_Earl_PBR_Diffuse_Lighting.pdf
[Sampling Microfacet BRDF]: https://agraphicsguynotes.com/posts/sample_microfacet_brdf/
[Notes on importance sampling]: https://www.tobias-franke.eu/log/2014/03/30/notes_on_importance_sampling.html
[How Is The NDF Really Defined?]: https://www.reedbeta.com/blog/hows-the-ndf-really-defined/
[Article - Physically Based Rendering - Cook–Torrance]: http://www.codinglabs.net/article_physically_based_rendering_cook_torrance.aspx

[Normals and the Inverse Transpose, Part 1: Grassmann Algebra]: https://www.reedbeta.com/blog/normals-inverse-transpose-part-1/
[Normals and the Inverse Transpose, Part 2: Dual Spaces]: https://www.reedbeta.com/blog/normals-inverse-transpose-part-2/
[Normal Mapping Without Precomputed Tangents]: http://www.thetenthplanet.de/archives/1180