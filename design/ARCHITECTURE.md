# Oval Video Player — System Architecture

## 1. Architecture Overview

**Stack:** Makepad (Rust, GPU-native) + Mojo (GPU refraction engine). **macOS only.**

This architecture replaces the earlier winit/wgpu/egui plan. Makepad handles windowing, rendering, and UI natively — no separate windowing or GPU crate needed. Mojo provides a GPU compute kernel for the real-time glass refraction effect (the "Milkdrop" of Oval).

### Technology Stack

| Layer | Technology | Role |
|-------|-----------|------|
| **UI Framework** | Makepad 1.0 | Window, rendering, shaders, widgets, events — everything |
| **Refraction Engine** | Mojo | GPU kernel: Snell's law refraction, chromatic aberration |
| **Video Decoding** | ffmpeg-next | Demux, decode, hardware accel (VideoToolbox) |
| **Audio Output** | cpal | Audio playback, A/V sync clock source |
| **Interop** | C FFI | Mojo .dylib loaded by Rust at runtime |
| **Platform** | macOS only | Metal backend via Makepad's platform layer |

### Why Makepad (Not winit + wgpu + egui)

- Makepad is **GPU-native** — all rendering happens on GPU, styling IS shaders
- Single crate (`makepad-widgets`) replaces winit + wgpu + egui + objc2
- Built-in `Sdf2d` API for shape rendering (oval mask, glow, boolean ops)
- `live_design!` macro for shader + layout DSL with hot-reload
- `#[live]` fields on `DrawQuad` automatically become shader uniforms
- `WindowDragQuery` event for native window dragging without title bar
- Eliminates the platform interop layer entirely — Makepad handles NSWindow internally

### High-Level Architecture Diagram

```
┌────────────────────────────────────────────────────────────────┐
│                        USER INTERACTION                        │
│              Mouse, Keyboard, Drag & Drop, Scrub               │
└──────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────────┐
│                    MAKEPAD APPLICATION                          │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │  Event Loop  │  │  App State   │  │   File Manager       │ │
│  │  (Makepad)   │←→│  Machine     │  │   (open, drag-drop)  │ │
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
│  │  │ (keyframe + │  │ (Video   │  │ (ring buffer,      │  │ │
│  │  │  decode fwd)│  │ Toolbox) │  │  2-4 frames)       │  │ │
│  │  └─────────────┘  └──────────┘  └────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                 MAKEPAD RENDER PIPELINE                   │ │
│  │                                                          │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐ ┌──────────┐ │ │
│  │  │ Texture  │→ │ Video    │→ │ Mojo     │→│ Oval     │ │ │
│  │  │ Upload   │  │ Scaling  │  │ Refract  │ │ Mask +   │ │ │
│  │  │ (YUV →   │  │ (cover   │  │ (FFI to  │ │ Iridesc  │ │ │
│  │  │  shader) │  │  mode)   │  │  .dylib) │ │ + Gloss  │ │ │
│  │  └──────────┘  └──────────┘  └──────────┘ └──────────┘ │ │
│  │                                                          │ │
│  │  ┌──────────────────────────────────────────────────────┐│ │
│  │  │ Makepad Widgets: Transport, Timeline, Time Display   ││ │
│  │  │ (native Makepad widgets, not egui)                   ││ │
│  │  └──────────────────────────────────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │              MAKEPAD PLATFORM LAYER                       │ │
│  │                                                          │ │
│  │  macOS only — Metal backend                              │ │
│  │  NSWindow: borderless, transparent, isOpaque=false       │ │
│  │  WindowDragQuery for oval-area dragging                  │ │
│  │  Hit-testing via ellipse equation in event handler       │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

---

## 2. Module Structure

```
oval-player/
├── Cargo.toml
├── src/
│   ├── main.rs                    # Entry point (pub use makepad_widgets, app_main)
│   ├── app.rs                     # Makepad App: live_design, DrawOval shader,
│   │                              #   event handling, state machine
│   ├── playback/                  # (Sprint 2)
│   │   ├── mod.rs
│   │   ├── demuxer.rs             # FFmpeg format context, packet reading
│   │   ├── decoder.rs             # Video/audio decoding (SW + VideoToolbox)
│   │   ├── seeker.rs              # Frame-accurate seeking logic
│   │   ├── clock.rs               # A/V sync clock (audio-master)
│   │   └── frame_queue.rs         # Ring buffer for decoded frames
│   ├── refraction/                # (Sprint 3)
│   │   ├── mod.rs
│   │   ├── bridge.rs              # Mojo .dylib FFI: load, call kernel
│   │   └── fallback.rs            # Pure-shader fallback if .dylib missing
│   └── mojo/                      # (Sprint 3) Mojo source
│       ├── refraction_kernel.mojo # GPU kernel: Snell's law, chromatic aberration
│       └── build.sh               # Compile to .dylib
├── research/
│   ├── MOJO_EVAL.md
│   ├── VIDEO_TECH.md
│   ├── WINDOW_SYSTEM.md
│   └── UI_DESIGN.md
└── design/
    └── ARCHITECTURE.md            # This document
