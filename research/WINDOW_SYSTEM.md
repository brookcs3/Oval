# Window System Research for Oval Player

## Executive Summary

Creating a transparent, oval-shaped window requires platform-specific techniques on both macOS and Windows. This document details the approach for each platform and the cross-platform abstraction strategy using Rust's `winit` crate with platform-specific extensions.

**Key finding:** True non-rectangular windows are achieved by creating a transparent, borderless window and masking the content to an oval shape. On macOS this is well-supported through `NSWindow` + `CALayer` masking. On Windows this requires `WS_EX_LAYERED` with per-pixel alpha via `UpdateLayeredWindow` or a DWM composition approach. Both platforms require custom hit-testing for proper mouse interaction.

---

## 1. macOS: NSWindow + Core Animation

### Technique Overview

macOS provides first-class support for custom-shaped windows through its compositing window manager (Quartz Compositor). The approach:

1. Create a borderless `NSWindow` with `NSBorderlessWindowMask`
2. Set `isOpaque = false` and `backgroundColor = .clear`
3. Draw oval content in a custom `NSView` or apply a `CAShapeLayer` mask
4. The window manager automatically provides a shadow matching the drawn shape

### Implementation Details

#### Step 1: Window Configuration

```swift
// Conceptual — actual implementation in Rust via objc2 crate
class OvalWindow: NSWindow {
    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 450, height: 800),
            styleMask: .borderless,       // No title bar, no chrome
            backing: .buffered,
            defer: false
        )
        self.isOpaque = false             // Enable transparency
        self.backgroundColor = .clear     // Transparent background
        self.hasShadow = true             // OS-level shadow follows shape
        self.level = .floating            // Float above normal windows
        self.isMovableByWindowBackground = true  // Drag anywhere
    }

    override var canBecomeKey: Bool { true }   // Accept keyboard input
    override var canBecomeMain: Bool { true }  // Can be main window
}
```

#### Step 2: Oval Masking via CAShapeLayer

Two approaches for masking:

**Approach A: NSView draw override**
```swift
class OvalContentView: NSView {
    override func draw(_ dirtyRect: NSRect) {
        NSColor.clear.set()
        dirtyRect.fill()  // Clear everything

        let ovalPath = NSBezierPath(ovalIn: bounds)
        ovalPath.addClip()  // Clip all subsequent drawing to oval

        // Draw video frame here (fills only the oval)
        drawVideoFrame(in: bounds)
    }
}
```

**Approach B: CALayer mask (Preferred)**
```swift
let maskLayer = CAShapeLayer()
maskLayer.path = CGPath(ellipseIn: view.bounds, transform: nil)
view.layer?.mask = maskLayer
```

Approach B is preferred because:
- GPU-accelerated masking (no CPU-side clipping)
- Works with any content rendered to the layer (including wgpu output)
- Automatically anti-aliases the oval edge

#### Step 3: Integrating with wgpu

The critical challenge is connecting wgpu's Metal rendering to an `NSView` that has an oval mask:

1. `winit` creates the window and provides a `RawWindowHandle`
2. wgpu creates a `Surface` from the `RawWindowHandle` (uses Metal on macOS)
3. wgpu renders to the surface's `CAMetalLayer`
4. Apply `CAShapeLayer` mask to the `CAMetalLayer` or its parent layer

