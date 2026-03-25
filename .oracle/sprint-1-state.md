---
category: state
priority: critical
title: Sprint 1 — Current State and Intent
context: Active sprint planning and execution
tags: [sprint-1, state, active]
updated: 2026-03-24
---

## Phase
Sprint 1: Window + Oval Mask (NOT STARTED — scaffolding phase)

## Intent
Produce a draggable iridescent pearl oval that sits on the macOS desktop as a physical object. Not a window — an object. The MDK + Riven aesthetic should be visible in the idle state: white pearl base with oil-slick thin-film interference shimmer, specular highlight tracking the mouse, and proper desktop transparency.

## Key Technical Findings (from expert investigation)

### CompositeAlphaMode — CRITICAL CORRECTION
The .oracle/shader-design.md says to use `CompositeAlphaMode::Auto`, but Context7 docs confirm that Auto only selects between `Opaque` or `Inherit`. For actual desktop transparency, the implementation MUST:
1. Query `surface.get_capabilities(&adapter)` for supported alpha modes
2. Explicitly select `PreMultiplied` if available (Metal on macOS supports this)
3. Fall back to `PostMultiplied` if PreMultiplied unavailable

### winit API (confirmed current)
- `ApplicationHandler` trait with `resumed()` and `window_event()`
- Window creation: `event_loop.create_window(Window::default_attributes().with_transparent(true).with_decorations(false))`
- Entry: `EventLoop::new()` + `event_loop.run_app(&mut app)`
- macOS extensions: `WindowAttributesExtMacOS` — `with_movable_by_window_background()`, `with_has_shadow()`
- NOTE: CLAUDE.md mentions `can_create_surfaces()` but the actual callback is `resumed()`

### wgpu API (confirmed current)
- `SurfaceConfiguration` with `alpha_mode: CompositeAlphaMode` field
- Must query capabilities before configuring
- `surface.get_capabilities(&adapter).alpha_modes` returns supported modes

## Resolved Uncertainties (researched 2026-03-24)
- **objc2 API for NSWindow access:** Use `HasWindowHandle` -> `AppKitWindowHandle.ns_view` -> cast to `&NSView` -> `.window()` -> `Retained<NSWindow>`. See `research/OBJC2_WINIT_INTEROP.md`.
- **drag_window() on borderless:** Yes, it works. Calls `[NSWindow performWindowDragWithEvent:]` under the hood, independent of style mask. Must be called during mouse-down event.
- **CAMetalLayer.isOpaque:** Must be set to false AFTER wgpu surface creation via objc2. `layer.setOpaque(false)` on the CALayer reference suffices (CAMetalLayer inherits from CALayer).
- **PreMultiplied alpha mode:** Confirmed supported on macOS Metal surfaces. Shader must output pre-multiplied alpha (`rgb * alpha, alpha`).
- **winit with_transparent(true):** Necessary but NOT sufficient alone. Must also set CAMetalLayer.isOpaque=false and use CompositeAlphaMode::PreMultiplied.

## Remaining Uncertainties
- Exact timing of CAMetalLayer availability after `create_surface()` (immediate or needs one frame?)
- Whether macOS auto-shadow correctly traces shader-based alpha oval shape (vs CAShapeLayer mask)
- Runtime verification of full transparency pipeline end-to-end

## Next Action
Step 1: Create Cargo.toml + main.rs scaffold in oval-player/