```

**Note:** Makepad's architecture is more monolithic than winit+wgpu+egui. The `app.rs` file contains the `live_design!` macro which defines layout, shaders, and widget tree in a single DSL block. Separate Rust modules handle playback logic and Mojo FFI, but the visual layer lives in `live_design!`.

---

## 3. Data Flow

### 3.1 Video Playback Pipeline

```
File Open (drag-drop or file dialog)
    │
    ▼
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ avformat │     │ avcodec  │     │ Frame    │     │ Makepad  │
│ _open    │────→│ _send    │────→│ Queue    │────→│ Texture  │
│ _input() │     │ _packet()│     │ (ring    │     │ Upload   │
│          │     │ _receive │     │  buffer) │     │          │
│ Reads    │     │ _frame() │     │          │     │ YUV →    │
│ packets  │     │          │     │ 2-4      │     │ DrawQuad │
│ from     │     │ Decodes  │     │ frames   │     │ texture  │
│ disk     │     │ to YUV   │     │ ahead    │     │ uniform  │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
                                                         │
                                                         ▼
                                                   ┌──────────┐
                                                   │ DrawOval │
                                                   │ Shader   │
                                                   │          │
                                                   │ YUV→RGB  │
                                                   │ + Cover  │
                                                   │ + Refract│
                                                   │ + Gloss  │
                                                   │ + Mask   │
                                                   └──────────┘
                                                         │
                                                         ▼
                                                   ┌──────────┐
                                                   │ Makepad  │
                                                   │ Present  │
                                                   │ (Metal)  │
                                                   └──────────┘
```

### 3.2 Mojo Refraction Pipeline (Sprint 3)

```
Per frame:
    │
    ├── Rust side:
    │   1. Upload decoded video frame to shared buffer
    │   2. Pass uniforms: mouse_pos, oval_normals, time
    │   3. Call Mojo kernel via FFI
    │
    ├── Mojo side (.dylib):
    │   1. Read video pixels + uniforms
    │   2. For each pixel:
    │      a. Compute surface normal from oval curvature
    │      b. Apply Snell's law: refraction angle from IOR
    │      c. Sample video at refracted UV
    │      d. Split RGB channels slightly (chromatic aberration)
    │      e. Add Fresnel-based specular highlight
    │   3. Write refracted pixels to output buffer
    │
    └── Rust side:
        1. Upload Mojo output as texture
        2. Composite in DrawOval shader (iridescence + mask)
```

### 3.3 Input Event Flow (Makepad)

```
Makepad Event Loop
    │
    ├── Event::WindowDragQuery(dq)
    │   → Ellipse hit-test on dq.abs
    │   → Inside oval? → dq.response.set(Caption) → OS drags window
    │   → Outside oval? → NoAnswer → click passes through
    │
    ├── Hit::FingerHoverOver(fe)
    │   → Compute local UV from fe.abs / fe.rect
    │   → Update draw_oval.mouse_offset uniform
    │   → Redraw (iridescent highlight follows cursor)
    │   → Show/hide transport controls
    │
    ├── Hit::FingerDown(fe)
    │   → Inside timeline zone? → Start scrub
    │   → Inside video zone? → Toggle play/pause
    │
    ├── Hit::FingerMove(fe) [during scrub]
    │   → Map horizontal position to timeline
    │   → Coarse seek (keyframe)
    │
    ├── Hit::FingerUp(fe)
    │   → End scrub → fine seek to exact frame
    │   → Resume playback if was playing
    │
    ├── Event::FileDrop
    │   → Open video file → start playback
    │
    └── Hit::KeyDown
        → Space → toggle play/pause
        → Left/Right → seek ±5 seconds
        → Escape → close
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
│   Main Thread (Makepad event loop)                          │
│   ┌───────────────────────────────────────────────────┐    │
│   │ • Process Makepad events (Hit, WindowDragQuery)   │    │
│   │ • Update app state machine                        │    │
│   │ • Update DrawOval shader uniforms                 │    │
│   │ • Draw widgets via live_design                    │    │
│   │ • Call Mojo refraction kernel (FFI)               │    │
│   │ • Present via Makepad/Metal                       │    │
│   └───────────────────────────────────────────────────┘    │
│                          ↕ ring buffer                      │
│   Decode Thread                                             │
│   ┌───────────────────────────────────────────────────┐    │
│   │ • Read packets from file (avformat)               │    │
│   │ • Decode video frames (avcodec + VideoToolbox)    │    │
│   │ • Push decoded YUV frames to ring buffer          │    │
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
# UI Framework (handles windowing, rendering, widgets, shaders)
makepad-widgets = "1.0.0"

# Video Decoding
ffmpeg-next = "7.0"

# Audio
cpal = "0.15"

