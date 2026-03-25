---
category: gotcha
priority: high
title: Known platform-specific issues from research
context: Window system, rendering, platform interop
tags: [platform, macOS, windows, gotcha]
learned_from: smart-init-discovery
---

**macOS:**
- winit + wgpu transparency: Reports that `eframe::NativeOptions::transparent` doesn't work with wgpu on Mac. Workaround: directly set CAMetalLayer.isOpaque via objc2.
- NSWindow needs: borderless style mask, isOpaque=false, backgroundColor=clear, hasShadow=true
- Hit-testing: macOS auto-passes clicks through transparent areas, but explicit hitTest override recommended for precision.

**Windows:**
- winit transparency bug (winit #851): combining with_transparent(true) + with_decorations(false) on Windows 10 can produce invisible window. Workaround: manually manage WS_EX_LAYERED.
- SetWindowRgn gives jagged edges (no AA) — not acceptable for glossy aesthetic.
- Recommended approach: Per-pixel alpha via UpdateLayeredWindow, or shader-based alpha with DWM composition.
- WM_NCHITTEST needed for proper hit-testing and window drag via HTCAPTION.

**How to apply:** Plan for these workarounds from the start. Don't rely on winit's built-in transparency — prepare platform-specific escape hatches.
