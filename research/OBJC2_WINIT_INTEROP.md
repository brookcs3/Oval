# objc2 + winit + wgpu: macOS Transparent Window Interop Research

## Summary

This document covers the concrete Rust code required to extract an `NSWindow` from winit's window handle, configure it for transparency, set `CAMetalLayer.isOpaque = false` for wgpu's Metal backend, and handle window dragging on borderless windows. All code targets **winit 0.30+**, **wgpu 28+**, and **objc2 0.5+**.

---

## 1. Extracting NSWindow from winit's RawWindowHandle

### The Mechanism

winit 0.30+ implements the `HasWindowHandle` trait (from the `raw-window-handle` 0.6 crate). On macOS, calling `window.window_handle()` returns a `RawWindowHandle::AppKit(AppKitWindowHandle)`, which contains a `NonNull<c_void>` pointer to the `NSView` (not the `NSWindow`). You get the `NSWindow` by calling `.window()` on that view.

### Required Crates and Features

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = { version = "0.2", features = ["NSGeometry"] }
objc2-app-kit = { version = "0.2", features = [
    "NSWindow",
    "NSColor",
    "NSView",
    "NSResponder",
] }
objc2-quartz-core = { version = "0.2", features = [
    "CALayer",
    "CAMetalLayer",
] }
```

**Version compatibility note:** objc2 0.5.x uses `Retained<T>` (formerly `Id<T>` in 0.4.x). The `objc2-app-kit` and `objc2-foundation` crates at 0.2.x are designed for objc2 0.5.x. If you see `Id<T>` in older examples, substitute `Retained<T>`.

### Code: Extracting NSWindow

```rust
#[cfg(target_os = "macos")]
mod macos {
    use raw_window_handle::HasWindowHandle;
    use objc2::rc::Retained;
    use objc2_app_kit::{NSColor, NSView, NSWindow};
    use objc2_quartz_core::CALayer;

    /// Extract the NSWindow reference from a winit Window.
    ///
    /// # Safety
    /// The winit window must be valid and not yet dropped.
    /// Must be called on the main thread (macOS requirement for UI operations).
    pub unsafe fn get_ns_window(
        window: &winit::window::Window,
    ) -> Retained<NSWindow> {
        let handle = window
            .window_handle()
            .expect("failed to get window handle");

        let raw = handle.as_raw();
        let raw_window_handle::RawWindowHandle::AppKit(appkit_handle) = raw else {
            panic!("expected AppKit window handle on macOS");
        };

        // The AppKitWindowHandle contains a pointer to the NSView
        let ns_view: &NSView = unsafe {
            appkit_handle
                .ns_view
                .cast::<NSView>()
                .as_ref()
        };

        // Get the NSWindow from the NSView
        let ns_window: Retained<NSWindow> = ns_view
            .window()
            .expect("NSView has no parent NSWindow");

        ns_window
    }
}
```

### Alternative: Using the `as_raw()` pattern (winit 0.30 specifics)

In winit 0.30+, `window.window_handle()` returns a `Result<WindowHandle<'_>, HandleError>`. The `WindowHandle` wraps a `RawWindowHandle`. The key type chain is:

```
winit::window::Window
  -> .window_handle() -> Result<WindowHandle<'_>, HandleError>
    -> .as_raw() -> RawWindowHandle
      -> RawWindowHandle::AppKit(AppKitWindowHandle)
        -> .ns_view -> NonNull<c_void>  // pointer to NSView
```

The `ns_view` field is a `NonNull<c_void>`. You cast it to `&NSView` using objc2's type system. Then call `.window()` to get the parent `NSWindow`.

---

## 2. Setting NSWindow Transparency Properties via objc2

### The Three Essential Properties

For a transparent borderless window on macOS, you must set these on the `NSWindow`:

1. `setOpaque(false)` -- tells the window server this window has transparency
2. `setBackgroundColor(Some(&NSColor::clearColor()))` -- makes the window background fully transparent
3. `setHasShadow(true)` -- enables the OS-generated shadow that follows the rendered content shape

### Code: Configuring NSWindow for Transparency

