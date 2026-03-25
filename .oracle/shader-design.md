---
category: pattern
priority: high
title: WGSL shader architecture for oval pearl surface
context: Sprint 1 shader implementation, visual effects
tags: [shader, wgsl, iridescence, visual, implementation]
learned_from: smart-init-confirmed
---

**Oval WGSL fragment shader pipeline (single pass):**

1. **Ellipse SDF mask** — `d = (dx/rx)² + (dy/ry)²`, smoothstep AA at boundary, alpha=0 outside
2. **Pearl base** — `vec3(0.95, 0.95, 0.96)` warm white, depth shading at edges
3. **Thin-film iridescence** — Phase-shifted RGB sine waves (2π/3 apart), edge-strengthened via smoothstep mask, mouse offset rotates interference pattern, ~25% blend with pearl base
4. **Specular highlight** — Gaussian falloff `exp(-d² * k)`, tracks mouse, faint iridescent tint
5. **Secondary reflection** — Softer highlight in lower oval
6. **Fresnel rim glow** — Strongest iridescent color at very edge, soap bubble effect, ~35% intensity
7. **Composite** — clamp and output with oval alpha

**Thin-film technique:** Phase-shifted sine approach (fastest, good enough). The key insight: distance from center IS the viewing angle proxy since we don't have real 3D normals. `smoothstep(0.2, 0.9, dist)` for edge strengthening is physically correct (thin-film interference strongest at grazing angles).

**Critical wgpu setting:** Must query `surface.get_capabilities(&adapter)` and explicitly select `CompositeAlphaMode::PreMultiplied` (or `PostMultiplied` as fallback). Do NOT use `Auto` — it only selects between Opaque and Inherit and will NOT produce transparency. Without this, the window renders as a black rectangle.

**How to apply:** Follow this pipeline order. Start with steps 1-3 for Sprint 1 MVP, add 4-6 iteratively.
