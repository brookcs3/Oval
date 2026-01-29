# Oval Video Player — System Architecture

## 1. Architecture Overview

Based on the completed research phase (see `research/` directory), this document presents the complete system architecture for the Oval video player.

### Technology Stack Decision

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Language** | Rust | Safety, performance, mature ecosystem (Mojo evaluated and rejected — see `research/MOJO_EVAL.md`) |
| **Windowing** | winit | Cross-platform window creation with raw handle access |
| **GPU Rendering** | wgpu | Cross-platform GPU API (Metal on macOS, DX12/Vulkan on Windows) |
| **UI Controls** | egui | Immediate-mode UI for overlay controls (timeline, buttons) |
| **Video Decoding** | ffmpeg-next | FFmpeg Rust bindings — all codecs, hardware acceleration |
| **Audio Output** | cpal | Cross-platform audio playback |
| **Platform Interop** | objc2 (macOS), windows-rs (Windows) | Oval window masking, hit-testing |

### High-Level Architecture Diagram

```
┌────────────────────────────────────────────────────────────────┐
│                        USER INTERACTION                        │
│              Mouse, Keyboard, Drag & Drop, Scrub               │
└──────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────────┐
│                      APPLICATION CORE                          │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │  Event Loop  │  │  App State   │  │   File Manager       │ │
│  │  (winit)     │←→│  Machine     │  │   (open, drag-drop)  │ │
│  └──────┬───────┘  └──────┬───────┘  └──────────────────────┘ │
│         │                 │                                    │
│         ▼                 ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                   PLAYBACK ENGINE                        │ │
│  │                                                          │ │
│  │  ┌─────────────┐  ┌──────────┐  ┌────────────────────┐  │ │
│  │  │ Demuxer     │  │ Decoder  │  │ A/V Sync Clock     │  │ │
│  │  │ (avformat)  │→ │ (avcodec)│→ │ (audio-master)     │  │ │
│  │  └─────────────┘  └──────────┘  └────────────────────┘  │ │
│  │         ↕                ↕                ↕              │ │
│  │  ┌─────────────┐  ┌──────────┐  ┌────────────────────┐  │ │
│  │  │ Seek Engine │  │ HW Accel │  │ Frame Queue        │  │ │
│  │  │ (keyframe + │  │ (VTB/    │  │ (ring buffer,      │  │ │
│  │  │  decode fwd)│  │  DXVA)   │  │  2-4 frames)       │  │ │
│  │  └─────────────┘  └──────────┘  └────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                   RENDER PIPELINE                        │ │
│  │                                                          │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐ ┌──────────┐ │ │
│  │  │ Texture  │→ │ Video    │→ │ Glossy   │→│ Oval     │ │ │
│  │  │ Upload   │  │ Scaling  │  │ Overlay  │ │ Mask     │ │ │
│  │  │ (YUV     │  │ (cover/  │  │ (specular│ │ (alpha   │ │ │
│  │  │  planes) │  │  fit UV) │  │  shader) │ │  clip)   │ │ │
│  │  └──────────┘  └──────────┘  └──────────┘ └──────────┘ │ │
│  │                                                          │ │
│  │  ┌──────────┐  ┌──────────────────────────────────────┐  │ │
│  │  │ egui     │→ │ Controls Overlay                     │  │ │
│  │  │ Context  │  │ (timeline, transport, time display)  │  │ │
│  │  └──────────┘  └──────────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                 PLATFORM LAYER                           │ │
│  │                                                          │ │
│  │  ┌─────────────────────┐  ┌───────────────────────────┐ │ │
│  │  │  macOS              │  │  Windows                  │ │ │
│  │  │  NSWindow(borderless│  │  HWND + WS_EX_LAYERED    │ │ │
│  │  │  isOpaque=false)    │  │  Per-pixel alpha          │ │ │
│  │  │  CAShapeLayer mask  │  │  WM_NCHITTEST handler     │ │ │
│  │  │  Metal backend      │  │  DX12/Vulkan backend      │ │ │
│  │  └─────────────────────┘  └───────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

---

## 2. Module Structure

```
oval/
├── Cargo.toml
├── src/
│   ├── main.rs                    # Entry point, event loop setup
│   ├── app.rs                     # Application state machine
│   ├── playback/
│   │   ├── mod.rs
│   │   ├── demuxer.rs             # FFmpeg format context, packet reading
│   │   ├── decoder.rs             # Video/audio decoding (SW + HW)
│   │   ├── seeker.rs              # Frame-accurate seeking logic
│   │   ├── clock.rs               # A/V sync clock (audio-master)
│   │   └── frame_queue.rs         # Ring buffer for decoded frames
│   ├── render/
│   │   ├── mod.rs
│   │   ├── pipeline.rs            # wgpu render pipeline setup
│   │   ├── texture.rs             # Video frame → GPU texture upload
│   │   ├── shaders/
│   │   │   ├── oval.wgsl          # Oval mask + video + glossy overlay
│   │   │   └── yuv_convert.wgsl   # YUV→RGB conversion shader
│   │   ├── effects.rs             # Glossy overlay parameters
│   │   └── presenter.rs           # Frame presentation timing
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── controls.rs            # Transport controls (play/pause/skip)
│   │   ├── timeline.rs            # Timeline bar + scrub handle
│   │   ├── hover.rs               # Hover detection + fade animation
│   │   └── theme.rs               # Colors, sizes, animation curves
│   ├── window/
│   │   ├── mod.rs
│   │   ├── oval_window.rs         # Cross-platform oval window setup
│   │   ├── macos.rs               # NSWindow/CAShapeLayer (cfg target_os)
│   │   ├── windows.rs             # HWND/WS_EX_LAYERED (cfg target_os)
│   │   └── hit_test.rs            # Oval hit-testing (ellipse equation)
│   └── input/
│       ├── mod.rs
│       ├── drag.rs                # Window dragging logic
│       ├── scrub.rs               # Scrub gesture handling
│       └── drop.rs                # File drag-and-drop handling
├── shaders/
│   ├── oval.wgsl                  # Primary render shader
│   └── yuv_convert.wgsl           # Color space conversion
├── research/                      # Research phase documentation
│   ├── MOJO_EVAL.md
│   ├── VIDEO_TECH.md
│   ├── WINDOW_SYSTEM.md
│   └── UI_DESIGN.md
└── design/
    └── ARCHITECTURE.md            # This document
