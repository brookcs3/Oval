use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    // Custom oval drawing shader — white pearl with oil-slick iridescence
    //
    // Visual identity: white/luminous base surface. The sheen shifts through
    // oil-slick colors (magenta, teal, gold, violet) depending on the viewing
    // angle (approximated by UV position + mouse offset). Think: a white
    // sapphire lens with a thin-film interference coating. It has presence —
    // oddly alive, like it shouldn't quite exist on your desktop.
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

            // --- Base: luminous white, slightly warm ---
            let base = vec3(0.95, 0.95, 0.96);

            // --- Thin-film iridescence (oil slick / sapphire glass) ---
            // Simulates thin-film interference: color shifts based on
            // surface angle. We use the distance from center as a proxy
            // for viewing angle (steeper at edges = more color shift).
            // Mouse offset rotates the interference pattern.
            let angle = dist * 3.14159 * 1.8
                + self.mouse_offset.x * 0.5
                + self.mouse_offset.y * 0.3;

            // Three phase-shifted sine waves → RGB thin-film spectrum
            let film_r = 0.5 + 0.5 * sin(angle * 2.0 + 0.0);
            let film_g = 0.5 + 0.5 * sin(angle * 2.0 + 2.094);  // +2π/3
            let film_b = 0.5 + 0.5 * sin(angle * 2.0 + 4.189);  // +4π/3
            let film = vec3(film_r, film_g, film_b);

            // Iridescence strength: strongest at edges (steep angle),
            // absent at center (head-on view). This is physically correct
            // for thin-film interference.
            let iridescence_mask = smoothstep(0.2, 0.9, dist);

            // Mix: at center it's pure white, at edges the oil-slick
            // colors bleed through. Subtle — 25% max blend.
            let iridescent_color = mix(base, film, iridescence_mask * 0.25);

            // --- Specular highlight (upper region) ---
            // Bright white, like light hitting a convex glass surface
            let highlight_center = vec2(0.5 + self.mouse_offset.x * 0.15,
                                        0.28 + self.mouse_offset.y * 0.08);
            let highlight_d = (uv - highlight_center) / vec2(0.38, 0.18);
            let highlight_dist = length(highlight_d);
            let highlight = exp(-highlight_dist * highlight_dist * 2.5);

            // The specular also picks up a faint iridescent tint
            let spec_film_r = 0.5 + 0.5 * sin(angle * 1.5 + 1.0);
            let spec_film_g = 0.5 + 0.5 * sin(angle * 1.5 + 3.094);
            let spec_film_b = 0.5 + 0.5 * sin(angle * 1.5 + 5.189);
            let spec_tint = mix(vec3(1.0), vec3(spec_film_r, spec_film_g, spec_film_b), 0.12);
            let highlight_color = spec_tint * highlight * 0.45;

            // --- Secondary reflection (lower region) ---
            let bottom_center = vec2(0.5 - self.mouse_offset.x * 0.08, 0.83);
            let bottom_d = (uv - bottom_center) / vec2(0.28, 0.09);
            let bottom_dist = length(bottom_d);
            let bottom_highlight = exp(-bottom_dist * bottom_dist * 3.0);
            let bottom_color = vec3(1.0, 1.0, 1.0) * bottom_highlight * 0.12;

            // --- Depth shading: subtle concavity ---
            // Edges darken slightly to give the white surface dimension
            let depth = 1.0 - pow(dist, 2.5) * 0.15;

            // --- Rim: iridescent edge glow ---
            // The very edge of the oval catches the most color — like
            // the rim of a soap bubble or oil on water
            let rim = smoothstep(0.82, 0.98, dist) * (1.0 - smoothstep(1.0 - edge, 1.0, dist));
            let rim_hue = angle * 1.2 + 0.5;
            let rim_r = 0.5 + 0.5 * sin(rim_hue);
            let rim_g = 0.5 + 0.5 * sin(rim_hue + 2.094);
            let rim_b = 0.5 + 0.5 * sin(rim_hue + 4.189);
            let rim_color = vec3(rim_r, rim_g, rim_b) * rim * 0.35;

            // --- Composite ---
            let color = iridescent_color * depth + highlight_color + bottom_color + rim_color;
            let color = clamp(color, vec3(0.0), vec3(1.0));

            return vec4(color, oval_alpha);
        }
    }

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
                            mouse_offset: vec2(0.0, 0.0)
                        }
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawOval {
    #[deref]
    pub draw_super: DrawQuad,
    #[live]
    pub mouse_offset: Vec2,
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    window_size: DVec2,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
    }
}

/// Check if a point (in window coordinates) is inside the oval.
/// The oval fills the entire window as an ellipse.
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
        // Transport controls in Sprint 4
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);

        // Window drag: let user drag the window by clicking inside the oval.
        // Makepad's WindowDragQuery fires on every mouse-down; respond with
        // Caption to tell the OS to start a window drag.
        if let Event::WindowDragQuery(dq) = event {
            if point_in_oval(dq.abs, self.window_size) {
                dq.response.set(WindowDragQueryResponse::Caption);
            }
        }

        // Track window size for hit-testing
        if let Event::WindowGeomChange(gc) = event {
            self.window_size = gc.new_geom.inner_size;
        }

        // Mouse hover: update the DrawOval shader's mouse_offset uniform
        // so the iridescent highlight follows the cursor.
        if let Event::MouseMove(mm) = event {
            if self.window_size.x > 0.0 && self.window_size.y > 0.0 {
                // Normalize mouse position to -1..1 range centered on oval
                let nx = (mm.abs.x / self.window_size.x - 0.5) * 2.0;
                let ny = (mm.abs.y / self.window_size.y - 0.5) * 2.0;

                // Update the shader uniform on the oval_view's background
                let oval_view = self.ui.view(id!(oval_view));
                oval_view.apply_over(cx, live! {
                    draw_bg: { mouse_offset: (vec2(nx as f32, ny as f32)) }
                });
                oval_view.redraw(cx);
            }
        }

        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
