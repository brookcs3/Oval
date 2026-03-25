# CLAUDE.md - Oval Video Player Project

## Project Overview

Oval is a standalone video player that exists as a mysterious oval-shaped object on the desktop. The visual identity crosses **MDK (1998)** with **Riven (2024 remake)**: a white pearl egg with oil-slick iridescent shimmer. Not a window — an object.

Phase 1 (Research) is complete. Phase 2 (Implementation) begins with Sprint 1.

## Visual Identity

**MDK + Riven hybrid.** Glossy biomechanical materiality meets ancient tactile technology. White/offwhite pearl base, thin-film interference iridescence, physical object presence. Video resolves through the surface on file load.

## Technology Stack (LOCKED)

| Layer | Technology | Notes |
|-------|-----------|-------|
| Language | **Rust** | Mojo was evaluated and rejected (see `research/MOJO_EVAL.md`) |
| Windowing | winit | Cross-platform, ApplicationHandler trait API |
| GPU Rendering | wgpu | Metal (macOS), DX12/Vulkan (Windows), WGSL shaders |
| UI Controls | egui | egui-wgpu + egui-winit for overlay controls |
| Video Decoding | ffmpeg-next | All codecs, hardware acceleration via hwaccel API |
| Audio Output | cpal | Cross-platform audio |
| macOS Interop | objc2 | NSWindow borderless + CAShapeLayer oval mask |
| Windows Interop | windows-rs | WS_EX_LAYERED + per-pixel alpha + WM_NCHITTEST |

**Do not change this stack without explicit user approval.** A prior session drifted to a different framework — the repo had to be reset.

## Repository Structure

```
Oval/
├── CLAUDE.md                    # This file
├── oval-player-architect.md     # Agent specification and project blueprint
├── research/
│   ├── MOJO_EVAL.md            # Mojo evaluation (verdict: not suitable)
│   ├── VIDEO_TECH.md           # Video codec and decoding research
│   ├── WINDOW_SYSTEM.md        # Platform-specific window management
│   └── UI_DESIGN.md            # Visual design, ASCII sketches, shader designs
├── design/
│   └── ARCHITECTURE.md          # System architecture and implementation plan
├── .oracle/                     # ClaudeShack knowledge base
└── .guardian/                   # ClaudeShack quality config
```

## Key Technical Decisions

1. **Oval window:** Transparent borderless window + WGSL fragment shader alpha mask (not OS-level window regions)
2. **Video texture pipeline:** Upload YUV planes as separate GPU textures, convert to RGB in fragment shader
3. **Glossy effect:** Shader-based thin-film interference + specular highlight simulating curved reflective surface
4. **Seeking:** Keyframe-seek + decode-forward for frame accuracy; coarse scrub during drag, fine resolve on release
5. **Threading:** Decode thread (ring buffer) → Main thread (events + render) → Audio thread (cpal)
6. **Video scaling:** "Cover" mode by default (fill oval, crop edges)

## Implementation Sprints (Phase 2)

1. **Window + Oval Mask** — winit + wgpu + oval shader + platform config + hit-testing + dragging
2. **Video Playback** — ffmpeg-next integration, decode thread, YUV texture upload, playback
3. **Controls + Interaction** — egui overlay, timeline, scrubbing, play/pause
4. **Visual Polish** — idle state animation, drag-and-drop, icon morphs, vignette
5. **Hardware Accel + Audio** — VideoToolbox (macOS), DXVA (Windows), cpal audio, A/V sync
6. **Cross-Platform QA** — macOS + Windows testing, codec matrix, performance profiling

## Priority Hierarchy

1. Visual aesthetic fidelity (highest) — the look IS the product
2. Video quality
3. Performance (60fps+ target)
4. Code maintainability (lowest)

## Conventions and Rules

- **No placeholder TODOs** — complete each feature fully before moving on
- **ASCII sketches are mandatory** before implementation of visual features (see `research/UI_DESIGN.md`)
- **Cross-platform always** — macOS + Windows. Use `#[cfg(target_os = "...")]` modules in `src/window/`
- **Present trade-offs explicitly** when multiple implementation options exist
- **Never change the stack** without explicit user approval

## Agent Configuration

The `oval-player-architect.md` file contains a Claude agent specification:
- **Model:** Sonnet
- **Color:** Red
- **Purpose:** Executes Phase 2 implementation, sprint by sprint

## Git Workflow

- **Author:** Cameron Brooks <brooksc3@oregonstate.edu>
- **Commit signing:** Enabled (SSH key-based)
- Development branches follow `claude/` prefix convention
