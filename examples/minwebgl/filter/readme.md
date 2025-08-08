## Emboss filter

This example shows how to apply emboss filter to an image using `WebGl2`.
The filter is applied inside a circle of a `radius` defined in program. The circle center is
cursor position on the image.

### How it is useful

The example shows how to deal with convolutional processing, and how to create and use a
simple convolutional program in `WebGl2`.

### Running

Make sure you have installed all the necessary [dependencies](../../../module/min/minwebgl/readme.md)
In order to run the example navigate to example's directory and run next command:
``` bash
trunk serve
```
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