```
┌─────────────────────────────────────┐
│           NSWindow (borderless,     │
│           transparent)              │
│  ┌───────────────────────────────┐  │
│  │       NSView (contentView)    │  │
│  │  ┌─────────────────────────┐  │  │
│  │  │    CAMetalLayer         │  │  │
│  │  │    (wgpu surface)       │  │  │
│  │  │                         │  │  │
│  │  │   mask = CAShapeLayer   │  │  │
│  │  │   (ellipse path)        │  │  │
│  │  └─────────────────────────┘  │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

#### Step 4: Hit-Testing

With a transparent borderless window, macOS automatically passes clicks through fully transparent areas. Since the oval mask makes areas outside the oval transparent, click-through works automatically. However, for precise control:

```swift
override func hitTest(_ point: NSPoint) -> NSView? {
    let ovalPath = NSBezierPath(ovalIn: bounds)
    if ovalPath.contains(point) {
        return super.hitTest(point)
    }
    return nil  // Click passes through to window behind
}
```

#### Step 5: Window Dragging

With `.borderless` style, there's no title bar to drag. Options:

- `isMovableByWindowBackground = true` — entire window background becomes draggable
- Custom drag handling via `mouseDown`/`mouseDragged` for specific drag regions (e.g., only the top portion of the oval)

### Rust Implementation via `objc2`

From Rust, macOS APIs are accessed through the `objc2` crate (safe Objective-C bindings):

```rust
use objc2_app_kit::{NSWindow, NSWindowStyleMask, NSBackingStoreType};
use objc2_foundation::NSRect;

// Configure the winit-created window for oval mode
unsafe {
    let ns_window: &NSWindow = /* get from winit raw handle */;
    ns_window.setOpaque(false);
    ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
    ns_window.setHasShadow(true);
    ns_window.setStyleMask(NSWindowStyleMask::Borderless);

    // Apply oval mask to the content view's layer
    let content_view = ns_window.contentView().unwrap();
    content_view.setWantsLayer(true);
    let layer = content_view.layer().unwrap();
    // Create and apply CAShapeLayer mask...
}
```

---

## 2. Windows: Layered Windows + Per-Pixel Alpha

### Technique Overview

Windows does not have the same elegant compositing story as macOS. There are three approaches, each with trade-offs:

### Approach A: Window Regions (`SetWindowRgn`)

Create an elliptical region and assign it to the window:

```cpp
HRGN hRgn = CreateEllipticRgn(0, 0, width, height);
SetWindowRgn(hwnd, hRgn, TRUE);
```

**Pros:** Simple, proper hit-testing automatically follows the region shape.
**Cons:** No anti-aliasing on edges — the oval will have jagged, pixelated edges. This is unacceptable for the "glossy, futuristic" aesthetic.

### Approach B: Layered Window with Color Key

```cpp
SetWindowLong(hwnd, GWL_EXSTYLE,
    GetWindowLong(hwnd, GWL_EXSTYLE) | WS_EX_LAYERED);
SetLayeredWindowAttributes(hwnd, RGB(255,0,255), 0, LWA_COLORKEY);
// Paint everything outside the oval with the color key (magenta)
```

**Pros:** Better than regions, areas painted with the key color become transparent.
**Cons:** No per-pixel alpha (edges are either fully transparent or fully opaque — still jagged). Color key can conflict with actual content colors.

### Approach C: Per-Pixel Alpha via `UpdateLayeredWindow` (Recommended)

This provides the highest quality with smooth, anti-aliased edges:

```cpp
// Create a 32-bit ARGB DIB section
BITMAPINFO bmi = {};
bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
bmi.bmiHeader.biWidth = width;
bmi.bmiHeader.biHeight = -height;  // Top-down
bmi.bmiHeader.biPlanes = 1;
bmi.bmiHeader.biBitCount = 32;     // ARGB
bmi.bmiHeader.biCompression = BI_RGB;

HBITMAP hBitmap = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &bits, NULL, 0);

// Draw oval with anti-aliased edges into the bitmap
// Set alpha channel: 255 inside oval, 0 outside, gradient at edges

// Update the layered window
BLENDFUNCTION blend = {};
blend.BlendOp = AC_SRC_OVER;
blend.SourceConstantAlpha = 255;
blend.AlphaFormat = AC_SRC_ALPHA;  // Per-pixel alpha

UpdateLayeredWindow(hwnd, hdcScreen, &ptDst, &szWnd,
                    hdcMem, &ptSrc, 0, &blend, ULW_ALPHA);
