# UI Design Research for Oval Player

## Executive Summary

This document covers the visual design, UI element layout, shader-based glossy effects, and frame-accurate scrubbing interaction design for the Oval video player. All UI must exist within the oval constraint, prioritizing the "luminous doorknob" aesthetic while maintaining usability.

---

## 1. ASCII Sketches

### 1.1 Overall Window Shape and Proportions

```
          ┌─── 450px (logical) ───┐

              ╭─────────────╮                ─┬─
            ╭─┘               └─╮              │
          ╭─┘   GLOSSY OVERLAY   └─╮           │
        ╭─┘   (frosted highlight)   └─╮        │
       ╭┘                               ╮     │
      ╭┘                                 ╮    │
     ╭┘                                   ╮   │
     │                                     │   │
     │                                     │   │
     │         V I D E O                   │  800px
     │         F R A M E                   │ (logical)
     │                                     │   │
     │                                     │   │
     │                                     │   │
     ╰╮                                  ╭╯   │
      ╰╮        ▶ advancement ▶        ╭╯    │
       ╰╮    ╭──────────────────╮     ╭╯      │
        ╰─╮  │  ● timeline bar │   ╭─╯       │
          ╰─╮╰──────────────────╯ ╭─╯         │
            ╰─╮                 ╭─╯            │
              ╰─────────────────╯              ─┴─
```

### 1.2 UI Element Placement

```
              ╭─────────────╮
            ╭─┘  ░░░░░░░░░  └─╮      ← Glossy highlight zone
          ╭─┘  ░░░░░░░░░░░░░  └─╮       (gradient overlay)
        ╭─┘░░░░░░░░░░░░░░░░░░░  └─╮
       ╭┘                           ╮
      ╭┘                             ╮
     ╭┘                               ╮
     │                                 │
     │                                 │   ← Video content area
     │          [VIDEO]                │     (fills entire oval)
     │                                 │
     │                                 │
     │                                 │
     ╰╮           advancement          ╭╯
      ╰╮     ◀◀   ▶║   ▶▶           ╭╯  ← Transport controls
       ╰╮  ╭──●──────────────╮     ╭╯      (appear on hover)
        ╰─╮│  0:42 / 3:21   │   ╭─╯    ← Timeline bar
          ╰╰────────────────── ╭─╯        (bottom region)
            ╰─╮             ╭─╯
              ╰─────────────╯
```

### 1.3 Effect Layering (Side View / Cross-Section)

```
    Layer Stack (front to back):
    ═══════════════════════════

    ┌─────────────────────────┐  Layer 4: Glossy Overlay
    │  ░░▓▓░░░░░░░░░░░░░░░  │  (semi-transparent gradient)
    │  ░░░░░░░░░░░░░░░░░░░  │  (simulates light reflection)
    │                         │
    └─────────────────────────┘

    ┌─────────────────────────┐  Layer 3: UI Controls
    │                         │  (transport, timeline)
    │       ▶  ──●────────   │  (appear on hover, fade)
    │         0:42 / 3:21    │
    └─────────────────────────┘

    ┌─────────────────────────┐  Layer 2: Video Frame
    │  ┌───────────────────┐  │  (decoded frame texture)
    │  │                   │  │  (scaled to fill oval)
    │  │   VIDEO CONTENT   │  │
    │  │                   │  │
    │  └───────────────────┘  │
    └─────────────────────────┘

    ┌─────────────────────────┐  Layer 1: Oval Mask
    │         ╭───╮           │  (alpha mask - ellipse)
    │       ╭─┘   └─╮        │  (everything outside = alpha 0)
    │      ╭┘       ╰╮       │
    │      │         │        │
    │      ╰╮       ╭╯       │
    │       ╰─╮   ╭─╯        │
    │         ╰───╯           │
    └─────────────────────────┘
```

### 1.4 Timeline Scrubber Detail

```
    Timeline bar (inside lower oval region):

    ╭──────────────────────────────────╮
    │  ◁◁   ▶║   ▷▷    0:42 / 3:21   │
    │                                  │
    │  ╶───────●━━━━━━━━━━━━━━━━━━━╸  │
    │          ↑                       │
    │     current position             │
    │                                  │
    │  ╶ = played portion (bright)     │
    │  ━ = remaining (dim)             │
    │  ● = scrub handle (draggable)    │
    ╰──────────────────────────────────╯

    During scrub (drag):
    ╭──────────────────────────────────╮
    │  ╶━━━━━━━━━━━━━━●━━━━━━━━━━━━╸  │
    │                  ↑               │
    │            ┌───────────┐         │
    │            │  PREVIEW  │         │  ← Thumbnail preview
    │            │  FRAME    │         │    (above scrub pos)
    │            │  1:23     │         │
    │            └───────────┘         │
    ╰──────────────────────────────────╯
```

