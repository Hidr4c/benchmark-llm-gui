use eframe::egui;

pub struct TourSpotlight;

impl TourSpotlight {
    pub fn draw_spotlight(
        ctx: &egui::Context,
        target_rect: Option<egui::Rect>,
        accent_color: egui::Color32,
        is_in_transition: bool,
        callout_label: &str,
    ) {
        let dim_painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("tour_spotlight_dim_layer"),
        ));
        let fg_painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("tour_spotlight_fg_layer"),
        ));

        let screen = ctx.screen_rect();
        let dim_color = if is_in_transition {
            egui::Color32::from_black_alpha(20)
        } else {
            egui::Color32::from_black_alpha(130)
        };

        if let Some(raw_target) = target_rect {
            let target = raw_target.expand(6.0);

            // 1. Draw 4 dark rectangles leaving a spotlight cutout around target
            if target.top() > screen.top() {
                dim_painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), screen.top()),
                        egui::pos2(screen.right(), target.top()),
                    ),
                    0.0,
                    dim_color,
                );
            }
            if target.bottom() < screen.bottom() {
                dim_painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), target.bottom()),
                        egui::pos2(screen.right(), screen.bottom()),
                    ),
                    0.0,
                    dim_color,
                );
            }
            if target.left() > screen.left() {
                dim_painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), target.top()),
                        egui::pos2(target.left(), target.bottom()),
                    ),
                    0.0,
                    dim_color,
                );
            }
            if target.right() < screen.right() {
                dim_painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(target.right(), target.top()),
                        egui::pos2(screen.right(), target.bottom()),
                    ),
                    0.0,
                    dim_color,
                );
            }

            // 2. Animated Pulsing Neon Halo around the target element (drawn in foreground)
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

            fg_painter.rect_stroke(target, 8.0, outer_glow);
            fg_painter.rect_stroke(target, 8.0, inner_stroke);

            // 3. Animated Bouncing Pointing Finger with Callout Badge (drawn in foreground)
            // Auto-hide the pointing hand and callout tag whenever any dropdown menu is open
            // or when the user hovers their mouse directly over the target to interact with it.
            let is_popup_open = ctx.memory(|m| m.any_popup_open());
            let is_hovered = ctx.input(|i| i.pointer.hover_pos().map_or(false, |pos| target.expand(6.0).contains(pos)));

            if !is_popup_open && !is_hovered {
                let bounce = (time * 6.0).sin() as f32 * 5.0;
                draw_pointing_hand(&fg_painter, screen, target, accent_color, callout_label, bounce);
            }
        } else if !is_in_transition {
            dim_painter.rect_filled(screen, 0.0, egui::Color32::from_black_alpha(110));
        }
    }

    pub fn compute_card_position(
        screen: egui::Rect,
        target_rect: Option<egui::Rect>,
        card_width: f32,
        card_height: f32,
    ) -> egui::Pos2 {
        let avail_w = (screen.width() - 48.0).max(360.0);
        let actual_card_w = card_width.min(avail_w);
        let center_x = screen.left() + (screen.width() - actual_card_w) * 0.5;

        let y = if let Some(target) = target_rect {
            // Anti-masking rule:
            // If the spotlight target is in the upper 55% of the screen,
            // place the card firmly at the BOTTOM of the screen so the entire
            // table and filter bar remain 100% visible!
            if target.center().y < screen.top() + screen.height() * 0.55 {
                (screen.bottom() - card_height - 20.0).max(screen.top() + 60.0)
            } else {
                // If the target is in the lower half of the screen (e.g. detail drawer),
                // place the card at the TOP of the screen so the bottom panel remains 100% visible!
                screen.top() + 75.0
            }
        } else {
            (screen.bottom() - card_height - 20.0).max(screen.top() + 60.0)
        };

        egui::pos2(center_x, y)
    }
}

fn draw_pointing_hand(
    painter: &egui::Painter,
    screen: egui::Rect,
    target: egui::Rect,
    accent: egui::Color32,
    callout_text: &str,
    bounce: f32,
) {
    let is_pointing_up = target.center().y < screen.top() + screen.height() * 0.55;
    let hand_icon = if is_pointing_up { "👆" } else { "👇" };

    let center_x = target.center().x.clamp(screen.left() + 150.0, screen.right() - 150.0);
    let hand_y = if is_pointing_up {
        target.bottom() + 18.0 + bounce
    } else {
        target.top() - 18.0 + bounce
    };
    let hand_pos = egui::pos2(center_x, hand_y);

    // High-contrast background disk with glowing accent ring
    let bg_color = egui::Color32::from_rgba_unmultiplied(12, 17, 28, 250);
    painter.circle_filled(hand_pos, 20.0, bg_color);
    painter.circle_stroke(hand_pos, 20.0, egui::Stroke::new(2.5, accent));

    let font_id = egui::FontId::proportional(22.0);
    painter.text(
        hand_pos,
        egui::Align2::CENTER_CENTER,
        hand_icon,
        font_id,
        egui::Color32::WHITE,
    );

    // Floating callout bubble
    if !callout_text.is_empty() {
        let text_font = egui::FontId::proportional(12.5);
        let badge_y = if is_pointing_up {
            hand_pos.y + 28.0
        } else {
            hand_pos.y - 28.0
        };
        let tag_text = format!("  {callout_text}  ");
        let approx_w = (tag_text.len() as f32 * 7.5).max(120.0);
        let tag_rect = egui::Rect::from_center_size(egui::pos2(center_x, badge_y), egui::vec2(approx_w, 24.0));

        painter.rect_filled(tag_rect, 6.0, egui::Color32::from_rgba_unmultiplied(10, 16, 26, 250));
        painter.rect_stroke(tag_rect, 6.0, egui::Stroke::new(1.5, accent));
        painter.text(
            tag_rect.center(),
            egui::Align2::CENTER_CENTER,
            tag_text,
            text_font,
            egui::Color32::WHITE,
        );
    }
}