# Threading / Sync
crossbeam-channel = "0.5"
ringbuf = "0.4"
```

**Note:** No winit, wgpu, egui, or platform interop crates needed. Makepad handles all of that internally via its platform layer + Metal backend.

---

## 7. Render Pipeline Detail

### 7.1 Makepad DrawOval Shader

Makepad shaders are defined inline in `live_design!` and compile to Metal (macOS). Uniforms are `#[live]` fields on the `DrawQuad`-derived struct.

```
DrawOval struct fields (→ shader uniforms):
  draw_super: DrawQuad          // base quad (provides self.pos, self.rect_size)
  mouse_offset: Vec2            // cursor position, normalized 0..1
  time: f32                     // animation time for idle state pulsing
  video_texture: Texture        // (Sprint 2) decoded video frame
  refraction_texture: Texture   // (Sprint 3) Mojo refraction output

Shader pipeline (single pass in DrawOval::pixel):
  1. Compute ellipse SDF distance
  2. Discard fragments outside oval (alpha = 0)
  3. If video playing:
     a. Sample video texture with cover-mode UV mapping
     b. If Mojo refraction available: sample refraction texture instead
  4. Apply thin-film iridescence (phase-shifted RGB sines)
     - Strongest at edges (steep viewing angle)
     - Mouse offset rotates interference pattern
  5. Add specular highlight (follows mouse)
  6. Add secondary bottom reflection
  7. Add iridescent rim glow
  8. Apply depth shading
  9. Output with oval alpha mask
```

### 7.2 Makepad Widget Tree

```
<Root>
  <Window>                          // borderless, transparent
    caption_bar = { visible: false }
    pass: { clear_color: #0000 }
    body = <View>
      <OvalView>                    // custom widget
        draw_bg: <DrawOval>         // the shader
        <TransportControls>         // (Sprint 4) play/pause/skip
        <Timeline>                  // (Sprint 4) scrub bar
        <TimeDisplay>               // (Sprint 4) MM:SS / H:MM:SS
```

---

## 8. Implementation Sprints

### Sprint 1: Scaffold + Oval Window ← CURRENT
1. ~~Create Rust project with Makepad~~ ✓
2. ~~DrawOval shader: white pearl + iridescent sheen~~ ✓
3. Borderless transparent window (NSWindow isOpaque=false)
4. Oval hit-testing via ellipse equation
5. Window dragging via WindowDragQuery
6. Mouse tracking → shader mouse_offset uniform
7. **Done when:** Draggable iridescent oval on macOS desktop

### Sprint 2: Video Playback
1. ffmpeg-next integration (demux + decode thread)
2. Ring buffer for decoded frames
3. YUV texture upload to Makepad
4. YUV→RGB conversion in DrawOval shader
5. Video scaled to cover mode within oval
6. **Done when:** Video plays inside the oval

### Sprint 3: Mojo Refraction Engine
1. Mojo GPU kernel: per-pixel Snell's law refraction
2. Chromatic aberration (RGB channel separation)
3. Compile to .dylib, Rust loads via C FFI
4. Specular highlight follows mouse with dampening
5. Composite refracted output in DrawOval
6. **Done when:** Video refracts through curved glass

### Sprint 4: Controls + Interaction
1. Transport controls (Makepad widgets)
2. Timeline bar with scrub handle
3. Hover → controls fade in/out (300ms)
4. Coarse scrub during drag, fine seek on release
5. Time display, spacebar toggle
6. **Done when:** Fully interactive player

### Sprint 5: Visual Polish
1. Idle state: animated pulsing specular on white pearl
2. "DROP VIDEO HERE" indicator
3. Drag-and-drop file opening
4. Icon morph animations, scrub handle glow states
5. **Done when:** Full cyber-arcane aesthetic

### Sprint 6: Audio + Hardware Acceleration
1. cpal audio output + A/V sync
2. VideoToolbox hardware acceleration
3. Performance profiling (60fps target)
4. **Done when:** Production-ready

---

## 9. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Makepad transparent window on macOS | High | May need to patch platform layer (setOpaque:NO) |
| Makepad texture upload API (for video frames) | Medium | Study fractal_zoom example, may need raw Metal interop |
| Mojo .dylib FFI stability | Medium | Pure-shader fallback in `refraction/fallback.rs` |
| Mojo GPU kernel performance | Low | Kernel is simple (per-pixel refraction), well within GPU budget |
| ffmpeg-next build on macOS | Low | Homebrew ffmpeg + pkg-config |

---

## 10. Success Metrics

- [ ] Renders as a transparent oval window on macOS desktop
- [ ] White pearl surface with oil-slick iridescent sheen
- [ ] Specular highlight follows mouse movement
- [ ] Plays modern video formats (H.264, H.265, VP9, AV1)
- [ ] VideoToolbox hardware-accelerated decoding
- [ ] Real-time glass refraction via Mojo GPU kernel
- [ ] Drag-to-scrub timeline with frame-accurate seeking
- [ ] 60fps+ rendering without frame drops
- [ ] Clicks outside oval pass through to desktop
- [ ] Window dragging by clicking inside oval
- [ ] Drag-and-drop file opening