### 1.5 Hover State Transitions

```
    STATE 1: Idle (no hover)        STATE 2: Hover (controls visible)
    ╭─────────────╮                 ╭─────────────╮
  ╭─┘ ░░░░░░░░░░░ └─╮            ╭─┘ ░░░░░░░░░░░ └─╮
╭─┘                   └─╮      ╭─┘                   └─╮
│                         │    │                         │
│                         │    │                         │
│       [VIDEO]           │    │       [VIDEO]           │
│       (clean,           │    │       (dimmed            │
│        no UI)           │    │        slightly)         │
│                         │    │                         │
│                         │    │    ◀◀   ▶║   ▶▶        │
╰╮                       ╭╯   ╰╮  ──●──────────       ╭╯
  ╰─╮                 ╭─╯       ╰─╮ 0:42/3:21      ╭─╯
    ╰─────────────────╯             ╰───────────────╯

    Transition: 300ms ease-in-out fade
```

---

## 2. Glossy Overlay Effect Design

### 2.1 The "Luminous Doorknob" Aesthetic

The visual identity calls for a "glossy metallic appearance like a luminous doorknob in white liminal space." This effect is achieved through a multi-layer shader approach:

#### Light Reflection Model

The glossy overlay simulates a curved, reflective surface (like glass or polished metal) over the video content:

```
    Top of oval:          Strong highlight (near-white, 40% opacity)
                          ↓ gradient
    Upper-middle:         Subtle highlight (white, 10% opacity)
                          ↓ gradient
    Center:               No overlay (0% opacity — video shows clearly)
                          ↓ gradient
    Lower portion:        Very subtle darkening (black, 5% opacity)
                          ↓
    Bottom edge:          Slight vignette (black, 15% opacity)
```

#### Specular Highlight Shape

The highlight is not a simple linear gradient — it follows a curved path simulating light reflecting off a convex surface:

```
              ╭─────────────╮
            ╭─┘ ●●●●●●●●●●● └─╮
          ╭─┘●●●●●●●●●●●●●●●●●└─╮    ← Primary specular
        ╭─┘  ●●●●●●●●●●●●●●●●●   └─╮    (elliptical, offset
       ╭┘      ●●●●●●●●●●●●●       ╮     from center)
      ╭┘          ●●●●●●●           ╮
     ╭┘              ●               ╮   ← Fades to nothing
     │                                │
     │                                │
     │                                │
     │                                │
     ╰╮                              ╭╯
      ╰╮          ◌◌◌◌◌◌           ╭╯   ← Secondary reflection
       ╰╮      ◌◌◌◌◌◌◌◌◌        ╭╯      (subtle, at bottom)
        ╰─╮                    ╭─╯
          ╰─────────────────╭─╯
            ╰─────────────╯
```

### 2.2 Fragment Shader Design (WGSL)

```wgsl
// Oval mask + glossy overlay fragment shader

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@group(0) @binding(0) var video_texture: texture_2d<f32>;
@group(0) @binding(1) var video_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let uv = in.uv;

    // --- Oval Mask ---
    let center = vec2f(0.5, 0.5);
    let d = (uv - center) / center;  // normalized to [-1, 1]
    let dist = length(d);
    let edge = fwidth(dist);
    let oval_alpha = 1.0 - smoothstep(1.0 - edge, 1.0, dist);

    if (oval_alpha < 0.01) {
        discard;  // Outside oval — fully transparent
    }

    // --- Video Frame ---
    let video_color = textureSample(video_texture, video_sampler, uv);

    // --- Glossy Overlay ---
    // Primary specular highlight (upper region)
    let highlight_center = vec2f(0.5, 0.25);  // Offset upward
    let highlight_d = (uv - highlight_center) / vec2f(0.4, 0.2);
    let highlight_dist = length(highlight_d);
    let highlight = exp(-highlight_dist * highlight_dist * 2.0);
    let highlight_color = vec3f(1.0, 1.0, 1.0) * highlight * 0.35;

    // Secondary reflection (bottom)
    let bottom_center = vec2f(0.5, 0.85);
    let bottom_d = (uv - bottom_center) / vec2f(0.3, 0.1);
    let bottom_dist = length(bottom_d);
    let bottom_highlight = exp(-bottom_dist * bottom_dist * 3.0);
    let bottom_color = vec3f(1.0, 1.0, 1.0) * bottom_highlight * 0.08;

    // Edge vignette
    let vignette = 1.0 - pow(dist, 3.0) * 0.3;

    // Composite
    var final_color = video_color.rgb * vignette;
    final_color = final_color + highlight_color + bottom_color;
    final_color = clamp(final_color, vec3f(0.0), vec3f(1.0));

    return vec4f(final_color, oval_alpha);
}
```

