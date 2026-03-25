---
category: gotcha
priority: critical
title: Do not change the technology stack without explicit user approval
context: Architecture, dependency, and framework decisions
tags: [architecture, critical, confirmed-by-user]
learned_from: smart-init-confirmed
---

A previous Claude session unilaterally replaced the entire technology stack (winit+wgpu+egui → Makepad, dropped Windows, resurrected Mojo) in a single commit without user approval. This wasted work and required a hard reset.

**Why:** The user explicitly reverted this on 2026-03-24. Stack changes are architectural decisions that must be discussed and approved.

**How to apply:** Never change frameworks, drop platform targets, or swap core dependencies without presenting the trade-offs and getting explicit confirmation. If a framework isn't working, discuss alternatives — don't just switch.
