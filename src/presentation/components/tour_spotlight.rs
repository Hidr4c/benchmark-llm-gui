use eframe::egui;

pub struct TourSpotlight;

impl TourSpotlight {
    pub fn draw_spotlight(
        ctx: &egui::Context,
        target_rect: Option<egui::Rect>,
        accent_color: egui::Color32,
    ) {
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("tour_spotlight_dim_layer"),
        ));

        let screen = ctx.screen_rect();
        let dim_color = egui::Color32::from_black_alpha(150);

        if let Some(raw_target) = target_rect {
            let target = raw_target.expand(6.0);

            // 1. Draw 4 dark rectangles leaving a spotlight cutout around target
            // Top rect
            if target.top() > screen.top() {
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), screen.top()),
                        egui::pos2(screen.right(), target.top()),
                    ),
                    0.0,
                    dim_color,
                );
            }
            // Bottom rect
            if target.bottom() < screen.bottom() {
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), target.bottom()),
                        egui::pos2(screen.right(), screen.bottom()),
                    ),
                    0.0,
                    dim_color,
                );
            }
            // Left rect
            if target.left() > screen.left() {
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), target.top()),
                        egui::pos2(target.left(), target.bottom()),
                    ),
                    0.0,
                    dim_color,
                );
            }
            // Right rect
            if target.right() < screen.right() {
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(target.right(), target.top()),
                        egui::pos2(screen.right(), target.bottom()),
                    ),
                    0.0,
                    dim_color,
                );
            }

            // 2. Animated Pulsing Neon Halo around the target element
            let time = ctx.input(|i| i.time);
            let pulse = (time * 5.0).sin().abs() as f32;
            let glow_alpha = (100.0 + pulse * 130.0) as u8;

            let outer_glow = egui::Stroke::new(
                7.0 + pulse * 4.0,
                egui::Color32::from_rgba_unmultiplied(
                    accent_color.r(),
                    accent_color.g(),
                    accent_color.b(),
                    glow_alpha / 3,
                ),
            );
            let inner_stroke = egui::Stroke::new(
                2.5 + pulse * 1.5,
                egui::Color32::from_rgba_unmultiplied(
                    accent_color.r(),
                    accent_color.g(),
                    accent_color.b(),
                    glow_alpha,
                ),
            );

            painter.rect_stroke(target, 8.0, outer_glow);
            painter.rect_stroke(target, 8.0, inner_stroke);

            // 3. Animated Bouncing Pointing Finger (Natural visual beacon)
            let bounce = (time * 6.0).sin() as f32 * 5.0;
            let pointer_center_x = target.center().x.clamp(80.0, screen.right() - 80.0);

            if target.bottom() + 50.0 > screen.bottom() {
                let hand_pos = egui::pos2(pointer_center_x, target.top() - 20.0 + bounce);
                draw_pointing_hand(&painter, hand_pos, accent_color, true);
            } else {
                let hand_pos = egui::pos2(pointer_center_x, target.bottom() + 20.0 + bounce);
                draw_pointing_hand(&painter, hand_pos, accent_color, false);
            }
        } else {
            painter.rect_filled(screen, 0.0, egui::Color32::from_black_alpha(110));
        }
    }

    pub fn compute_card_position(
        screen: egui::Rect,
        target_rect: Option<egui::Rect>,
        card_width: f32,
        card_height: f32,
        is_guide_open: bool,
    ) -> egui::Pos2 {
        let safe_left = screen.left() + 24.0;
        let safe_right = if is_guide_open {
            (screen.right() - 290.0).max(safe_left + 360.0)
        } else {
            screen.right() - 24.0
        };

        let avail_w = (safe_right - safe_left).max(360.0);
        let actual_card_w = card_width.min(avail_w);

        let (left_x, top_y) = if let Some(target) = target_rect {
            let ideal_left = target.center().x - (actual_card_w * 0.5);
            let max_left = (safe_right - actual_card_w).max(safe_left);
            let x = ideal_left.clamp(safe_left, max_left);

            let y = if target.bottom() + card_height + 55.0 < screen.bottom() {
                target.bottom() + 45.0
            } else if target.top() - card_height - 35.0 > screen.top() {
                target.top() - card_height - 35.0
            } else {
                (screen.bottom() - card_height - 24.0).max(screen.top() + 60.0)
            };
            (x, y)
        } else {
            let x = safe_left + (avail_w - actual_card_w) * 0.5;
            let y = (screen.bottom() - card_height - 30.0).max(screen.top() + 60.0);
            (x, y)
        };

        egui::pos2(left_x, top_y)
    }
}

fn draw_pointing_hand(
    painter: &egui::Painter,
    center_pos: egui::Pos2,
    accent: egui::Color32,
    is_pointing_down: bool,
) {
    let hand_icon = if is_pointing_down { "👇" } else { "👆" };

    // High-contrast background disk with glowing accent ring
    let bg_color = egui::Color32::from_rgba_unmultiplied(12, 17, 28, 240);
    painter.circle_filled(center_pos, 19.0, bg_color);
    painter.circle_stroke(center_pos, 19.0, egui::Stroke::new(2.0, accent));

    let font_id = egui::FontId::proportional(22.0);
    painter.text(
        center_pos,
        egui::Align2::CENTER_CENTER,
        hand_icon,
        font_id,
        egui::Color32::WHITE,
    );
}
