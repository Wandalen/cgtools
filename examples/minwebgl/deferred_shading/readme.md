# Deferred Shading with Light Volumes

This demo illustrates how to set up a deferred shading pipeline and utilize light volumes for optimized lighting calculations.

![showcase](showcase.png)

## Why Use Deferred Shading and Light Volumes?

Lighting calculations can be computationally expensive, significantly impacting performance, especially with a large number of light sources. [Deferred shading][1] addresses this by separating the geometry pass from the lighting pass.

Light volumes further optimize this process by limiting lighting calculations only to the regions of the G-buffer that are affected by a specific light source. A light volume acts as a bounding shape for a light source, defining the area it illuminates. The size of the light volume should correspond to the light's effective radius. This approach requires an advanced [light attenuation function][2] that accurately models the light's falloff within the volume.

## How to Run

Ensure you have installed all the necessary [dependencies](../../../module/min/minwebgl/readme.md). Navigate to the example's directory in your terminal and run the following command:

``` bash
trunk serve --release
```

[1]: https://en.wikipedia.org/wiki/Deferred_shading
[2]: https://lisyarus.github.io/blog/posts/point-light-attenuation.html