```

**Pros:** Smooth anti-aliased oval edges, true per-pixel transparency, professional appearance.
**Cons:** More complex; requires compositing the rendered frame into the ARGB bitmap before calling `UpdateLayeredWindow`.

### Approach D: DWM Composition (Modern Windows 10/11)

Use DWM (Desktop Window Manager) APIs for compositor-level transparency:

```cpp
// Enable blur-behind with a custom region (creates transparent areas)
DWM_BLURBEHIND bb = {};
bb.dwFlags = DWM_BB_ENABLE | DWM_BB_BLURREGION;
bb.fEnable = TRUE;
bb.hRgnBlur = CreateEllipticRgn(0, 0, width, height);
DwmEnableBlurBehindWindow(hwnd, &bb);

// Extend client area over entire window
MARGINS margins = {-1, -1, -1, -1};
DwmExtendFrameIntoClientArea(hwnd, &margins);
```

**Pros:** Leverages OS compositor, potentially better performance. Works with Direct3D rendering.
**Cons:** Behavior varies between Windows 10 and 11. Less control over edge anti-aliasing.

### Recommended Windows Approach

**Use Approach C (Per-Pixel Alpha) with wgpu DX12 backend:**

1. wgpu renders the oval scene (video + effects) to an offscreen texture
2. Read back the rendered frame to CPU memory
3. Composite into a 32-bit ARGB bitmap with the oval alpha mask
4. Call `UpdateLayeredWindow` to display

Alternatively, if wgpu can render directly to the window's surface with proper alpha:

1. Create a borderless, transparent window using `WS_EX_LAYERED`
2. wgpu renders to the window surface (DX12 swap chain)
3. Apply oval alpha mask in the fragment shader (alpha = 0 outside oval)
4. DWM composites the result with per-pixel alpha

The shader-based approach (option 2) avoids the CPU readback and is preferred for performance.

### Hit-Testing on Windows

With `WS_EX_LAYERED`, Windows still sends mouse events to transparent areas. Fix this with `WM_NCHITTEST`:

```cpp
case WM_NCHITTEST: {
    POINT pt = { LOWORD(lParam), HIWORD(lParam) };
    ScreenToClient(hwnd, &pt);

    // Check if point is inside the oval
    float cx = width / 2.0f, cy = height / 2.0f;
    float dx = (pt.x - cx) / cx;
    float dy = (pt.y - cy) / cy;
    if (dx*dx + dy*dy <= 1.0f) {
        return HTCLIENT;  // Inside oval — handle the click
    }
    return HTTRANSPARENT;  // Outside oval — pass through
}
```

### Window Dragging on Windows

Without a title bar, handle window dragging by returning `HTCAPTION` from `WM_NCHITTEST` for the drag region:

```cpp
// In WM_NCHITTEST handler, for points in upper portion of oval:
if (dy < -0.3f && dx*dx + dy*dy <= 1.0f) {
    return HTCAPTION;  // Upper region = draggable
}
```

---

## 3. Cross-Platform Abstraction

### winit as the Foundation

`winit` provides cross-platform window creation. The Oval-specific configuration happens through platform extensions:

```rust
use winit::window::WindowBuilder;
use winit::dpi::LogicalSize;

let window = WindowBuilder::new()
    .with_title("Oval")
    .with_inner_size(LogicalSize::new(450.0, 800.0))
    .with_decorations(false)         // No title bar
    .with_transparent(true)          // Transparent background
    .with_resizable(false)           // Fixed oval size
    .build(&event_loop)?;
```

### Platform-Specific Layer

After `winit` creates the window, apply platform-specific oval configuration:

```rust
#[cfg(target_os = "macos")]
fn configure_oval_window(window: &winit::window::Window) {
    // Access NSWindow via raw handle
    // Set isOpaque=false, backgroundColor=clear
    // Apply CAShapeLayer oval mask
}

