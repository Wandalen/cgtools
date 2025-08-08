## Renderer
Contains structures to create and render scene. So far in webgl only.

Example:

``` no_run rust
use minwebgl as gl;
use renderer::webgl::
{
    loaders,
    Renderer,
    SwapFramebuffer,
    post_processing::
    {
        ToneMappingPass,
        ToSrgbPass,
        ToneMappingAces
    }
};
// Get window handle
let window = gl::web_sys::window().unwrap();
// Get an html document handle
let document = window.document().unwrap();
// Turn of the antialiasing, because the renderer renders to the
// multisample render buffer
let options = gl::context::ContexOptions::default().antialias( false );
// Create canvas
let canvas = gl::canvas::make()?;
// Create context
let gl = gl::context::from_canvas_with( &canvas, options )?;
// Enable float textures as color renderable
let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

// Create the renderer, specifying the level of multisampling
let renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 );

// Load the gltf file
let gltf = loaders::gltf::load( &document, "path_to_the_gltf", &gl ).await?;
// Get scenes from the gltf
let scene = gltf.scenes;

// Create the camera
let camera = Camera::new( ... );


// To render the postprocessing effect, we need to create a SwapFrameBuffer,
// that will use ping-pong method to render to textures. After each pass,
// textures need to be swapped
let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

// Since the renderer renders to HDR texture, we need to first tonemap it into LDR, and then
// apply gamma correction.
let tonemapping = ToneMappingPass::< ToneMappingAces >::new( &gl, canvas.width(), canvas.height() )?;
let to_srgb = ToSrgbPass::new( &gl, true )?;

// Update the world matrix for the nodes in the scene
scenes[ 0 ].borrow_mut().update_world_matrix();

// Render the scene
renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera ).expect( "Failed to render" );

// Reset the swap_buffer state
swap_buffer.reset();
// Bind the framebuffer behind the swap_buffer
swap_buffer.bind( &gl );
// Swap buffer has an `input` and `output` textures. Passes apply the transformation to the `input` texture, rendering it into the `output` texture. The `output` texture is created by the swap_buffer. The `input` texture needs to be set
swap_buffer.set_input( renderer.get_main_texture() );

// Apply the tonemapping pass
let output = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() ).expect( "Failed to render tonemapping pass" );

// Update the output texture of the swap_buffer
swap_buffer.set_output( output );
// Swap the `input` and `output` textures
swap_buffer.swap();

// ToSrgbPass will render to the screen, because we passed `true` to the `render_to_screen` variable of the pass
let _ = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() ).expect( "Failed to render ToSrgbPass" );
```

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
- [Vulkan-glTF-PBR]
-

#### Normal mapping
- [Normals and the Inverse Transpose, Part 1: Grassmann Algebra]
- [Normals and the Inverse Transpose, Part 2: Dual Spaces]
- [Normal Mapping Without Precomputed Tangents]

#### KHR Extensions
- [KHR_materials_specular]

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

[KHR_materials_specular]:  https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_specular/README.md
[Vulkan-glTF-PBR]: https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/data/shaders/genbrdflut.frag
[Image Based Lighting with Multiple Scattering]: https://bruop.github.io/ibl/
