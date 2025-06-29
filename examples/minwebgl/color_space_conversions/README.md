# Color Space Conversion Demo

This project demonstrates real-time color space conversion using Rust with `wasm-bindgen` for web assembly, leveraging the `minwebgl` and `color` crates. It provides a visual tool to pick an sRGB color and see its representation across various other color spaces.

![showcase](showcase.png)

---

## How it is useful

This demo showcases several useful techniques and concepts for working with color in web development with Rust and WebAssembly:

* **Color Space Conversions**: Learn how to convert colors between a wide array of color spaces, including `A98Rgb`, `Aces2065-1`, `AcesCg`, `DisplayP3`, `HSL`, `HWB`, `Lab`, `Lch`, `LinearSrgb`, `Oklab`, `Oklch`, `ProphotoRgb`, `Rec2020`, `XyzD50`, and `XyzD65`. This is powered by the `color` crate.
* **Real-time UI Updates**: Observe how to create a responsive web application where user input (color picking) instantly updates multiple UI elements.
* **DOM Manipulation**: Understand how to select and modify HTML elements, set CSS styles, and update text content directly from Rust.
* **Event Handling**: Learn how to set up event listeners for user interactions (e.g., `input` events on a color picker).

-----

## Running

Ensure you have all the necessary dependencies installed. This example uses `trunk` for building and serving the WebAssembly application.

To run the example:

1.  Navigate to the example's directory in your terminal.

2.  Run the command:

    ```bash
    trunk serve --release
    ```

3.  Open your web browser to the address provided by trunk (usually `http://127.0.0.1:8080`).