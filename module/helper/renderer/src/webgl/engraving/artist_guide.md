# 3D Asset Pipeline Guide: Preparing Models for Dynamic Text Engraving (glTF / WebGL 2)

This document outlines the technical requirements for mesh geometry, UV mapping structures, and export settings when creating 3D models (rings, bracelets, pendants, cups) intended for integration with the dynamic WebGL text engraving subsystem (`src/webgl/engraving/`, shader block `USE_ENGRAVING` in `../shaders/main.frag`).

---

## 1. Geometry and Mesh Topology Requirements

* **Scale & Transformations:** Always apply all transformations prior to export: **`Ctrl + A` -> `All Transforms`** in Blender (Ensure `Scale = 1.0, 1.0, 1.0`).
* **Engraving Zone Topology:**
  * The face loop/strip designated for engraving must consist strictly of **Quads** with clean, regular topology.
  * Avoid triangles (Tris) and N-Gons inside the engraving strip.
  * Maintain uniform edge loop density across the entire length of the strip.
* **Normal Orientation:** All face normals must point outward (Face Orientation: Blue).

---

## 2. UV Map Structure (Strictly 2 Channels)

The mesh MUST contain **exactly two UV channels** in its mesh properties (`Object Data Properties` -> `UV Maps`):

| Channel in Blender | Default Name | Exported glTF Attribute | Purpose |
| :--- | :--- | :--- | :--- |
| **Channel 1** | `UVMap` | `TEXCOORD_0` | **Base Material:** PBR metal textures, satin finish, scratches, micro-surface, and normal maps. Unwrapped using standard 3D texturing rules. |
| **Channel 2** | `UVMap.001` | `TEXCOORD_1` | **Engraving Zone:** Dedicated UV mapping for dynamic text rendering. Must strictly follow the unwrapping rules below. |

> **Important — tangent-space alignment:** the shader builds a single tangent frame per fragment (from the glTF `TANGENT` attribute if present, otherwise a derivative-based fallback — see §6) and that frame is always computed against **`TEXCOORD_0`**, never `TEXCOORD_1`. The engraving groove's relief (which way letter edges catch the light) is then shaded using that same `TEXCOORD_0`-aligned frame. If `TEXCOORD_1`'s U/V axes are rotated relative to `TEXCOORD_0`'s at the engraving zone (e.g. `TEXCOORD_0` is a rotated island in an atlas), the text mask itself will still be correct, but the carved-groove shading will look rotated relative to the letters. Keep `TEXCOORD_0`'s U/V orientation at the engraving strip aligned with `TEXCOORD_1`'s (same left-to-right / up-down sense) to avoid this.

---

## 3. Unwrapping Rules for `TEXCOORD_1` (UV2)

The second UV channel (`UVMap.001`) acts as a projection grid for the offscreen HTML5 Canvas text texture in WebGL shaders.

```text
0,1 ┌──────────────────────────────────────────┐ 1,1
    │ E N G R A V I N G   Z O N E  ( T E X T ) │  <-- Full strip
0,0 └──────────────────────────────────────────┘ 1,0
```

### 3.1. Active Engraving Faces

1. **Rectangulate Strip:** Unwrap the face loop into a perfectly straight, flat rectangle (In Blender: `Unwrap` -> `Follow Active Quads` -> `Even`).
2. **Fill Bounding Box (1:1):** Scale and position the rectangulated strip so its corners **precisely align with the `[0.0, 0.0]` to `[1.0, 1.0]` UV bounding box**.
   * *Do NOT leave margins or padding (Margin = 0.0).*
   * *The island must occupy 100% of the active UV space.*
3. **Reading Direction:** Text maps left-to-right. Ensure the left edge of the mesh strip maps to $X = 0.0$ and the right edge maps to $X = 1.0$.
4. **Mark seams around the strip:** before unwrapping, mark a UV seam along the boundary between the engraving strip and the rest of the mesh. Without a seam there, boundary vertices are shared between the `[0,1]` island and the collapsed `(-1,-1)` region below (§3.2), and a single triangle spanning both would interpolate UV1 across that huge jump, corrupting the mask/gradient sampling right at the edge of the engraving zone.

### 3.2. Unused Mesh Faces (Rest of the Model)

For all other mesh faces where engraving **does NOT** occur:

1. Select them in the 2nd UV channel.
2. Collapse them into a single vertex point (`Scale -> 0`).
3. Translate the collapsed point outside the active UV bounds to coordinates **`(-1.0, -1.0)`**.

> **Technical Context:** The WebGL shader checks if UV coordinates fall within the `[0.0, 1.0]` range (inclusive) and only samples the mask texture when they do (`main.frag`: `engravingInBounds = all(greaterThanEqual(...)) && all(lessThanEqual(...))`). Moving unused geometry to `(-1.0, -1.0)` — anywhere outside `[0,1]` works, this is just a simple, unambiguous choice — prevents it from ever sampling the mask texture. Leaving unused faces at the UV origin `(0,0)` instead would be wrong: that's a valid in-bounds coordinate and would sample the corner of the actual text texture.

---

## 4. Physical Aspect Ratio Calculation

Because the engraving strip in `TEXCOORD_1` is intentionally stretched into a 1:1 UV square `[0..1] x [0..1]`, the frontend developer must know the **real physical aspect ratio** of the strip to render the HTML5 Canvas at matching dimensions, preventing text stretching or compression.

Artists must calculate and supply the physical aspect ratio:

$$\text{Aspect Ratio} = \frac{\text{Strip Length } (L)}{\text{Strip Height } (H)}$$

### Measuring Strip Dimensions in Blender

