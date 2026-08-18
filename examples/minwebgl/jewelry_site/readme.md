# Jewelry Site

**Keywords:** Web, Jewelry, Configurator, HTML, CSS, JS

<!-- Fix(BUG-XXX): claimed a 3D/WebGL configurator and listed WebGL as a keyword — this site has
     no canvas, no WebGL context, and no 3D library anywhere in its markup or scripts; it swaps
     between pre-rendered 2D preview images by filename. `src/main.rs` exists only as an inert
     wasm-bindgen placeholder so trunk's tooling accepts the crate — it renders nothing.
     Root cause: aspirational wording never checked against the actual implementation.
     Pitfall: this crate sits in a directory of genuine WebGL demos, making an unverified
     "WebGL"/"3D" claim here easy to accept at a glance instead of checking the actual scripts. -->
This demo showcases a product‑ready jewelry configurator built for real‑time, client‑side customization using pre-rendered 2D preview images. Users can browse ring designs, switch materials and gems, and preview combinations instantly.

![image](./showcase.webp)

---

## Features Overview

The jewelry configurator provides real‑time, interactive customization with the following UI elements:

* **Main page** - contains titles, text, buttons and ring images with animated traisitions recreated with GSAP when user scroll page
* **Transition between main page and configurator** - smooth transition between main page and configurator using GSAP
* **Metal Selector** – choose between *Copper*, *Gold*, or *Silver* in configurator
* **Gem Selector** – choose between *Emerald*, *Ruby*, or *White Crystal* in configurator
* **Ring Type Selector** – three ring designs: **1**, **2**, and **3** in configurator
* **Real-time Preview Update** – instantly updates the preview image when any option changes
* **Night Mode Toggle** – available in both landing page header and configurator header

  * Activated through a **moon icon button**
  * Switches UI color palette to a darker theme

All interactions are animated using GSAP and CSS for smooth transitions.

---

## How to Run

Any static server is enough. For example, using Python:

```bash
python -m http.server 8000
```

Then open:

```
http://localhost:8000/
```
