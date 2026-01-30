use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    // Custom oval drawing shader — the core visual identity
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

            // --- Background: dark metallic gradient ---
            let uv = self.pos;
            let base_top = vec3(0.102, 0.102, 0.180);    // #1a1a2e
            let base_bot = vec3(0.086, 0.129, 0.243);    // #16213e
            let base = mix(base_top, base_bot, uv.y);

            // --- Specular highlight (upper region) ---
            // Simulates light reflecting off a convex surface
            let highlight_center = vec2(0.5 + self.mouse_offset.x * 0.15,
                                        0.25 + self.mouse_offset.y * 0.08);
            let highlight_d = (uv - highlight_center) / vec2(0.4, 0.2);
            let highlight_dist = length(highlight_d);
            let highlight = exp(-highlight_dist * highlight_dist * 2.0);
            let highlight_color = vec3(1.0, 1.0, 1.0) * highlight * 0.40;

            // --- Secondary reflection (lower region) ---
            let bottom_center = vec2(0.5 - self.mouse_offset.x * 0.08, 0.82);
            let bottom_d = (uv - bottom_center) / vec2(0.3, 0.1);
            let bottom_dist = length(bottom_d);
            let bottom_highlight = exp(-bottom_dist * bottom_dist * 3.0);
            let bottom_color = vec3(1.0, 1.0, 1.0) * bottom_highlight * 0.10;

            // --- Edge vignette ---
            let vignette = 1.0 - pow(dist, 3.0) * 0.4;

            // --- Rim light (subtle metallic edge glow) ---
            let rim = smoothstep(0.85, 1.0, dist) * (1.0 - smoothstep(1.0 - edge, 1.0, dist));
            let rim_color = vec3(0.3, 0.35, 0.5) * rim * 0.6;

            // --- Composite ---
            let color = base * vignette + highlight_color + bottom_color + rim_color;
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
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {
        // Will add transport controls here in Sprint 4
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
