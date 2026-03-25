---
category: pattern
priority: critical
title: Original project intent and stack decision
context: All development decisions
tags: [architecture, stack, critical, confirmed-by-user]
learned_from: smart-init-confirmed
---

Oval is a standalone video player with a distinctive oval-shaped window and a glossy, futuristic "luminous doorknob" aesthetic inspired by Winamp's form factor innovation.

**Stack (confirmed by research and user):**
- Rust (core language)
- winit (cross-platform windowing)
- wgpu (GPU rendering — Metal on macOS, DX12/Vulkan on Windows)
- egui (immediate-mode UI overlay for controls)
- ffmpeg-next (video decoding, all codecs)
- cpal (audio output)
- objc2 (macOS platform interop)
- windows-rs (Windows platform interop)

**Platform scope:** macOS AND Windows. Cross-platform is a hard requirement.

**Mojo was evaluated and explicitly rejected** (see research/MOJO_EVAL.md). Reasons: no GUI support, no Windows support, no classes, immature FFI. Do not suggest Mojo for any part of this project.

**Why:** A previous Claude session drifted from this stack to Makepad + Mojo, dropping Windows support. The user explicitly reverted this drift on 2026-03-24 and confirmed the original research conclusions.

**How to apply:** Every implementation decision must target both macOS and Windows. Never suggest single-platform frameworks or tools.