```

---

## 3. Data Flow

### 3.1 Video Playback Pipeline

```
File Open
    │
    ▼
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ avformat │     │ avcodec  │     │ Frame    │     │ wgpu     │
│ _open    │────→│ _send    │────→│ Queue    │────→│ Texture  │
│ _input() │     │ _packet()│     │ (ring    │     │ Upload   │
│          │     │ _receive │     │  buffer) │     │          │
│ Reads    │     │ _frame() │     │          │     │ Y/U/V    │
│ packets  │     │          │     │ 2-4      │     │ planes   │
│ from     │     │ Decodes  │     │ frames   │     │ → GPU    │
│ disk     │     │ to YUV   │     │ ahead    │     │ textures │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
                                                         │
                                                         ▼
                                                   ┌──────────┐
                                                   │ Fragment │
                                                   │ Shader   │
                                                   │          │
                                                   │ YUV→RGB  │
                                                   │ + Scale  │
                                                   │ + Gloss  │
                                                   │ + Mask   │
                                                   └──────────┘
                                                         │
                                                         ▼
                                                   ┌──────────┐
                                                   │ Present  │
                                                   │ to       │
                                                   │ Surface  │
                                                   └──────────┘
```

### 3.2 Seeking Pipeline

```
User Scrub Input
    │
    ├── During drag (coarse):
    │   │
    │   ▼
    │   av_seek_frame(AVSEEK_FLAG_BACKWARD)
    │   → Seek to nearest keyframe
    │   → Decode single frame
    │   → Display immediately
    │
    └── On release (fine):
        │
        ▼
        av_seek_frame(AVSEEK_FLAG_BACKWARD)
        → Seek to keyframe BEFORE target
        → Decode forward, discarding frames
        → Stop at target PTS
        → Display exact frame
        → Resume playback if was playing
```

### 3.3 Input Event Flow

```
winit Event Loop
    │
    ├── WindowEvent::CursorMoved
    │   → Update hover state (show/hide controls)
    │   → Update dynamic glossy highlight position (optional)
    │
    ├── WindowEvent::MouseInput (press)
    │   → Hit test: inside oval?
    │     → Inside control zone? → Start scrub / toggle play
    │     → Inside video zone? → Start drag-scrub / toggle play
    │     → Outside oval? → Pass through (HTTRANSPARENT)
    │
    ├── WindowEvent::MouseInput (release)
    │   → End scrub → fine seek to position
    │   → End drag → stop window move
    │
    ├── WindowEvent::DroppedFile
    │   → Open video file → start playback
    │
    ├── WindowEvent::KeyboardInput
    │   → Space → toggle play/pause
    │   → Left/Right → seek ±5 seconds
    │   → Escape → close
    │
    └── MainEventsCleared
        → Check frame queue for next frame
        → If frame PTS <= audio clock → upload + render
        → Request redraw
