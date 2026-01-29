# Video Technology Research for Oval Player

## Executive Summary

This document surveys the video codec landscape, hardware acceleration APIs, and decoding library options for the Oval video player. The recommended approach is to use **FFmpeg via the `ffmpeg-next` Rust crate** for decoding, with **hardware acceleration through platform-native APIs** (VideoToolbox on macOS, DXVA2/D3D11VA on Windows). Video frames are uploaded as GPU textures via wgpu for rendering within the oval viewport.

---

## 1. Video Codec Landscape (2025-2026)

### Codec Matrix

| Codec | Standard | License | Typical Use | Bit Depth | Max Resolution |
|-------|----------|---------|-------------|-----------|----------------|
| **H.264/AVC** | ITU-T/ISO | Licensed (widely free in practice) | Universal baseline | 8-bit | 8K (Level 6.2) |
| **H.265/HEVC** | ITU-T/ISO | Licensed (royalty fees) | 4K streaming, Apple ecosystem | 10-bit HDR | 8K |
| **VP9** | Google | Royalty-free | YouTube, Android, Chrome | 10-bit HDR | 8K |
| **AV1** | Alliance for Open Media | Royalty-free | Next-gen streaming | 10-bit HDR | 8K+ |
| **H.266/VVC** | ITU-T/ISO | Licensed | Future standard | 10-bit HDR | 16K |

### Codec Adoption Status (2025)

**H.264** remains the universal baseline — virtually all hardware and software can decode it. It is the "safe default" for any video player.

**HEVC/H.265** has widespread hardware support across smartphones, TVs, and computers, especially within the Apple ecosystem. Licensing fees remain a barrier to universal adoption, but hardware decoders are ubiquitous on modern devices.

**VP9** is still prevalent on YouTube and Android devices. Hardware decoding support is strong on Intel, AMD, and NVIDIA GPUs. Browser support is solid in Chrome, Firefox, and Edge. VP9 serves as the fallback where AV1 hardware decode isn't available.

**AV1** is the clear future standard. YouTube now encodes more than 75% of its videos in AV1. Over 70% of global video watched on Meta platforms uses AV1. Hardware decoding is available on:
- Apple Silicon M3+ (macOS)
- NVIDIA RTX 30 series and newer (Ampere+)
- AMD RDNA 2 and newer
- Intel Arc and 12th gen+ integrated graphics
- Qualcomm Snapdragon 888+

**VVC/H.266** is too early for practical support. No significant hardware decoder deployment exists. Not recommended for Oval's initial release.

### Container Formats

| Container | Extension | Common Codecs | Notes |
|-----------|-----------|---------------|-------|
| MP4 (ISOBMFF) | .mp4, .m4v | H.264, H.265, AV1 | Most common, universally supported |
| Matroska | .mkv | All codecs | Flexible, common for enthusiast content |
| WebM | .webm | VP8, VP9, AV1 | Web-optimized subset of Matroska |
| MOV | .mov | H.264, H.265, ProRes | Apple ecosystem |
| AVI | .avi | Legacy codecs | Legacy support only |

### Recommendation for Oval

Support all major codecs through FFmpeg's unified decoder API. Priority order:
1. **H.264** — universal compatibility baseline
2. **H.265/HEVC** — 4K/HDR content
3. **AV1** — modern content, rapidly growing
4. **VP9** — YouTube/web content compatibility
5. Others (VP8, MPEG-2, etc.) — "free" through FFmpeg, no extra effort

---

## 2. Hardware Acceleration APIs

### macOS: VideoToolbox + Metal

**VideoToolbox** is Apple's hardware-accelerated video encode/decode framework. It is the only hardware acceleration path on macOS.

**Supported codecs via hardware:**
- H.264: All Macs from 2012+
- H.265/HEVC: Macs from 2017+ (excluding MacBook Air 13-inch 2017)
- VP9: Recent Intel Macs with hardware support, Apple Silicon
- AV1: Apple Silicon M3 series and later only

**Integration path:** FFmpeg's `videotoolbox` hwaccel backend handles the VideoToolbox API. Decoded frames can be in `CVPixelBuffer` format, which can be rendered via Metal or converted to RGB for wgpu texture upload.

**Metal rendering:** wgpu uses Metal as its backend on macOS. Decoded frames from VideoToolbox can be uploaded to wgpu textures either:
- Direct Metal texture sharing (zero-copy, optimal but requires unsafe Metal interop)
- CPU-side copy: decode → `CVPixelBuffer` → `memcpy` to staging buffer → `queue.write_texture()` (simpler, slight overhead)

### Windows: DXVA2 / D3D11VA / D3D12VA

**DXVA2 (DirectX Video Acceleration 2)** and **D3D11VA** are the standard hardware decode APIs on Windows.

