---
category: pattern
priority: medium
title: Video decode pipeline design
context: Video playback implementation
tags: [video, ffmpeg, pipeline]
learned_from: smart-init-discovery
---

**Decoding:** ffmpeg-next crate. Supports H.264, H.265, VP9, AV1 and all containers.

**Optimal texture upload:** Upload YUV planes as 3 separate GPU textures (Y, U, V), convert to RGB in fragment shader. This halves upload bandwidth vs CPU-side RGBA conversion.

**Threading:** Decode thread → ring buffer (2-4 frames ahead) → Main thread consumes for rendering. crossbeam-channel for commands, ringbuf for frames, AtomicU64 for audio clock.

**Seeking:** Coarse scrub (keyframe-only during drag) + fine resolve (decode-forward on release). av_seek_frame with AVSEEK_FLAG_BACKWARD to nearest keyframe before target, then decode forward discarding until target PTS.

**Hardware acceleration:** VideoToolbox (macOS), DXVA2/D3D11VA (Windows). Automatic fallback to software decode if HW unavailable.

**How to apply:** Follow this pipeline architecture. Don't do CPU-side YUV→RGB conversion — use the GPU shader path.