```rust
#[cfg(target_os = "macos")]
pub unsafe fn configure_transparent_window(ns_window: &NSWindow) {
    use objc2_app_kit::NSColor;

    // 1. Window is not opaque (has transparent pixels)
    ns_window.setOpaque(false);

    // 2. Background is fully transparent
    //    clearColor() returns Retained<NSColor>, we pass a reference
    let clear = unsafe { NSColor::clearColor() };
    ns_window.setBackgroundColor(Some(&clear));

    // 3. Enable shadow -- macOS will auto-generate a shadow that
    //    follows the shape of the non-transparent content.
    //    This is what gives the oval its "floating object" look.
    ns_window.setHasShadow(true);

    // 4. (Optional) Make the window floating above normal windows
    //    NSWindowLevel::Floating = 3
    //    Uncomment if desired for "always on top" behavior:
    // use objc2_app_kit::NSWindowLevel;
    // ns_window.setLevel(NSWindowLevel::Floating);
}
```

### objc2 Method Call Syntax Notes

objc2 0.5+ generates Rust methods directly on the Objective-C class types. The mapping is:

| Objective-C | objc2 Rust |
|---|---|
| `[window setOpaque:NO]` | `ns_window.setOpaque(false)` |
| `[window setBackgroundColor:[NSColor clearColor]]` | `ns_window.setBackgroundColor(Some(&NSColor::clearColor()))` |
| `[window setHasShadow:YES]` | `ns_window.setHasShadow(true)` |
| `[window contentView]` | `ns_window.contentView()` -- returns `Option<Retained<NSView>>` |
| `[view setWantsLayer:YES]` | `ns_view.setWantsLayer(true)` |
| `[view layer]` | `ns_view.layer()` -- returns `Option<Retained<CALayer>>` |

The `NSColor::clearColor()` method is a class method (Objective-C `+[NSColor clearColor]`). In objc2-app-kit 0.2, it is exposed as an `unsafe` associated function that returns `Retained<NSColor>`.

### Content View Layer Setup

For the CAShapeLayer mask to work (and for wgpu to composite with transparency), the content view must be layer-backed:

```rust
#[cfg(target_os = "macos")]
pub unsafe fn setup_content_view_layer(ns_window: &NSWindow) {
    use objc2_app_kit::NSView;

    let content_view: Retained<NSView> = ns_window
        .contentView()
        .expect("window has no content view");

    // Ensure the view is layer-backed (required for CALayer operations)
    content_view.setWantsLayer(true);
}
```

---

## 3. Setting CAMetalLayer.isOpaque = false

### Why This Is Required

winit's `with_transparent(true)` and the NSWindow configuration above are necessary but may not be sufficient. When wgpu creates a Metal surface, it creates (or uses) a `CAMetalLayer` as the backing layer for the content view. By default, `CAMetalLayer.isOpaque` is `true`, which tells the compositor to ignore alpha values and treat the layer as fully opaque. You must set it to `false` for alpha compositing to actually work.

This is the root cause of the commonly reported issue: "I set transparent(true) on winit but my window background is still black/opaque."

### Code: Setting CAMetalLayer.isOpaque = false

```rust
#[cfg(target_os = "macos")]
pub unsafe fn configure_metal_layer_transparency(ns_window: &NSWindow) {
    use objc2_app_kit::NSView;
    use objc2_quartz_core::CALayer;
    use objc2::runtime::AnyObject;
    use objc2::msg_send;

    let content_view: Retained<NSView> = ns_window
        .contentView()
        .expect("window has no content view");

    content_view.setWantsLayer(true);

    if let Some(layer) = content_view.layer() {
        // CALayer.setOpaque(false) -- tells compositor to respect alpha
        layer.setOpaque(false);

        // If the layer is specifically a CAMetalLayer (which it will be
        // after wgpu creates a surface), we also need to set isOpaque
        // on the CAMetalLayer itself.
        //
        // Since CAMetalLayer inherits from CALayer, setOpaque(false)
        // on the CALayer reference should suffice. But to be safe,
        // we can also access it as a CAMetalLayer:
        //
        // Note: objc2-quartz-core's CAMetalLayer type may require
        // feature "CAMetalLayer" in objc2-quartz-core.
    }
}
```

### More Direct Approach via objc2 msg_send (if CAMetalLayer typing is awkward)

If the `CAMetalLayer` type isn't directly available or the inheritance chain is tricky, you can use raw message sending:

```rust
#[cfg(target_os = "macos")]
pub unsafe fn set_metal_layer_opaque_false(ns_window: &NSWindow) {
    use objc2_app_kit::NSView;
    use objc2::msg_send;

    let content_view = ns_window.contentView().unwrap();
    content_view.setWantsLayer(true);

    if let Some(layer) = content_view.layer() {
        // setOpaque: is defined on CALayer, which CAMetalLayer inherits
        layer.setOpaque(false);
    }
}
```

### Timing: When to Call This

The `CAMetalLayer` is created by wgpu when you call `instance.create_surface(&window)`. The layer might not exist before that point. The recommended sequence is:

```
1. winit creates window (with_transparent(true), with_decorations(false))
2. Configure NSWindow (setOpaque, setBackgroundColor, setHasShadow)
3. wgpu creates surface (this creates or adopts the CAMetalLayer)
4. Set CAMetalLayer.isOpaque = false  <-- AFTER surface creation
5. Configure wgpu surface with CompositeAlphaMode::PreMultiplied
```

### CompositeAlphaMode (Critical Companion Setting)

Setting `CAMetalLayer.isOpaque = false` is only half the story. The wgpu `SurfaceConfiguration` must also specify the correct alpha mode:

```rust
let caps = surface.get_capabilities(&adapter);

// Find the best alpha mode for transparency
let alpha_mode = if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
    wgpu::CompositeAlphaMode::PreMultiplied
} else if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PostMultiplied) {
    wgpu::CompositeAlphaMode::PostMultiplied
} else {
    // Fallback -- transparency may not work
    wgpu::CompositeAlphaMode::Auto
};

let config = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format: surface_format,
    width: size.width,
    height: size.height,
    present_mode: wgpu::PresentMode::Fifo,
    alpha_mode,  // <-- THIS IS CRITICAL
    view_formats: vec![],
    desired_maximum_frame_latency: 2,
};
surface.configure(&device, &config);
```

**On macOS Metal:** `PreMultiplied` is typically supported. This means your fragment shader must output pre-multiplied alpha:

```wgsl
// Pre-multiplied alpha: multiply RGB by alpha before output
return vec4f(final_color * oval_alpha, oval_alpha);
```

Not just:
```wgsl
// This is WRONG for PreMultiplied mode -- will produce bright fringes
return vec4f(final_color, oval_alpha);
```

---

## 4. Is winit's `with_transparent(true)` Sufficient Alone?

### Short Answer: No. It is necessary but not sufficient.

### What `with_transparent(true)` Does on macOS

When you call `Window::default_attributes().with_transparent(true)` on winit 0.30+, winit internally:

1. Sets `NSWindow.isOpaque = false`
2. Sets `NSWindow.backgroundColor = NSColor.clearColor()`
3. May set the window's `hasShadow` to `false` (varies by winit version)

So winit handles **part** of the NSWindow configuration. However, it does NOT:

