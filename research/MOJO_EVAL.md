# Mojo Language Evaluation for Oval Video Player

## Executive Summary

**Verdict: Mojo is NOT suitable for the Oval video player project at this time.**

Mojo is a powerful systems programming language designed for AI/ML and GPU computing, but it lacks the fundamental capabilities required for GUI application development. The Oval project requires a fallback to an established systems language. This document evaluates Mojo's current state and recommends **Rust** as the primary implementation language, with platform-specific Swift (macOS) and C++ (Windows) for window management interop.

---

## 1. Mojo Language Overview

Mojo is developed by Modular Inc., founded by Chris Lattner (creator of Swift and LLVM) and Tim Davis. It aims to combine Python's usability with systems-level performance comparable to C++, Rust, and Zig. The language compiles through MLIR (Multi-Level Intermediate Representation), enabling code generation for CPUs, GPUs, and ASICs.

### Current Version Status (January 2026)

- **Latest release:** v0.25.6+ (installed via `pip install mojo` since September 2025)
- **Compiler status:** Closed source (open source standard library)
- **Mojo 1.0 target:** H1 2026 (announced December 2025)
- **Platforms:** Linux and macOS only (no Windows support)
- **Funding:** $380M total at $1.6B valuation (September 2025 round)

### Language Features Present

| Feature | Status |
|---------|--------|
| `fn` typed functions | Available |
| `struct` types | Available |
| Operator overloading | Available |
| Decorators | Available |
| Borrow checker | Available |
| Python interop | Available (import Python modules) |
| Static typing with inference | Available |
| SIMD intrinsics | Available |
| GPU kernel programming | Available |
| `var`/`let` bindings | Available |

### Language Features Missing (Critical for GUI)

| Feature | Status | Impact on Oval |
|---------|--------|----------------|
| **Classes** | Not supported | Cannot use inheritance-based GUI frameworks |
| **Lambda syntax** | Not started | Cannot use callback-driven UI patterns |
| **Mature C FFI** | Partial (DLHandle only) | Cannot ergonomically bind to native GUI APIs |
| **Windows support** | Not available | Eliminates cross-platform requirement |
| **Native GUI libraries** | None exist | No path to creating windows/UI |
| **Stable toolchain** | Not yet | Cross-compilation, packaging, build system incomplete |
| **List/dict comprehensions** | Missing | Minor but notable Python incompatibility |
| **`global` keyword** | Missing | Limits state management patterns |

---

## 2. GUI Development Feasibility Analysis

### Direct GUI Development: Not Possible

Mojo has **zero native GUI libraries or frameworks**. Community members on the Modular Discord confirm that "GUI is going to be very painful right now." The language lacks:

1. **No windowing abstraction** — No equivalent to winit, GLFW, SDL, or native window APIs
2. **No rendering pipeline** — No 2D/3D graphics libraries
3. **No event loop** — No input handling infrastructure
4. **No widget toolkit** — No buttons, sliders, text rendering

### Workaround Evaluation

#### Option A: Python Interop for GUI
- **Approach:** Import Python GUI libraries (tkinter, PyQt, etc.) via Mojo's Python interop
- **Problems:** Performance bottleneck at interop boundary; Python GUI libraries are not suitable for real-time video rendering at 60fps; cannot achieve transparent/non-rectangular windows through Python abstractions
- **Verdict:** Non-viable for a performance-critical video player

#### Option B: C FFI via DLHandle
- **Approach:** Manually bind to native GUI APIs (Cocoa, Win32) through dynamic linking
- **Problems:** Requires hand-writing every binding (no automatic binding generation); verbose and error-prone; no class support means wrapping OOP-based APIs (NSWindow, etc.) is architecturally painful; Windows not supported at all
- **Verdict:** Theoretically possible on macOS with enormous effort, impossible on Windows

#### Option C: Web-Based UI
- **Approach:** Run a local web server from Mojo, render UI in a browser
- **Problems:** Cannot achieve transparent oval window; cannot access hardware video acceleration; latency overhead; defeats the purpose of a native application
- **Verdict:** Non-viable for project requirements

### The Fundamental Blocker