### 2.3 Dynamic Highlight (Optional Enhancement)

For a truly "luminous" effect, the specular highlight can respond to the mouse cursor position, simulating the user "holding" a reflective object:

```
Mouse at top-left:          Mouse at center:
  ●●●●                        ●●●●
●●●●●●●●                    ●●●●●●●●
  ●●●●●●                      ●●●●
     ●●                          ●

(highlight follows cursor with dampening)
```

This is achieved by passing the mouse position as a uniform to the shader and using it to offset the `highlight_center`.

---

## 3. Transport Controls Design

### 3.1 Control Layout

Controls appear on hover with a fade-in animation (300ms ease-in-out). They occupy the lower 25% of the oval.

```
    Control Region (lower oval):

    ╭──────────────────────────────────╮
    │                                  │
    │         ◀◀    ▶║    ▶▶          │   ← Play/Pause + Skip
    │                                  │
    │    ╶━━━━━━━━━━●━━━━━━━━━━━━╸    │   ← Timeline bar
    │                                  │
    │         0:42  /  3:21            │   ← Time display
    │                                  │
    ╰──────────────────────────────────╯
```

### 3.2 Interaction Zones

```
    ╭───────────────────────────────────────╮
    │                                       │
    │              ZONE A                   │
    │        (tap = toggle play/pause)      │
    │        (drag vertical = nothing)      │
    │                                       │
    │                                       │
    │                                       │
    ├───────────────────────────────────────┤  ← ~75% from top
    │              ZONE B                   │
    │     (tap = toggle play/pause)         │
    │     (hover = show controls)           │
    │     (drag horizontal = scrub)         │
    │                                       │
    ╰───────────────────────────────────────╯
```

### 3.3 Scrubbing Interaction

**Coarse scrub (during drag):**
1. User presses on timeline bar or drags horizontally anywhere in Zone B
2. Map horizontal position to timeline: `seek_pos = (mouse_x - oval_left) / oval_width`
3. Seek to nearest keyframe (fast, approximate)
4. Display keyframe with "preview" indicator

**Fine resolve (on release):**
1. User releases drag
2. Perform frame-accurate seek to exact `seek_pos`
3. Resume playback if was playing before scrub

**Drag-forward/reverse (anywhere on video):**
1. User presses and drags horizontally on the video area
2. Horizontal distance from press point maps to seek speed
3. Small drag = slow scrub, large drag = fast scrub
4. Release = stop at current frame

---

## 4. Color Palette and Typography

### 4.1 UI Color Scheme

```
    ┌─────────────────────────────────────────┐
    │  Element           │  Color             │
    ├─────────────────────────────────────────┤
    │  Background        │  Transparent       │
    │  Oval edge shadow  │  #000000 @ 30%     │
    │  Glossy highlight  │  #FFFFFF @ 35%     │
    │  Controls bg       │  #000000 @ 50%     │
    │  Timeline track    │  #FFFFFF @ 20%     │
    │  Timeline played   │  #FFFFFF @ 80%     │
    │  Scrub handle      │  #FFFFFF @ 100%    │
    │  Time text         │  #FFFFFF @ 70%     │
    │  Button icons      │  #FFFFFF @ 80%     │
    │  Button hover      │  #FFFFFF @ 100%    │
    └─────────────────────────────────────────┘
```

### 4.2 Typography

- **Time display:** Monospace font (SF Mono on macOS, Consolas on Windows)
- **Size:** 11px logical
- **Weight:** Regular for elapsed, Light for total duration
- **Format:** `MM:SS` for videos < 1hr, `H:MM:SS` for longer

---

## 5. Animation Specifications

### 5.1 Control Fade

```
    Hover Enter:
    t=0ms    opacity: 0.0
    t=150ms  opacity: 0.7
    t=300ms  opacity: 1.0
    easing: cubic-bezier(0.4, 0.0, 0.2, 1.0)

    Hover Exit (after 2s delay):
    t=0ms    opacity: 1.0
    t=200ms  opacity: 0.5
    t=400ms  opacity: 0.0
    easing: cubic-bezier(0.4, 0.0, 0.2, 1.0)
```

### 5.2 Play/Pause Icon Transition