1. **Height ($H$):** Length of one vertical edge along the strip.
2. **Length ($L$):**
   * *For 360° Full-Circle Rings/Cylinders:* $L = \pi \times D \approx 3.14159 \times \text{Outer/Inner Diameter}$.
   * *For Partial Mesh Strips:* $L = \text{Length of 1 horizontal edge} \times \text{Number of segment quads in the strip}$.

> **Example:** Circumference length $L = 62.8\text{ mm}$, height $H = 2.0\text{ mm}$.
> $\text{Aspect Ratio} = 62.8 / 2.0 = 31.4$. The developer will initialize an offscreen Canvas at $4096 \times 128\text{ px}$.

---

## 5. Metadata Transfer (`engraving_config.json`)

Accompany each exported `.glb` model with an `engraving_config.json` parameter file (or include it in the task description), validated against [`engraving_config.schema.json`](engraving_config.schema.json). `engraving_config.json`'s top-level `nodes` is an **array** — a piece of jewelry with several independent engraving zones (a ring band's inner surface, a pendant tag, each link of a charm bracelet, ...) gets one array entry per zone, each with its own `nodeName`, `aspectRatio`, physical strip height, and so on. There is no shared/inherited state between entries: every node is resolved and sized completely independently (`EngravingSession::build` walks `nodes` and applies each one to its own mesh node), so a ring band's `stripHeightMm: 2.5` and a much smaller charm's `stripHeightMm: 1.2` coexist correctly in the same file. `nodeName`, `uvChannel`, `aspectRatio`, `maxCharacters`, `defaultFont` and `allowedFonts` are **required** on every entry — the whole config fails to load if any entry is missing them:

```json
{
  "nodeName": "Ring_Band_Mesh",
  "uvChannel": 1,
  "aspectRatio": 31.4,
  "maxCharacters": 35,
  "defaultFont": "Roboto",
  "allowedFonts": [ "Roboto", "Playfair Display" ]
}
```

* `nodeName`: Exact name of the Mesh node in Blender's Outliner hierarchy (this is what ends up in the glTF and is matched against at runtime).
* `uvChannel`: UV channel index (`1` corresponds to `TEXCOORD_1`). Must not be a channel already used by any base-color/normal/etc. texture on the same material.
* `aspectRatio`: Calculated physical ratio ($L / H$).
* `maxCharacters`: Upper bound on text length; enforced before rasterizing.
* `defaultFont` / `allowedFonts`: CSS font-family whitelist for this node's engraving text. There is no `defaultText` field — initial text is set at runtime by the calling application (`EngravingSession::set_text`), not baked into this config.

Optional fields (`textureHeight`, `padding`, `engravingStrength`, `engravingRoughness`, `engravingDarkening`) all have sane defaults — see [`engraving_config.schema.json`](engraving_config.schema.json) for the full list, defaults and validation ranges, and [`engraving_config.example.json`](engraving_config.example.json) for a complete three-node example on one piece of jewelry: a full-circle ring band (`HYBRID` sizing), a bracelet charm link (`PHYSICAL` sizing, a much smaller and differently-shaped strip), and a pendant tag (no sizing fields at all, demonstrating the `RELATIVE` auto-fit fallback).

### 5.1. Physical Text Sizing (Optional)

By default (no fields below set) engraving text auto-fits: it grows to the largest size that fits inside the strip's padding on every side, so short and long text end up at different physical sizes on the model. If the physical dimensions from §4 matter — e.g. a jeweler's spec requiring "1.5mm tall lettering" regardless of how much text is typed — supply:

* `stripHeightMm`: same $H$ you measured in §4.
* `defaultFontSizeMm`: the target physical letter height.
* `minFontSizeMm` (optional): the smallest the engine may shrink text to before it refuses further overflow instead of continuing to shrink.
* `sizingMode` (optional): `"HYBRID"` (shrink toward `minFontSizeMm` on overflow — the default once the two fields above are both set), `"PHYSICAL"` (never shrink; overflowing text is rejected outright), or `"RELATIVE"` (the auto-fit default; set explicitly together with `fontSizeRatio` to pin text to a fixed fraction of the strip height instead of auto-fitting).

These are set per node, so a multi-zone piece isn't forced to use one strip height everywhere. In [`engraving_config.example.json`](engraving_config.example.json): `RingBand_Inner` is a worked `HYBRID` example (a 2.5mm-tall strip targeting 1.5mm letters, shrinking no further than 0.8mm), `Charm_Bracelet_Link` is a much smaller, unrelated `PHYSICAL` strip (1.2mm tall, 0.8mm letters, no shrinking) on the same piece, and `PendantTag_Back` opts out of physical sizing entirely.

---

## 6. glTF 2.0 (.glb) Export Settings

When exporting from Blender (**`File` -> `Export` -> `glTF 2.0 (.glb)**`), use the following configuration:

* **Format:** `glTF Binary (.glb)`
* **Include:**
* [x] `Limit to Selected Objects`

* **Data -> Mesh:**
* [x] **`UVs`** *(Required: Exports both UV0 and UV1 channels)*
* [x] **`Normals`**
* [x] **`Tangents`** *(Optional: the shader falls back to a screen-space-derivative TBN when no `TANGENT` attribute is present, so this isn't a hard requirement. Exporting real tangents is still recommended — it's more stable at grazing angles and avoids a per-fragment derivative computation. Either way, tangents (real or derived) are always built against `TEXCOORD_0` — see the alignment note in §2.)*
* [x] **`Apply Modifiers`**

* **Data -> Material:**
  * `Export Materials` (PBR)