```

---

## 4. State Machine

```
                        ┌───────────┐
                        │   EMPTY   │
            ┌──────────→│ (no file) │
            │           └─────┬─────┘
            │                 │ File opened / dropped
            │                 ▼
            │           ┌───────────┐
            │           │  LOADING  │
            │           │ (demux +  │
            │           │  probe)   │
            │           └─────┬─────┘
            │                 │ First frame decoded
            │                 ▼
            │           ┌───────────┐
     File   │      ┌───→│  PAUSED   │←───┐
     error  │      │    │ (showing  │    │ Space / click
     or     │      │    │  frame)   │    │
     close  │      │    └─────┬─────┘    │
            │      │          │ Space / click
            │      │          ▼          │
            │      │    ┌───────────┐    │
            │      └────│  PLAYING  │────┘
            │           │ (decode + │
            │           │  render)  │
            │           └─────┬─────┘
            │                 │
            │                 ▼
            │           ┌───────────┐
            │           │ SCRUBBING │
            │           │ (seeking) │
            │           └─────┬─────┘
            │                 │ Release
            │                 ▼
            │          Return to PLAYING
            │          or PAUSED (whichever
            └──────── was active before scrub)
```

---

## 5. Threading Model

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Main Thread (winit event loop)                            │
│   ┌───────────────────────────────────────────────────┐    │
│   │ • Process winit events (input, resize, close)     │    │
│   │ • Update app state machine                        │    │
│   │ • Drive egui UI logic                             │    │
│   │ • Issue wgpu render commands                      │    │
│   │ • Present frames to surface                       │    │
│   └───────────────────────────────────────────────────┘    │
│                          ↕ ring buffer                      │
│   Decode Thread                                             │
│   ┌───────────────────────────────────────────────────┐    │
│   │ • Read packets from file (avformat)               │    │
│   │ • Decode video frames (avcodec, HW or SW)         │    │
│   │ • Convert color space if needed                   │    │
│   │ • Push decoded frames to ring buffer              │    │
│   │ • Handle seek commands from main thread           │    │
│   └───────────────────────────────────────────────────┘    │
│                                                             │
│   Audio Thread (managed by cpal)                            │
│   ┌───────────────────────────────────────────────────┐    │
│   │ • Pull audio samples from audio ring buffer       │    │
│   │ • Output to system audio device                   │    │
│   │ • Update audio clock (used for A/V sync)          │    │
│   └───────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

Communication between threads:
- **Main → Decode:** Commands (play, pause, seek) via `crossbeam-channel`
- **Decode → Main:** Decoded frames via `ringbuf` (lock-free ring buffer)
- **Audio → Main:** Audio clock position via `AtomicU64`

---

## 6. Key Crate Dependencies

```toml
[dependencies]
# Windowing
winit = "0.30"

# GPU Rendering
wgpu = "28.0"

# UI
egui = "0.29"
egui-wgpu = "0.29"
egui-winit = "0.29"

# Video Decoding
ffmpeg-next = "7.0"

# Audio
cpal = "0.15"

# Threading / Sync
crossbeam-channel = "0.5"
ringbuf = "0.4"

# Platform interop
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-app-kit = "0.2"
objc2-foundation = "0.2"
objc2-quartz-core = "0.2"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = ["Win32_UI_WindowsAndMessaging", "Win32_Graphics_Dwm"] }
```

---

## 7. Render Pipeline Detail

### 7.1 wgpu Pipeline Configuration

```
Pipeline Layout:
  Bind Group 0: Video textures
    - binding 0: Y plane (texture_2d<f32>)
    - binding 1: U plane (texture_2d<f32>)
    - binding 2: V plane (texture_2d<f32>)
    - binding 3: sampler (linear filtering)

  Bind Group 1: Uniforms
    - binding 0: struct {
        oval_size: vec2f,           // Window dimensions
        video_aspect: f32,          // Source video aspect ratio
        time: f32,                  // Animation time
        mouse_pos: vec2f,           // For dynamic highlight
        controls_opacity: f32,      // Fade state
        scale_mode: u32,            // 0=cover, 1=fit
      }