- Set `CAMetalLayer.isOpaque = false` (this is the wgpu/Metal layer, not winit's domain)
- Configure `CompositeAlphaMode` on the wgpu surface (this is entirely your responsibility)
- Set `hasShadow = true` (winit may disable it; you want it for the floating object look)

### What You Still Must Do Manually

Even with `with_transparent(true)`:

| Step | Who Handles It | Required? |
|---|---|---|
| `NSWindow.setOpaque(false)` | winit (via `with_transparent(true)`) | Done by winit |
| `NSWindow.setBackgroundColor(clear)` | winit (via `with_transparent(true)`) | Done by winit |
| `NSWindow.setHasShadow(true)` | **You** (via objc2) | Yes -- winit may turn it off |
| `CAMetalLayer.setOpaque(false)` | **You** (via objc2 after surface creation) | Yes -- critical |
| `CompositeAlphaMode::PreMultiplied` | **You** (wgpu surface config) | Yes -- critical |
| Pre-multiplied alpha in shader | **You** (WGSL fragment shader) | Yes -- prevents fringing |

### Known Issues and Reports

- **egui #2680** -- `eframe::NativeOptions::transparent` doesn't work with wgpu renderer on macOS. Root cause: CAMetalLayer.isOpaque not being set. Workaround: direct objc2 manipulation, which is exactly what we're doing.

- **winit transparency behavior** -- winit's `with_transparent(true)` is a "hint" that the window will have transparent content. It sets up the NSWindow properties, but the rendering backend (wgpu) must also cooperate. The two-layer problem (NSWindow transparency + CAMetalLayer transparency) trips up many developers.

- **wgpu CompositeAlphaMode::Auto pitfall** -- `Auto` selects `Opaque` on most platforms if that's listed first in capabilities. You must explicitly query capabilities and select `PreMultiplied`.

### Recommendation

Use a belt-and-suspenders approach -- set `with_transparent(true)` on winit AND configure everything via objc2 afterward. This is defensive and handles any edge cases where winit's behavior changes between versions:

```rust
// Belt: winit hints
let attrs = Window::default_attributes()
    .with_transparent(true)
    .with_decorations(false)
    .with_inner_size(LogicalSize::new(450.0, 800.0))
    .with_resizable(false);

// macOS-specific winit extensions
#[cfg(target_os = "macos")]
let attrs = {
    use winit::platform::macos::WindowAttributesExtMacOS;
    attrs
        .with_movable_by_window_background(true)
        .with_has_shadow(true)
};

let window = event_loop.create_window(attrs)?;

// Suspenders: direct objc2 configuration
#[cfg(target_os = "macos")]
unsafe {
    let ns_window = macos::get_ns_window(&window);
    macos::configure_transparent_window(&ns_window);
}

// ... create wgpu surface ...

// After wgpu surface creation:
#[cfg(target_os = "macos")]
unsafe {
    let ns_window = macos::get_ns_window(&window);
    macos::set_metal_layer_opaque_false(&ns_window);
}

// Configure surface with PreMultiplied alpha mode
```

---

## 5. `drag_window()` on Borderless Transparent Windows

### Does `Window::drag_window()` Work?

**Yes, `drag_window()` works on borderless windows on macOS.** winit's `drag_window()` method (added in winit 0.29, available in 0.30+) initiates an OS-level window drag operation. On macOS, it calls `[NSWindow performWindowDragWithEvent:]` under the hood. This works regardless of window style mask (borderless, transparent, etc.) because it operates at the NSWindow level, not the title bar level.

### How to Use It

You call `window.drag_window()` in response to a mouse-down event:

```rust
fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
) {
    match event {
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => {
            // Check if the click is inside the oval and in a "drag zone"
            // (e.g., not on a UI control)
            if self.hit_test_oval(self.cursor_pos) && !self.hit_test_controls(self.cursor_pos) {
                if let Some(window) = &self.window {
                    let _ = window.drag_window();
                }
            }
        }
        WindowEvent::CursorMoved { position, .. } => {
            self.cursor_pos = position;
        }
        _ => {}
    }
}
```

### Caveats and Edge Cases

1. **Must be called during a mouse-down event.** `drag_window()` requires an active mouse-down event to initiate the drag. Calling it at other times is a no-op. On macOS, it internally needs the current `NSEvent` to pass to `performWindowDragWithEvent:`.

2. **winit event flow during drag.** Once `drag_window()` is called, macOS takes over the drag loop. winit will NOT receive `CursorMoved` events during the drag -- the OS handles the window positioning directly. You will receive a `WindowEvent::Moved` when the drag ends (or periodically during the drag on some versions).

3. **Return value.** `drag_window()` returns `Result<(), ExternalError>`. It can fail if there's no active mouse event, but this is rare in practice.

### Alternative: `with_movable_by_window_background(true)`

winit provides a macOS-specific attribute `WindowAttributesExtMacOS::with_movable_by_window_background(true)`. This sets `NSWindow.isMovableByWindowBackground = true`, which makes the entire window background area draggable automatically.

```rust
#[cfg(target_os = "macos")]
let attrs = {
    use winit::platform::macos::WindowAttributesExtMacOS;
    attrs.with_movable_by_window_background(true)
};
```

**Pros:**
- Zero code required -- the OS handles everything
- Works automatically with transparent windows (only non-transparent areas are draggable)
- No hit-testing needed for basic drag

**Cons:**
- You can't distinguish between "drag the window" and "click to interact" -- every mouse-down on a non-transparent area starts a drag
- Can conflict with video interaction (click-to-pause, scrubbing)
- Less control over which regions are draggable

### Recommended Approach for Oval

Use `drag_window()` with manual hit-testing, NOT `with_movable_by_window_background(true)`. Here's why:

The oval has multiple interaction zones:
- Upper ~75% = drag zone (click starts window drag, double-click toggles play/pause)
- Lower ~25% = control zone (click interacts with timeline/transport controls)

With `with_movable_by_window_background(true)`, you can't prevent dragging when the user clicks on the timeline. With `drag_window()`, you control exactly when dragging starts:

```rust
WindowEvent::MouseInput {
    state: ElementState::Pressed,
    button: MouseButton::Left,
    ..
} => {
    let pos = self.cursor_pos;

    if !self.is_inside_oval(pos) {
        // Outside oval -- do nothing (click passes through)
        return;
    }

    if self.is_in_control_zone(pos) && self.controls_visible {
        // Over timeline/transport -- handle as UI interaction
        self.start_scrub(pos);
    } else {
        // In the video/drag zone -- start window drag
        if let Some(window) = &self.window {
            let _ = window.drag_window();
        }
    }
}
```

### Fallback: Manual Position Tracking

If `drag_window()` ever proves unreliable (it shouldn't on macOS, but just in case), the manual approach is:

```rust
// On mouse down:
self.dragging = true;
self.drag_start_cursor = cursor_screen_position;
self.drag_start_window = window.outer_position().unwrap();

// On cursor moved (if dragging):
if self.dragging {
    let delta = current_screen_cursor - self.drag_start_cursor;
    let new_pos = self.drag_start_window + delta;
    window.set_outer_position(new_pos);
}

// On mouse up:
self.dragging = false;
```

This is more work and less smooth than `drag_window()`, but it's fully cross-platform and doesn't depend on OS drag APIs.

---

## 6. Complete Integration Example

Here's how all the pieces fit together in a Sprint 1 skeleton:

```rust
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct OvalApp {
    window: Option<Window>,
    // wgpu state would go here
    cursor_pos: winit::dpi::PhysicalPosition<f64>,
}

impl OvalApp {
    fn new() -> Self {
        Self {
            window: None,
            cursor_pos: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        }
    }

    fn is_inside_oval(&self, pos: winit::dpi::PhysicalPosition<f64>) -> bool {
        if let Some(window) = &self.window {
            let size = window.inner_size();
            let cx = size.width as f64 / 2.0;
            let cy = size.height as f64 / 2.0;
            let dx = (pos.x - cx) / cx;
            let dy = (pos.y - cy) / cy;
            dx * dx + dy * dy <= 1.0
        } else {
            false
        }
    }
}

impl ApplicationHandler for OvalApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // --- Window Attributes ---
        let mut attrs = WindowAttributes::default()
            .with_title("Oval")
            .with_inner_size(LogicalSize::new(450.0, 800.0))
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false);

        // --- macOS-specific extensions ---
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowAttributesExtMacOS;
            attrs = attrs
                .with_has_shadow(true)
                // Don't use with_movable_by_window_background -- we want
                // manual control via drag_window() for zone-based dragging
                ;
        }

        let window = event_loop
            .create_window(attrs)
            .expect("failed to create window");

        // --- objc2: Configure NSWindow for transparency ---
        #[cfg(target_os = "macos")]
        unsafe {
            use raw_window_handle::HasWindowHandle;
            use objc2_app_kit::{NSColor, NSView, NSWindow};

            let handle = window.window_handle().unwrap();
            let raw = handle.as_raw();
            if let raw_window_handle::RawWindowHandle::AppKit(appkit_handle) = raw {
                let ns_view: &NSView = appkit_handle
                    .ns_view
                    .cast::<NSView>()
                    .as_ref();

                let ns_window = ns_view.window().unwrap();

                // Defensive: ensure transparency is configured
                // (winit should have done this, but belt-and-suspenders)
                ns_window.setOpaque(false);
                ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
                ns_window.setHasShadow(true);

                // Ensure content view is layer-backed
                if let Some(content_view) = ns_window.contentView() {
                    content_view.setWantsLayer(true);
                }
            }
        }

        // --- TODO: Create wgpu instance, surface, adapter, device ---
        // After surface creation:
        //
        // #[cfg(target_os = "macos")]
        // unsafe {
        //     // Set CAMetalLayer.isOpaque = false
        //     let handle = window.window_handle().unwrap();
        //     let raw = handle.as_raw();
        //     if let raw_window_handle::RawWindowHandle::AppKit(appkit_handle) = raw {
        //         let ns_view: &NSView = appkit_handle.ns_view.cast::<NSView>().as_ref();
        //         let ns_window = ns_view.window().unwrap();
        //         if let Some(content_view) = ns_window.contentView() {
        //             if let Some(layer) = content_view.layer() {
        //                 layer.setOpaque(false);
        //             }
        //         }
        //     }
        // }
        //
        // --- Configure surface with PreMultiplied alpha ---
        // let caps = surface.get_capabilities(&adapter);
        // let alpha_mode = caps.alpha_modes.iter()
        //     .find(|&&m| m == CompositeAlphaMode::PreMultiplied)
        //     .or_else(|| caps.alpha_modes.iter()
        //         .find(|&&m| m == CompositeAlphaMode::PostMultiplied))
        //     .copied()
        //     .unwrap_or(CompositeAlphaMode::Auto);

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.is_inside_oval(self.cursor_pos) {
                    // In drag zone -- initiate window drag
                    if let Some(window) = &self.window {
                        let _ = window.drag_window();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Render the oval here (wgpu render pass)
                // Request next frame:
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = OvalApp::new();
    event_loop.run_app(&mut app).expect("event loop error");
}
```

---

## 7. Cargo.toml Dependencies (Sprint 1 Minimum)

```toml
[package]
name = "oval-player"
version = "0.1.0"
edition = "2021"

[dependencies]
winit = "0.30"
wgpu = "28.0"
raw-window-handle = "0.6"
log = "0.4"
env_logger = "0.11"
pollster = "0.4"

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = { version = "0.2", features = ["NSGeometry"] }
objc2-app-kit = { version = "0.2", features = [
    "NSWindow",
    "NSColor",
    "NSView",
    "NSResponder",
] }
objc2-quartz-core = { version = "0.2", features = [
    "CALayer",
    "CAMetalLayer",
] }
```

**Note on `raw-window-handle`:** winit 0.30 re-exports `raw-window-handle` 0.6 types. You may not need the explicit dependency if you access types through winit's re-export. However, having it explicit avoids version confusion.

---

## 8. Potential Gotchas and Open Questions

### Confirmed Facts
- winit's `with_transparent(true)` sets NSWindow properties but NOT CAMetalLayer.isOpaque
- `drag_window()` works on borderless windows on macOS
- objc2 0.5+ uses `Retained<T>` (not `Id<T>`)
- `CompositeAlphaMode::PreMultiplied` is supported on macOS Metal surfaces
- The raw window handle on macOS gives you an `NSView`, not an `NSWindow` -- you must call `.window()` on it

### Needs Runtime Verification
- Exact timing of when CAMetalLayer becomes available after `create_surface()` -- may need a one-frame delay or may be immediate
- Whether `layer.setOpaque(false)` on the CALayer base class is sufficient, or if you need to downcast to CAMetalLayer specifically (should be sufficient since setOpaque is defined on CALayer)
- Whether shadow rendering looks correct with the oval shape when using shader-based alpha masking (vs. CAShapeLayer masking) -- macOS generates shadows from the opaque content shape, so shader-based alpha should produce the correct shadow outline
- Whether winit's `with_has_shadow(true)` macOS extension actually sets `NSWindow.hasShadow = true` or if we need the objc2 call regardless

### Feature Flag Discovery
The objc2-app-kit and objc2-quartz-core crates use cargo feature flags to gate which class methods are available. If a method like `setOpaque` isn't found at compile time, check that the correct feature is enabled. The feature names typically match the Objective-C header file names:

- `NSWindow` feature -> `NSWindow` class methods
- `NSColor` feature -> `NSColor` class methods
- `NSView` feature -> `NSView` class methods
- `CALayer` feature -> `CALayer` class methods
- `CAMetalLayer` feature -> `CAMetalLayer` class methods

Some methods require additional features. For example, `NSColor::clearColor()` requires the `NSColor` feature on `objc2-app-kit`. If compilation fails with "method not found," the first thing to check is feature flags.