**NVIDIA NVDEC supported codecs:**
- MPEG-2, VC-1, H.264, H.265/HEVC, VP8, VP9, AV1
- RTX 30+ (Ampere): 8K@60 AV1 decoding
- Blackwell: Doubled H.264 throughput, H.264/HEVC 4:2:2 support

**AMD AMF supported codecs:**
- H.264, H.265/HEVC, VP9, AV1 (RDNA 2+)

**Intel Quick Sync (QSV) supported codecs:**
- H.264, H.265/HEVC, VP9, AV1 (12th gen+)

**Integration path:** FFmpeg's `d3d11va` or `dxva2` hwaccel backends. Decoded frames arrive as D3D11 textures, which can be shared with wgpu (DX12 backend) or copied to CPU for texture upload.

### Linux (Future Consideration)

If Linux support is ever added:
- **VA-API** (Intel, AMD) and **VDPAU** (NVIDIA legacy) are the hardware decode APIs
- FFmpeg supports both transparently

### Hardware Acceleration Decision Matrix

```
Codec Request
      │
      ▼
 ┌──────────────────┐
 │  Is HW decoder    │──── Yes ──→ Use HW decoder (VideoToolbox/DXVA)
 │  available for    │              │
 │  this codec?      │              ▼
 └──────────────────┘        Decode to GPU texture
          │                   (zero-copy if possible)
          No
          │
          ▼
 ┌──────────────────┐
 │  Use FFmpeg SW    │
 │  decoder (libavcodec)│
 └──────────────────┘
          │
          ▼
    Decode to CPU buffer
    Upload via write_texture()
```

---

## 3. Decoding Library Options

### Option 1: FFmpeg via `ffmpeg-next` (Recommended)

**`ffmpeg-next`** is the most mature Rust FFmpeg binding. It wraps libavcodec, libavformat, libavutil, and libswscale.