Vertex Shader:
  Full-screen quad (2 triangles covering viewport)
  Passes through UV coordinates

Fragment Shader:
  1. Calculate oval distance from center
  2. Discard fragments outside oval (+ AA edge)
  3. Map UV to video texture with aspect correction
  4. Sample Y, U, V planes → convert to RGB
  5. Apply vignette
  6. Add glossy specular highlights
  7. Output with oval alpha
```

### 7.2 Render Pass Order

```
Pass 1: Video + Effects (main pipeline)
  → Renders video frame with oval mask and glossy overlay
  → Output: RGBA texture with alpha

Pass 2: UI Overlay (egui)
  → Renders transport controls, timeline, time display
  → Composited on top of Pass 1 output
  → Respects oval boundary (clipped)

Present:
  → Swap chain present to window surface
  → Platform compositor handles transparency
```

---

## 8. Implementation Order

### Sprint 1: Window + Oval Mask
1. Set up Rust project with Cargo
2. Create winit window (borderless, transparent)
3. Initialize wgpu surface and pipeline
4. Implement oval fragment shader (static color + mask + AA)
5. Platform-specific window configuration (macOS: NSWindow, Windows: WS_EX_LAYERED)
6. Hit-testing (clicks outside oval pass through)
7. Window dragging
8. **Deliverable:** A visible, draggable oval shape on screen

### Sprint 2: Video Playback
1. Integrate ffmpeg-next (demux + decode)
2. Decode thread with ring buffer
3. YUV texture upload to wgpu
4. YUV→RGB conversion in shader
5. Video scaling (cover mode) within oval
6. Basic play from start, stop at end
7. **Deliverable:** Video plays inside the oval

### Sprint 3: Controls + Interaction
1. Integrate egui for overlay UI
2. Hover detection + control fade animation
3. Play/pause toggle (click + spacebar)
4. Timeline bar rendering
5. Basic seeking (click on timeline)
6. Drag-to-scrub with coarse/fine seek
7. Time display
8. **Deliverable:** Fully interactive video player

### Sprint 4: Visual Polish
1. Glossy overlay shader (specular highlight)
2. Vignette effect
3. Dynamic highlight (follows mouse, optional)
4. Idle state (no-video "doorknob" appearance)
5. Drag-and-drop file opening
6. Smooth animation for all state transitions
7. **Deliverable:** "Luminous doorknob" aesthetic achieved

### Sprint 5: Hardware Acceleration + Audio
1. Enable VideoToolbox hwaccel (macOS)
2. Enable DXVA2/D3D11VA hwaccel (Windows)
3. Audio decoding and output (cpal)
4. A/V synchronization (audio-master clock)
5. **Deliverable:** Hardware-accelerated playback with audio

### Sprint 6: Cross-Platform QA
1. Test on macOS (Intel + Apple Silicon)
2. Test on Windows 10 and 11
3. Test with H.264, H.265, VP9, AV1 content
4. Performance profiling (target: 60fps continuous)
5. Memory leak checks
6. Edge cases (very short videos, 4K+, corrupt files)
7. **Deliverable:** Production-ready application

---

## 9. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| winit transparency bug on Windows | High | Manual WS_EX_LAYERED management, bypass winit's transparency |
| wgpu + transparency on macOS | Medium | Direct CAMetalLayer.isOpaque manipulation via objc2 |
| FFmpeg build complexity | Medium | Use pre-built FFmpeg binaries + vcpkg |
| Frame drops during scrub | Medium | Keyframe-only coarse scrub, async decode-ahead |
| HW accel not available | Low | Automatic fallback to software decode |
| egui rendering within oval | Low | Clip egui output to oval region in shader |

---

## 10. Success Metrics

Per the original specification:

- [ ] Renders as a transparent oval window on screen
- [ ] Plays modern video formats (H.264, H.265, VP9, AV1)
- [ ] Hardware-accelerated decoding on supported hardware
- [ ] Drag-to-scrub timeline control with frame-accurate seeking
- [ ] Glossy, futuristic "luminous doorknob" aesthetic during playback
- [ ] Works on macOS and Windows
- [ ] 60fps+ rendering without frame drops
- [ ] Proper hit-testing (clicks outside oval pass through)
- [ ] Window dragging support
- [ ] Drag-and-drop file opening
