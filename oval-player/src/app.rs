use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    // ─────────────────────────────────────────────────────────────
    // DrawOval: White pearl with occasional oil-slick shimmer
    //
    // The oval is an OBJECT — a mysterious artifact. Mostly quiet,
    // luminous white pearl. Occasionally, a shimmer of oil-slick
    // iridescence flows across the surface like thin-film interference.
    // Arcane. Otherworldly. A thing from somewhere else.
    // ─────────────────────────────────────────────────────────────
    DrawOval = {{DrawOval}} {
        fn pixel(self) -> vec4 {
            let center = self.rect_size * 0.5;
            let p = (self.pos * self.rect_size - center) / center;
            let dist = length(p);

            // Oval mask with anti-aliased edge
            let edge = 0.01;
            let oval_alpha = 1.0 - smoothstep(1.0 - edge, 1.0 + edge, dist);

            if oval_alpha < 0.01 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            let uv = self.pos;

            // ═══════════════════════════════════════════════════════════
            // OCCASIONAL SHIMMER — oil slick flows across every ~5-8 sec
            // ═══════════════════════════════════════════════════════════
            //
            // Multiple sine waves at incommensurate frequencies create
            // quasi-random "shimmer events". When waves align → shimmer.

            let wave1 = sin(self.time * 0.4) * sin(self.time * 0.17);
            let wave2 = sin(self.time * 0.29 + 2.0) * sin(self.time * 0.11);
            let shimmer_trigger = max(wave1, wave2);

            // Shimmer only appears when trigger exceeds threshold
            // High threshold = rare, subtle appearances
            let shimmer_active = smoothstep(0.75, 0.92, shimmer_trigger);

            // The shimmer flows diagonally across the surface
            // Direction slowly rotates over time for variety
            let flow_angle = self.time * 0.05;
            let flow_dir = vec2(cos(flow_angle), sin(flow_angle) * 0.6);
            let flow_pos = dot(p, flow_dir);

            // Shimmer wave travels across the surface
            let shimmer_phase = self.time * 0.7;
            let shimmer_wave = flow_pos - fract(shimmer_phase) * 2.5 + 0.8;

            // Sharp wavefront with soft falloff — like oil spreading
            let shimmer_band = exp(-shimmer_wave * shimmer_wave * 8.0)
                             * smoothstep(-0.3, 0.1, shimmer_wave);

            // Oil-slick spectrum: deep purples, teals, magentas, golds
            let oil_hue = shimmer_wave * 12.0 + self.time * 0.3 + dist * 4.0;
            let oil_r = 0.5 + 0.5 * sin(oil_hue + 0.0);
            let oil_g = 0.5 + 0.5 * sin(oil_hue + 2.094);
            let oil_b = 0.5 + 0.5 * sin(oil_hue + 4.189);
            let oil_color = vec3(oil_r, oil_g, oil_b);

            // Final shimmer: active * band * color intensity
            // Subtle — a hint, not a show
            let shimmer = shimmer_active * shimmer_band * 0.22;

            // ═══════════════════════════════════════════════════════════
            // BASE: Luminous white pearl — the quiet state
            // ═══════════════════════════════════════════════════════════

            let base = vec3(0.96, 0.96, 0.97);

            // Very subtle static iridescence at edges (always present, faint)
            let edge_angle = dist * 3.14159 * 1.5
                + self.mouse_offset.x * 0.4
                + self.mouse_offset.y * 0.25;
            let edge_film_r = 0.5 + 0.5 * sin(edge_angle * 2.0);
            let edge_film_g = 0.5 + 0.5 * sin(edge_angle * 2.0 + 2.094);
            let edge_film_b = 0.5 + 0.5 * sin(edge_angle * 2.0 + 4.189);
            let edge_film = vec3(edge_film_r, edge_film_g, edge_film_b);

            // Edge iridescence: very subtle, only at rim
            let edge_mask = smoothstep(0.5, 0.95, dist) * 0.12;
            let pearl_color = mix(base, edge_film, edge_mask);

            // ═══════════════════════════════════════════════════════════
            // SPECULAR HIGHLIGHTS — the glass surface
            // ═══════════════════════════════════════════════════════════

            // Primary highlight (upper region, follows mouse)
            let highlight_center = vec2(
                0.5 + self.mouse_offset.x * 0.15,
                0.28 + self.mouse_offset.y * 0.08
            );
            let highlight_d = (uv - highlight_center) / vec2(0.38, 0.18);
            let highlight_dist = length(highlight_d);
            let highlight = exp(-highlight_dist * highlight_dist * 2.5) * 0.38;

            // Secondary reflection (lower, dimmer)
            let bottom_center = vec2(0.5 - self.mouse_offset.x * 0.08, 0.83);
            let bottom_d = (uv - bottom_center) / vec2(0.28, 0.09);
            let bottom_dist = length(bottom_d);
            let bottom_highlight = exp(-bottom_dist * bottom_dist * 3.0) * 0.10;

            // Depth shading — center brighter than edges
            let depth = 1.0 - pow(dist, 2.5) * 0.12;

            // ═══════════════════════════════════════════════════════════
            // RIM — subtle edge definition
            // ═══════════════════════════════════════════════════════════

            let rim = smoothstep(0.85, 0.98, dist) * (1.0 - smoothstep(1.0 - edge, 1.0, dist));
            let rim_hue = edge_angle * 1.2 + shimmer_trigger * 2.0;
            let rim_r = 0.5 + 0.5 * sin(rim_hue);
            let rim_g = 0.5 + 0.5 * sin(rim_hue + 2.094);
            let rim_b = 0.5 + 0.5 * sin(rim_hue + 4.189);
            let rim_color = vec3(rim_r, rim_g, rim_b) * rim * (0.18 + shimmer_active * 0.12);

            // ═══════════════════════════════════════════════════════════
            // DROP ZONE INDICATOR — shows on hover
            // ═══════════════════════════════════════════════════════════

            let drop_ring = smoothstep(0.58, 0.60, dist) * (1.0 - smoothstep(0.60, 0.62, dist));
            let drop_color = vec3(0.7, 0.75, 0.85) * drop_ring * self.hover * 0.4;

            // ═══════════════════════════════════════════════════════════
            // COMPOSITE — bring it all together
            // ═══════════════════════════════════════════════════════════

            let color = pearl_color * depth;

            // Add the occasional oil-slick shimmer
            let color = mix(color, oil_color, shimmer);

            // Add highlights
            let color = color + vec3(highlight + bottom_highlight);

            // Add rim and drop indicator
            let color = color + rim_color + drop_color;

            let color = clamp(color, vec3(0.0), vec3(1.0));

            return vec4(color, oval_alpha);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // App layout
    // ─────────────────────────────────────────────────────────────
    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: {
                    inner_size: vec2(450.0, 800.0),
                },
                pass: {
                    clear_color: #0000
                },
                caption_bar = {
                    visible: false
                },
                body = <View> {
                    width: Fill,
                    height: Fill,
                    oval_view = <View> {
                        width: Fill,
                        height: Fill,
                        show_bg: true,
                        draw_bg: <DrawOval> {
                            mouse_offset: vec2(0.0, 0.0),
                            time: 0.0,
                            hover: 0.0,
                        }
                    }
                    // "DROP VIDEO" label, centered in the oval
                    drop_label = <View> {
                        width: Fill,
                        height: Fill,
                        align: { x: 0.5, y: 0.55 },
                        flow: Down,
                        <Label> {
                            text: "DROP VIDEO",
                            draw_text: {
                                color: #bbbcc0,
                                text_style: {
                                    font_size: 14.0,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

app_main!(App);

// ─────────────────────────────────────────────────────────────
// DrawOval Rust struct — fields become shader uniforms
// ─────────────────────────────────────────────────────────────
#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawOval {
    #[deref]
    pub draw_super: DrawQuad,
    #[live]
    pub mouse_offset: Vec2,
    #[live]
    pub time: f32,
    #[live]
    pub hover: f32,
}

// ─────────────────────────────────────────────────────────────
// App state
// ─────────────────────────────────────────────────────────────
#[derive(Clone, Debug, PartialEq)]
pub enum PlayerState {
    Empty,
    Playing,
    Paused,
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    window_size: DVec2,
    #[rust]
    state: PlayerState,
    #[rust]
    video_path: Option<String>,
    #[rust]
    time: f64,
    #[rust]
    mouse_inside: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::Empty
    }
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
    }
}

/// Ellipse hit-test: is this point inside the oval?
fn point_in_oval(pos: DVec2, window_size: DVec2) -> bool {
    if window_size.x <= 0.0 || window_size.y <= 0.0 {
        return false;
    }
    let center = window_size * 0.5;
    let dx = (pos.x - center.x) / center.x;
    let dy = (pos.y - center.y) / center.y;
    (dx * dx + dy * dy) <= 1.0
}

impl MatchEvent for App {
    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {
        // Transport control actions will go here in Sprint 4
    }

    fn handle_key_down(&mut self, cx: &mut Cx, event: &KeyEvent) {
        match event.key_code {
            // Escape quits
            KeyCode::Escape => cx.quit(),

            // Space toggles play/pause (once video is loaded)
            KeyCode::Space => {
                match self.state {
                    PlayerState::Playing => self.state = PlayerState::Paused,
                    PlayerState::Paused => self.state = PlayerState::Playing,
                    PlayerState::Empty => {}
                }
            }
            _ => {}
        }
    }

    fn handle_startup(&mut self, cx: &mut Cx) {
        // Start the animation timer — this drives the idle breathing
        // and will later drive video frame presentation
        let _ = cx.start_interval(1.0 / 60.0);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);

        // ── Animation tick: advance time, redraw ──
        if let Event::Timer(_te) = event {
            self.time += 1.0 / 60.0;

            // Update time uniform on shader for idle breathing animation
            let oval_view = self.ui.view(id!(oval_view));
            oval_view.apply_over(cx, live! {
                draw_bg: { time: (self.time as f32) }
            });
            oval_view.redraw(cx);
        }

        // ── Window drag: click inside oval = drag window ──
        if let Event::WindowDragQuery(dq) = event {
            if point_in_oval(dq.abs, self.window_size) {
                dq.response.set(WindowDragQueryResponse::Caption);
            }
        }

        // ── Track window geometry ──
        if let Event::WindowGeomChange(gc) = event {
            self.window_size = gc.new_geom.inner_size;
        }

        // ── Mouse tracking: update shader uniforms ──
        if let Event::MouseMove(mm) = event {
            if self.window_size.x > 0.0 && self.window_size.y > 0.0 {
                let nx = (mm.abs.x / self.window_size.x - 0.5) * 2.0;
                let ny = (mm.abs.y / self.window_size.y - 0.5) * 2.0;
                let inside = point_in_oval(mm.abs, self.window_size);

                self.mouse_inside = inside;

                let oval_view = self.ui.view(id!(oval_view));
                oval_view.apply_over(cx, live! {
                    draw_bg: {
                        mouse_offset: (vec2(nx as f32, ny as f32)),
                        hover: (if inside { 1.0f32 } else { 0.0f32 }),
                    }
                });
                oval_view.redraw(cx);

                // Show/hide the drop label based on hover + state
                let drop_label = self.ui.view(id!(drop_label));
                drop_label.set_visible(cx, self.state == PlayerState::Empty && inside);
            }
        }

        // ── File drag-and-drop ──
        if let Event::Drag(dh) = event {
            // Show we accept drops inside the oval
            if point_in_oval(dh.abs, self.window_size) {
                *dh.response.lock().unwrap() = DragResponse::Copy;
            }
        }

        if let Event::Drop(de) = event {
            if point_in_oval(de.abs, self.window_size) {
                // Extract file path from the drop
                for item in de.items.iter() {
                    if let DragItem::FilePath { path, .. } = item {
                        let path_str = path.to_string();
                        // Check for video file extensions
                        let ext = path_str.rsplit('.').next().unwrap_or("").to_lowercase();
                        if matches!(ext.as_str(),
                            "mp4" | "mkv" | "webm" | "mov" | "avi" | "m4v" | "ts"
                        ) {
                            log!("Oval: Video file dropped: {}", path_str);
                            self.video_path = Some(path_str);
                            self.state = PlayerState::Paused;

                            // Hide the drop label
                            let drop_label = self.ui.view(id!(drop_label));
                            drop_label.set_visible(cx, false);
                        }
                    }
                }
            }
        }

        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
