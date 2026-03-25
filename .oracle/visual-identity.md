---
category: pattern
priority: critical
title: Visual identity — MDK + Riven hybrid aesthetic
context: All visual/shader/UI work, idle state, video playback, interaction design
tags: [design, visual, shader, critical, confirmed-by-user]
learned_from: smart-init-confirmed
---

The visual identity is a cross between **MDK (1998)** and **Riven (2024 remake)**.

**MDK contribution:** Glossy biomechanical surfaces. Smooth, organic curves with reflective materiality. The cockpit viewport framing — looking *through* an organic shape at content. Surfaces that feel impossibly smooth and alive.

**Riven contribution:** Mysterious tactile objects. Ancient-feeling technology with physical presence. Devices that look like they shouldn't exist but feel completely real. Weathered, substantial, *there*. An object in a world, not a UI element.

**The Oval itself:**
- White/offwhite egg-like base color (NOT dark — a pearl, not obsidian)
- Oil-slick iridescent shimmer (thin-film interference — color shifts with viewing angle)
- Sits on the desktop as a PHYSICAL OBJECT, not a window
- Has presence — "oddly alive, like it shouldn't quite exist on your desktop"
- When a video file is loaded, video resolves through the surface

**Shader characteristics:**
- Primary specular highlight (upper oval) follows mouse position
- Secondary reflection (lower oval)
- Thin-film interference: color shifts based on surface angle (steeper at edges = more color shift)
- Iridescence strongest at edges, absent at center (physically correct for thin-film)
- Rim glow at the very edge — like a soap bubble or oil on water
- Depth shading: edges darken slightly for concavity/dimension

**Priority hierarchy:** Visual aesthetic > Video quality > Performance > Code maintainability.

**How to apply:** The look IS the product. Every shader decision must serve the "mysterious physical object" feel. ASCII sketches mandatory before visual features (see research/UI_DESIGN.md).