Even if Mojo's FFI were mature, the **absence of Windows platform support** is an absolute disqualifier. The Oval spec requires cross-platform operation on both macOS and Windows. Mojo currently targets only Linux and macOS.

---

## 3. Recommended Alternative: Rust

### Why Rust

Rust is the natural fallback for this project. It shares Mojo's goals of systems-level performance with safety guarantees, and it has a mature ecosystem for every component Oval requires:

| Requirement | Rust Solution | Maturity |
|-------------|--------------|----------|
| Cross-platform windowing | `winit` | Production-ready |
| GPU rendering | `wgpu` (Vulkan/Metal/DX12/OpenGL) | Production-ready |
| Video decoding | `ffmpeg-next`, `rsmpeg` | Production-ready |
| Hardware acceleration | Via FFmpeg + platform APIs | Production-ready |
| UI widgets | `egui` (immediate mode) | Production-ready |
| Transparent windows | `winit` + platform interop | Functional (with caveats) |

### Rust vs Other Alternatives

#### Swift
- **Pros:** Native macOS support, first-class Cocoa/AppKit access, excellent for oval NSWindow
- **Cons:** No Windows support; would require separate C++/Win32 codebase for Windows
- **Role:** Could be used for macOS-specific window management via Rust FFI

#### C++ (Qt/SDL)
- **Pros:** Mature cross-platform, full platform API access
- **Cons:** Memory safety concerns, slower iteration, complex build systems
- **Role:** Not recommended as primary language given Rust's advantages

#### Electron/Tauri
- **Pros:** Easy cross-platform UI
- **Cons:** Cannot achieve transparent non-rectangular windows with proper hit-testing; web rendering overhead; not suitable for 60fps video with shader overlays
- **Role:** Not viable

---

## 4. Mojo's Future Relevance

Mojo remains an interesting language with genuine potential. Key milestones that could make it viable for projects like Oval in the future:

1. **Mojo 1.0 (H1 2026):** Will bring stability and open-source compiler
2. **Classes support:** Would enable wrapping OOP-based GUI frameworks
3. **Mature C/C++ FFI:** Would allow binding to native platform APIs
4. **Windows platform support:** Would fulfill cross-platform requirement
5. **Community GUI libraries:** Would need to emerge post-1.0

Realistically, Mojo could become viable for GUI applications in **late 2027 or later**, assuming 1.0 ships on schedule and community ecosystem development follows.

---

## 5. Recommended Technology Stack

Based on this evaluation, the recommended stack for Oval is:

```
┌─────────────────────────────────────────────┐
│              Application Layer              │
│        Rust (core logic, state mgmt)        │
├─────────────────────────────────────────────┤
│              UI Layer                       │
│   egui (immediate-mode widgets over wgpu)   │
├─────────────────────────────────────────────┤
│            Rendering Layer                  │
│  wgpu (Metal/Vulkan/DX12) + custom shaders  │
├─────────────────────────────────────────────┤
│            Video Decode Layer               │
│  ffmpeg-next (H.264/H.265/VP9/AV1 + hwaccel)│
├─────────────────────────────────────────────┤
│           Windowing Layer                   │
│  winit + platform-specific oval masking     │
│  macOS: NSWindow/CALayer | Win: WS_EX_LAYERED│
├─────────────────────────────────────────────┤
│              OS / Hardware                  │
│    macOS (Metal) | Windows (DX12/Vulkan)    │
└─────────────────────────────────────────────┘
```

---

## 6. Conclusion

Mojo is a promising language for AI/ML and GPU computing but is fundamentally unready for GUI application development. The absence of classes, mature FFI, native GUI libraries, and Windows support makes it unsuitable for Oval. **Rust provides every capability Oval requires** with a production-ready ecosystem and strong cross-platform story. The project should proceed with Rust as the primary language, leveraging wgpu for rendering, ffmpeg-next for video decoding, winit for windowing, and egui for UI controls.

The spirit of the original spec — showcasing a cutting-edge language's "power and versatility" — can still be honored by building Oval in Rust, which is itself a modern systems language that proves you can have both safety and performance without compromise.