#[cfg(target_os = "windows")]
fn configure_oval_window(window: &winit::window::Window) {
    // Access HWND via raw handle
    // Set WS_EX_LAYERED
    // Configure per-pixel alpha
    // Set up WM_NCHITTEST handler
}
```

### Known winit Issues

**Transparency + decorations interaction on Windows:** There is a known issue (winit #851) where combining `with_transparent(true)` and `with_decorations(false)` on Windows 10 can produce an invisible window. The workaround is to remove the `WS_EX_LAYERED` flag that winit adds automatically and manage layered window behavior manually.

**wgpu + transparency on macOS:** There have been reports (egui #2680) that `eframe::NativeOptions::transparent` doesn't work with the wgpu renderer on Mac. The workaround is to directly manipulate the `CAMetalLayer`'s `isOpaque` property via `objc2`.

---

## 4. Oval Geometry

### Dimensions

Target: ~9:16 portrait aspect ratio, similar to TikTok/Stories format.

Recommended default window size: **450 × 800 pixels** (logical)

This gives an aspect ratio of 0.5625 (9:16), fitting comfortably on most displays while being large enough for video enjoyment.

### Oval Equation

For hit-testing and masking, the oval is defined by the ellipse equation:

```
(x - cx)²     (y - cy)²
─────────  +  ─────────  ≤  1
   rx²           ry²

where:
  cx = width / 2  = 225
  cy = height / 2 = 400
  rx = width / 2  = 225
  ry = height / 2 = 400
```

In normalized coordinates (0.0 to 1.0):
```
((u - 0.5) / 0.5)² + ((v - 0.5) / 0.5)² ≤ 1.0

Simplified: (2u - 1)² + (2v - 1)² ≤ 1.0
```

This formula is used in:
- Fragment shader (oval mask + anti-aliasing)
- Hit-test handler (mouse click routing)
- Shadow generation

### Anti-Aliased Edge

For smooth edges in the shader:

```wgsl
let dist = length(vec2f(2.0 * uv.x - 1.0, 2.0 * uv.y - 1.0));
let edge_width = fwidth(dist);  // Automatic anti-aliasing width
let alpha = 1.0 - smoothstep(1.0 - edge_width, 1.0 + edge_width, dist);
```

This produces a smooth, anti-aliased oval boundary regardless of screen resolution and DPI.

---

## 5. DPI / Retina Handling

### macOS Retina

macOS Retina displays use 2x (or 3x on some monitors) scaling. `winit` reports both logical size (what the user sees) and physical size (actual pixels):

- Logical size: 450 × 800
- Physical size: 900 × 1600 (at 2x)

wgpu's surface should be configured with the physical size. The oval mask and video content should render at full physical resolution for crisp edges on Retina displays.

### Windows DPI Scaling

Windows uses per-monitor DPI awareness. Declare the app as DPI-aware:

```rust
// In Cargo.toml manifest or via API
SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
```

`winit` handles DPI scaling through its `ScaleFactor` API. Use logical coordinates for UI layout and physical coordinates for rendering.

---

## 6. Architecture Summary

```
┌─────────────────────────────────────────────────┐
│                   winit                         │
│         (cross-platform window creation)        │
│                                                 │
│  ┌───────────────┐      ┌───────────────────┐  │
│  │ macOS Backend │      │  Windows Backend  │  │
│  │               │      │                   │  │
│  │ NSWindow      │      │ HWND              │  │
│  │ .borderless   │      │ WS_EX_LAYERED     │  │
│  │ .clear bg     │      │ Per-pixel alpha    │  │
│  │ CAShapeLayer  │      │ WM_NCHITTEST      │  │
│  │ oval mask     │      │ oval hit-test     │  │
│  └───────────────┘      └───────────────────┘  │
│                                                 │
│            ┌─────────────────┐                  │
│            │     wgpu        │                  │
│            │  Metal / DX12   │                  │
│            │  Render oval    │                  │
│            │  content with   │                  │
│            │  alpha mask     │                  │
│            └─────────────────┘                  │
└─────────────────────────────────────────────────┘
```
