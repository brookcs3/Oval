# CLAUDE.md - Oval Video Player Project

## Project Overview

Oval is a standalone video player with a distinctive oval-shaped, cyber-arcane aesthetic — glossy, refractive, like a smooth white doorknob from The Matrix crossed with MDK (1998). Everything is advanced and dystopian at the same time. Smooth, pretty, refracting.

**Phase 1 (Research) is complete.** The project is in **Phase 2 (Implementation)**.

**Stack:** Makepad (Rust GPU-native UI framework, 1.0 May 2025) + Mojo (GPU refraction engine). **macOS + Linux** (no Windows — Mojo limitation).

## Repository Structure

```
Oval/
├── CLAUDE.md                    # This file - AI assistant guide
├── oval-player-architect.md     # Agent specification (Phase 2 retooled)
├── research/                    # Completed Phase 1 research
│   ├── MOJO_EVAL.md            # Mojo eval → hybrid role: refraction engine
│   ├── VIDEO_TECH.md           # Video codec and decoding research
│   ├── WINDOW_SYSTEM.md        # Platform-specific window management
│   └── UI_DESIGN.md            # Visual design, ASCII sketches, shader designs
├── design/
│   └── ARCHITECTURE.md          # System architecture (Makepad + Mojo hybrid)
└── oval/                        # (Phase 2) Rust/Makepad application source
    ├── Cargo.toml
    └── src/
```

## Technology Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Language | **Rust** | Application core |
| UI Framework | **Makepad** | GPU-native, shader-based styling, 1.0 May 2025 |
| Refraction Engine | **Mojo** | GPU compute kernel for real-time glass refraction |
| Video Decoding | ffmpeg-next | All codecs, VideoToolbox hardware acceleration |
| Audio Output | cpal | Cross-platform audio |
| Platform | **macOS + Linux** | Metal (macOS), OpenGL (Linux). No Windows (Mojo limitation) |

## Why This Stack

- **Makepad** — brand new GPU-native framework where ALL rendering is on GPU. Styling is done with actual shaders. The glossy/refractive aesthetic is a first-class citizen, not a hack bolted on.
- **Mojo** — GPU compute kernels for the refraction effect. Light bends through the oval surface using real physics (Snell's law). The video underneath distorts like looking through curved glass. This is the "Milkdrop" of Oval — the thing people stare at.
- **macOS + Linux** — Mojo supports both. No Windows (Mojo limitation). Makepad handles both via Metal (macOS) and OpenGL (Linux).

## Key Technical Decisions

1. **Oval window:** Makepad transparent borderless window + shader-based oval mask
2. **Glass refraction:** Mojo GPU kernel computes per-pixel refraction. Video warps through curved glass surface. Specular highlights follow mouse. Chromatic aberration at edges.
3. **Video pipeline:** ffmpeg-next decode → YUV texture upload → Makepad shader compositing
4. **Mojo interop:** Mojo compiles to shared library (.dylib on macOS, .so on Linux), Rust calls via C FFI
5. **Video scaling:** "Cover" mode (fill oval, crop edges)
6. **TikTok format:** Portrait ~9:16, opinionated — this is what Oval plays

## Visual Identity

**Cyber-arcane.** Glossy metallic. The Matrix's white doorknob + MDK (1998). Advanced and dystopian. Smooth and pretty and refracting.

- The glass refraction is not decorative — it's physical. Light bends. Colors separate.
- Mouse movement shifts the specular highlight like turning a real glass object.
- Idle state: the oval IS the object. A luminous refractive shape. Drop a video into it.
- Playing state: video seen through curved glass. The refraction is always there, subtle but real.

## Implementation Sprints (Phase 2)

1. **Scaffold + Oval Window** — Makepad project, borderless transparent window, oval shader mask, hit-testing, dragging
2. **Video Playback** — ffmpeg-next integration, decode thread, texture upload, video in oval
3. **Mojo Refraction Engine** — GPU kernel, Snell's law refraction, chromatic aberration, FFI bridge
4. **Controls + Interaction** — Makepad UI overlay, timeline, scrubbing, play/pause
5. **Visual Polish** — idle state, drag-and-drop, animations, the full cyber-arcane look
6. **Audio + Sync** — cpal audio, A/V sync, VideoToolbox hardware acceleration

## Priority Hierarchy

1. Visual aesthetic fidelity (highest) — the look IS the product
2. Video quality
3. Performance (60fps+ target)
4. Code maintainability (lowest)

## Conventions and Rules

- **No placeholder TODOs** — complete each feature fully before moving on
- **ASCII sketches are mandatory** before implementation of visual features (see `research/UI_DESIGN.md`)
- **Best looking visuals, fastest path** — pick whatever gets the most impressive result quickest
- Platform-specific code in `src/platform/macos.rs`

## Agent Configuration

The `oval-player-architect.md` agent is retooled for Phase 2:
- **Model:** Sonnet
- **Color:** Red
- **Purpose:** Implementation execution — scaffold, build, ship

## Git Workflow

- **Author:** Cameron Brooks <brooksc3@oregonstate.edu>
- **Commit signing:** Enabled (SSH key-based)
- Development branches follow `claude/` prefix convention