**Advantages:**
- Supports every codec Oval needs (and hundreds more)
- Hardware acceleration support through hwaccel API
- Battle-tested (FFmpeg itself processes the majority of the world's video)
- Active maintenance
- Container format parsing included (libavformat)
- Audio decoding included for future expansion

**Disadvantages:**
- Large dependency (FFmpeg itself)
- Build complexity (must link FFmpeg C libraries)
- Unsafe FFI boundary requires careful memory management

**Key APIs for Oval:**
```
avformat_open_input()     → Open video file
avformat_find_stream_info() → Parse stream metadata
av_read_frame()           → Read compressed packets
avcodec_send_packet()     → Send packet to decoder
avcodec_receive_frame()   → Receive decoded frame
av_seek_frame()           → Seek to timestamp (frame-accurate with AVSEEK_FLAG_BACKWARD)
sws_scale()              → Color space conversion (YUV → RGB for GPU upload)
```

### Option 2: `rsmpeg` (Alternative)

**`rsmpeg`** (by Lark/Feishu) is a thin safe layer above FFmpeg's Rust bindings via `rusty_ffmpeg`.

**Advantages:**
- Safer API than `ffmpeg-next`
- Supports FFmpeg 6.x and 7.x
- Active maintenance by a large company (ByteDance/Lark)

**Disadvantages:**
- Smaller community than `ffmpeg-next`
- Slightly less documentation

### Option 3: GStreamer via `gstreamer-rs`

**Advantages:**
- Pipeline-based architecture (clean abstraction)
- Built-in hardware acceleration
- Handles audio sync automatically

**Disadvantages:**
- Heavy runtime dependency
- Complex pipeline configuration
- Overkill for a focused video player
- More difficult to integrate with custom wgpu rendering

### Option 4: Platform-Native Decoders Only

Using only VideoToolbox (macOS) and Media Foundation (Windows) directly.

**Advantages:**
- No external dependencies
- Guaranteed hardware acceleration
- Smaller binary size

**Disadvantages:**
- Must write two completely separate decoder backends
- Limited codec support compared to FFmpeg
- Much more code to maintain
- No software fallback for unsupported codecs

### Recommendation

**Use `ffmpeg-next` as the primary decoding library.** It provides the best balance of codec coverage, hardware acceleration support, and ecosystem maturity. The build complexity is a one-time cost that pays dividends in codec compatibility.

---

## 4. Frame-Accurate Seeking

Frame-accurate seeking is critical for the drag-to-scrub timeline feature. Here's how it works with FFmpeg:

### The Problem

Video codecs use inter-frame compression. Most frames (P-frames, B-frames) depend on previous frames. Only I-frames (keyframes) can be decoded independently. Keyframes typically appear every 1-5 seconds.

### The Solution: Seek + Decode Forward

```
Timeline:  [I]--[P]--[P]--[B]--[P]--[I]--[P]--[TARGET]--[P]--[I]
                                              ↑
                                        User wants this frame

Step 1: av_seek_frame() with AVSEEK_FLAG_BACKWARD
        → Seeks to nearest keyframe BEFORE target
        → Lands at [I] before target

Step 2: Decode forward, discarding frames
        → Decode [I] (discard)
        → Decode [P] (discard)
        → Decode [TARGET] ← this is the frame we want

Step 3: Display TARGET frame
```

### Performance Optimization for Scrubbing

During scrubbing (user dragging timeline), frame-accurate decode of every position would be too slow. Strategy:

1. **Coarse scrubbing:** During active drag, seek to nearest keyframe only (fast, approximate position shown)
2. **Fine resolve:** When user releases drag, perform full frame-accurate seek to exact position
3. **Thumbnail cache:** Pre-generate keyframe thumbnails for timeline preview (background task)

### Timestamp Mapping

FFmpeg uses `pts` (presentation timestamp) in stream timebase units. For timeline display:

```
display_seconds = frame.pts * stream.time_base.num / stream.time_base.den
timeline_position = display_seconds / total_duration
```

---

## 5. Video Frame Pipeline for Oval

### Decode → Render Pipeline

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  FFmpeg      │     │  Color Space │     │   wgpu       │
│  Decoder     │────→│  Conversion  │────→│  Texture     │
│  (HW or SW)  │     │  YUV→RGBA    │     │  Upload      │
└─────────────┘     └──────────────┘     └──────────────┘
                                                │
                                                ▼
                                         ┌──────────────┐
                                         │  Oval Mask   │
                                         │  + Glossy    │
                                         │  Overlay     │
                                         │  (Fragment   │
                                         │   Shader)    │
                                         └──────────────┘
                                                │
                                                ▼
                                         ┌──────────────┐
                                         │  Present to  │
                                         │  Screen      │
                                         └──────────────┘
```

### Frame Format Handling

Most decoders output in YUV color space (typically NV12 or YUV420P). Conversion to RGBA is needed before GPU upload:

- **Software path:** Use `sws_scale()` for YUV→RGBA conversion, then `queue.write_texture()` to upload
- **GPU path (optimal):** Upload YUV planes as separate textures, convert in fragment shader (avoids CPU-side conversion)

The GPU path is recommended for performance: upload 3 smaller textures (Y, U, V planes) and perform YUV→RGB in the fragment shader. This halves the upload bandwidth compared to converting to RGBA on the CPU.

### Threading Model

```
┌────────────────────┐    ┌────────────────────┐    ┌──────────────────┐
│   Decode Thread    │    │   Main Thread      │    │   Render Thread  │
│                    │    │                    │    │   (wgpu)         │
│  Read packets      │───→│  Receive frames    │───→│  Upload texture  │
│  Decode frames     │    │  Handle input      │    │  Draw oval       │
│  Color convert     │    │  Update timeline   │    │  Apply effects   │
│                    │    │  Manage state      │    │  Present         │
└────────────────────┘    └────────────────────┘    └──────────────────┘
        ↑                         ↑
        │                         │
   Ring buffer              User input
   (decoded frames)         (winit events)
```

Use a ring buffer (e.g., `ringbuf` crate) between decode and render threads to decouple decode speed from render framerate. Target 2-4 frames of decode-ahead buffer for smooth playback.

---

## 6. Audio Considerations

While the initial Oval spec focuses on video, audio is essential for a complete player:

- **Decode:** FFmpeg handles audio decoding alongside video
- **Output:** `cpal` crate for cross-platform audio output
- **Sync:** Audio clock drives video presentation (audio-master sync strategy)
- **A/V sync formula:** `video_delay = video_pts - audio_clock; if delay > threshold, skip/repeat frame`

This can be implemented as a Phase 2.5 feature after core video playback works.

---

## 7. Codec Support Priority for Implementation

### Phase 1 (MVP)
1. H.264 in MP4 container (most common format)
2. Software decoding only (simplify initial pipeline)

### Phase 2 (Core Codecs)
3. H.265/HEVC
4. VP9
5. AV1
6. MKV and WebM container support
7. Hardware acceleration (VideoToolbox on macOS, DXVA on Windows)

### Phase 3 (Polish)
8. HDR tone mapping (for 10-bit content on SDR displays)
9. Subtitle rendering (SRT, ASS)
10. Audio playback and A/V sync

---

## 8. Key Dependencies Summary

| Component | Crate | Purpose |
|-----------|-------|---------|
| Video decoding | `ffmpeg-next` | Codec access, seeking, format parsing |
| Color conversion | `ffmpeg-next` (sws) or GPU shader | YUV→RGB |
| Hardware accel | FFmpeg hwaccel API | VideoToolbox, DXVA |
| Frame timing | `frame-tick` or custom | NTSC-compatible frame scheduling |
| Audio output | `cpal` | Cross-platform audio |
| Ring buffer | `ringbuf` | Lock-free decode↔render communication |
