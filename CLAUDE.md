# CLAUDE.md - Oval Video Player Project

## Project Overview

Oval is a standalone video player with a distinctive oval-shaped, futuristic aesthetic inspired by Winamp's form factor innovation. The project has completed **Phase 1 (Research)** and is ready to begin **Phase 2 (Implementation)**.

**Key research outcome:** Mojo was evaluated and rejected for this project (see `research/MOJO_EVAL.md`). The implementation language is **Rust**, using wgpu for rendering, ffmpeg-next for video decoding, and winit for windowing.

## Repository Structure

```
Oval/
├── CLAUDE.md                    # This file - AI assistant guide
├── oval-player-architect.md     # Agent specification and project blueprint
├── research/
│   ├── MOJO_EVAL.md            # Mojo evaluation (verdict: not suitable)
│   ├── VIDEO_TECH.md           # Video codec and decoding research
│   ├── WINDOW_SYSTEM.md        # Platform-specific window management
│   └── UI_DESIGN.md            # Visual design, ASCII sketches, shader designs
└── design/
    └── ARCHITECTURE.md          # System architecture and implementation plan
```

## Technology Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Language | **Rust** | Mojo rejected — no GUI support, no Windows, no classes |
| Windowing | winit | Cross-platform, raw handle access for platform interop |
| GPU Rendering | wgpu | Metal (macOS), DX12/Vulkan (Windows) |
| UI Controls | egui | Immediate-mode overlay for timeline, transport |
| Video Decoding | ffmpeg-next | All codecs, hardware acceleration via hwaccel API |
| Audio Output | cpal | Cross-platform audio |
| macOS Interop | objc2 | NSWindow borderless + CAShapeLayer oval mask |
| Windows Interop | windows-rs | WS_EX_LAYERED + per-pixel alpha + WM_NCHITTEST |

## Key Technical Decisions

1. **Oval window:** Transparent borderless window + fragment shader alpha mask (not OS-level window regions)
2. **Video texture pipeline:** Upload YUV planes as separate GPU textures, convert to RGB in fragment shader (avoids CPU-side conversion)
3. **Glossy effect:** Shader-based specular highlight simulating curved reflective surface
4. **Seeking:** Keyframe-seek + decode-forward for frame accuracy; coarse scrub during drag, fine resolve on release
5. **Threading:** Decode thread (ring buffer) → Main thread (events + render) → Audio thread (cpal)
6. **Video scaling:** "Cover" mode by default (fill oval, crop edges) — maximizes immersion

## Implementation Sprints (Phase 2)

1. **Window + Oval Mask** — winit + wgpu + oval shader + platform config + hit-testing + dragging
2. **Video Playback** — ffmpeg-next integration, decode thread, YUV texture upload, playback
3. **Controls + Interaction** — egui overlay, timeline, scrubbing, play/pause
4. **Visual Polish** — glossy overlay shader, vignette, idle state, drag-and-drop, animations
5. **Hardware Accel + Audio** — VideoToolbox (macOS), DXVA (Windows), cpal audio, A/V sync
6. **Cross-Platform QA** — macOS + Windows testing, codec matrix, performance profiling

## Priority Hierarchy

1. Visual aesthetic fidelity (highest)
2. Video quality
3. Performance (60fps+ target)
4. Code maintainability (lowest)

## Conventions and Rules

- **No placeholder TODOs** — complete each feature fully before moving on
- **ASCII sketches are mandatory** before implementation of visual features (see `research/UI_DESIGN.md`)
- **Cross-platform consistency** — design for lowest common denominator, then enhance
- **Present trade-offs explicitly** when multiple implementation options exist
- Platform-specific code uses `#[cfg(target_os = "...")]` modules in `src/window/`

## Agent Configuration

The `oval-player-architect.md` file contains a Claude agent specification with YAML front matter:
- **Model:** Sonnet
- **Color:** Red
- **Purpose:** Orchestrates the entire development lifecycle from research through implementation

This agent should be activated when working on any aspect of the Oval player project.

## Git Workflow

- **Author:** Cameron Brooks <brooksc3@oregonstate.edu>
- **Commit signing:** Enabled (SSH key-based)
- Development branches follow `claude/` prefix convention
