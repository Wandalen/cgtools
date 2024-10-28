## Emboss filter

### Description

This example shows how to apply emboss filter to an image using `WebGl2` and `minwebgl`.
The filter is applied inside a circle of a `radius` defined in program. The circle center is
cursor position on the image.

### Point

The example shows how to deal with convolutional processing in `GLSL ES`, and how to create and use a
simple convolutional program in `WebGl2` and `minwebgl`.

### Running

In order to run the example navigate to example's directory and run `trunk serve` or `trunk serve --release`.
If you want to load own image, upload it into `resources` folder and then provide its path into `image_path`
variable.

``` rust
let image_path = "your_image.format";
```

Also you can change radius. Find the line containing radius definition and assign whatever value you like.

``` rust
let radius = 123.456;
```

---

![showcase](showcase.png)