```
    Play → Pause:
    ▶  →  ▶│  →  ▶║  →  ║  (morph over 200ms)

    Pause → Play:
    ║  →  ║▶  →  │▶  →  ▶  (morph over 200ms)
```

### 5.3 Scrub Handle

```
    Idle:     ●  (4px radius)
    Hover:    ◉  (6px radius, glow)
    Dragging: ◎  (8px radius, bright glow)
    Transition: 150ms ease-out
```

---

## 6. Video Scaling Within Oval

### 6.1 Aspect Ratio Handling

Videos will rarely match the oval's ~9:16 aspect ratio. Strategy:

```
    16:9 video in 9:16 oval:         9:16 video in 9:16 oval:

    ╭───────────╮                    ╭───────────╮
  ╭─┘           └─╮               ╭─┘ ░░░░░░░░░ └─╮
╭─┘                └─╮          ╭─┘ ░░░░░░░░░░░░░ └─╮
│   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓   │        │  ░░░░░░░░░░░░░░░░░  │
│   ▓▓▓▓VIDEO▓▓▓▓▓▓   │        │  ░░░░░░░░░░░░░░░░░  │
│   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓   │        │  ░░░░░VIDEO░░░░░░░  │
│   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓   │        │  ░░░░░░░░░░░░░░░░░  │
│                      │        │  ░░░░░░░░░░░░░░░░░  │
│   (letterboxed —     │        │  ░░░░░░░░░░░░░░░░░  │
│    black bars top/   │        │  (perfect fit)       │
╰╮   bottom)          ╭╯       ╰╮ ░░░░░░░░░░░░░░░░ ╭╯
  ╰─╮              ╭─╯           ╰─╮ ░░░░░░░░░░░ ╭─╯
    ╰──────────────╯                ╰─────────────╯
```

**Strategy:** Scale video to **cover** the oval (fill, crop edges) rather than **contain** (letterbox). Rationale: the oval already crops the corners, so maximizing video coverage creates a more immersive, "window into another world" effect. The user can toggle between fill and fit modes.

### 6.2 Video Texture UV Mapping

For "cover" mode, calculate the UV offset to center the video within the oval:

```wgsl
fn cover_uv(uv: vec2f, video_aspect: f32, oval_aspect: f32) -> vec2f {
    var scaled_uv = uv;
    if (video_aspect > oval_aspect) {
        // Video is wider — crop left/right
        let scale = oval_aspect / video_aspect;
        scaled_uv.x = uv.x * scale + (1.0 - scale) * 0.5;
    } else {
        // Video is taller — crop top/bottom
        let scale = video_aspect / oval_aspect;
        scaled_uv.y = uv.y * scale + (1.0 - scale) * 0.5;
    }
    return scaled_uv;
}
```

---

## 7. Idle / No-Video State

When no video is loaded, display the "luminous doorknob" in its pure form:

```
              ╭─────────────╮
            ╭─┘ ●●●●●●●●●●● └─╮
          ╭─┘●●●●●●●●●●●●●●●●●└─╮
        ╭─┘  ●●●●●●●●●●●●●●●●●   └─╮    ← Animated specular
       ╭┘      ●●●●●●●●●●●●●       ╮
      ╭┘          ●●●●●●●           ╮
     ╭┘              ●               ╮
     │                                │
     │     ╭──────────────────╮      │
     │     │    DROP VIDEO    │      │    ← Drag-and-drop target
     │     │    HERE          │      │
     │     ╰──────────────────╯      │
     │                                │
     ╰╮                              ╭╯
      ╰╮          ◌◌◌◌◌◌           ╭╯
       ╰╮      ◌◌◌◌◌◌◌◌◌        ╭╯
        ╰─╮                    ╭─╯
          ╰─────────────────╭─╯

    Background: Dark metallic gradient (#1a1a2e → #16213e)
    Specular highlight: Slowly orbiting/pulsing
    Border: Subtle metallic rim light
```

---

## 8. Rendering Pipeline Order

For each frame rendered:

```
1. Clear framebuffer to transparent (alpha = 0)

2. Calculate oval mask (ellipse equation)

3. Sample video texture
   → Apply aspect-ratio-correct UV mapping
   → Scale to cover oval

4. Apply vignette darkening at oval edges

5. Composite glossy overlay
   → Primary specular highlight (upper region)
   → Secondary reflection (lower region)

6. If controls visible:
   → Draw semi-transparent control backdrop
   → Draw timeline bar
   → Draw scrub handle
   → Draw transport icons
   → Draw time text

7. Apply oval alpha mask to final output

8. Present to surface
```
