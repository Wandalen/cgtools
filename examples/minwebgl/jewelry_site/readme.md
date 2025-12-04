# Jewelry Site

**Keywords:** Web, Jewelry, Configurator, WebGL

This demo showcases a product‑ready 3D jewelry configurator built for real‑time, client‑side customization. Users can explore ring models, switch materials and gems, and preview combinations instantly.

![image](./showcase.png)

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
