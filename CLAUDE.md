# CLAUDE.md - Oval Video Player Project

## Project Overview

Oval is a standalone video player with a distinctive oval-shaped, futuristic aesthetic inspired by Winamp's form factor innovation. The project is currently in the **pre-implementation planning phase** with no source code yet written. The repository contains an agent specification document that defines the research and development roadmap.

## Repository Structure

```
Oval/
├── CLAUDE.md                    # This file - AI assistant guide
├── oval-player-architect.md     # Agent specification and project blueprint
├── research/                    # (planned) Research documentation
│   ├── MOJO_EVAL.md            # Mojo/framework analysis
│   ├── VIDEO_TECH.md           # Video codec and decoding research
│   ├── WINDOW_SYSTEM.md        # Platform-specific window management
│   └── UI_DESIGN.md            # Visual design and ASCII sketches
└── design/
    └── ARCHITECTURE.md          # (planned) System architecture
```

## Key Technical Context

- **Primary Language:** Mojo (with fallback to Rust/Swift/C++ if Mojo proves insufficient for GUI work)
- **Target Platforms:** macOS and Windows
- **Form Factor:** Large oval window, portrait dimensions (~9:16 aspect ratio)
- **Visual Style:** Futuristic/arcane - glossy metallic appearance, transparent non-rectangular window
- **Codec Support:** H.264, H.265/HEVC, VP9, AV1
- **Hardware Acceleration:** Metal (macOS), DirectX/Vulkan (Windows)

## Development Phases

### Phase 1 - Research (Current Phase)
Research must be completed **before any code is written**. Five research areas:
1. Language/Framework evaluation (Mojo capabilities for GUI)
2. Video technology (codecs, hardware acceleration, decoding libraries)
3. Window management (non-rectangular windows on macOS and Windows)
4. UI/Rendering architecture (glossy effects, scrubbing, shaders)
5. ASCII sketches and visual planning

Each research document must be comprehensive (1000+ words of substantive technical analysis).

### Phase 2 - Implementation
Sequential build order:
1. Basic window creation and oval masking
2. Video decoding and rendering pipeline
3. Playback controls (play/pause)
4. Scrubbing/seeking functionality
5. Glossy effects and visual polish
6. Cross-platform testing

## Priority Hierarchy

1. Visual aesthetic fidelity (highest)
2. Video quality
3. Performance (60fps+ target)
4. Code maintainability (lowest)

## Conventions and Rules

- **No coding before research is complete** - all research docs must exist first
- **No placeholder TODOs** - complete each feature fully before moving on
- **ASCII sketches are mandatory** before implementation of visual features
- **Verify Mojo capabilities** through documentation rather than assuming
- **Cross-platform consistency** - design for lowest common denominator, then enhance
- **Present trade-offs explicitly** when multiple implementation options exist

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
