# Morph targets

This example demonstrates how to use a renderer crate for playing animations, update models morph targets. Example gives user posibility to choose animation and change its morph targets weights values using UI.

Morph targets (also called blend shapes) are a technique used in 3D graphics and animation to smoothly transform a mesh from one shape into another.

![Showcase]( ./showcase.png )

## How it is useful

This example showcases several useful techniques and concepts for WebGL2 development:

  * How update morph targets weights using available animations.

  * How link UI with morph targets weights.

## How it works

For discovering how skeletal animation works under the hood check this example description: [skeletal_animation](../skeletal_animation/readme.md).

Working with morph targets includes this steps:

### Loading morph targets displacements

Each mesh has own default set of morph targets weights values as array of floats.

Also each mesh primitive has morph targets list. Each morph target can have position, normal and tangent buffers with displacements or new values. It in standard glTF2, but other libraries can support also another types of displacements.

Current morph target implementation packs all data into one texture for each mesh, that is used in shader as uniform. You can find layout in description of function `DisplacementsData::upload` [here](../../../module/helper/renderer/src/webgl/skeleton.rs). This approach used because of free attributes slots shortage.

### Loading animated weights

Each animation can have one morph target sequence of keyframes. Each frame is set of weights for each morph target at the certain moment of time. They loaded as joints transform keyframes.

### Morph targets in renderer under the hood

Packed displacements texture used in renderer vertex shader. In shader displacement texture read offset calculated relatively to vertex offset, targets amount and displacements types count ( position/normal/tangent ). Displacements used in such way, for example for position: given set of displacements for each weight and set of weights current values, then they multiplied by each other then aggregated as sum. This sum added to base vertex position.

Displacement usage happens before skin matrix usage.

### Update animated weights

When all data prepared then rendering loop is started. Then firstly all animation channels updating relatively to current time. At this moment all morph target weights interpolated relatively to keyframes. Next step is write updated morph targets weights to skeleton where displacement data is stored. Then renderer calls [`Skeleton::upload`] method that calls [`DisplacementsData::upload`] that loads related to morph targets uniform variables and displacement texture to shader. Shader uses displacements while rendering next frame. And we can see result on screen.

## Running
Ensure you have all the necessary dependencies installed. This example uses trunk for building and serving the WebAssembly application.

To run the example:

Navigate to the example's directory in your terminal.

Run the command:

```bash
  trunk serve
```

Open your web browser to the address provided by trunk (usually http://127.0.0.1:8080).

The application will load the GLTF model, skeletons, animations and start the rendering loop, displaying animated 3D objects. You can select different animations that contained in GLTF file using the provided UI controls. Also you can change any morph target weight value in UI.

Feel free to replace `zophrac.glb` with your own 3D model and animations by modifying path to file in the main.rs file and loading own assets into [folder](../../../assets/gltf/animated/morph_targets).

## 📚 References

### Similar example on another engines or libraries
- [ThreeJS]

### Assets
- [Sketchfab]

### How morph targets works
- [Morph Targets]

[ThreeJS]: https://threejs.org/examples/?q=morph#webgl_morphtargets_face
[Sketchfab]: https://sketchfab.com/3d-models/zophrac-9fea6ffd67b840cb970f5b4570794709
[Morph Targets]: https://github.khronos.org/glTF-Tutorials/gltfTutorial/gltfTutorial_018_MorphTargets.html

