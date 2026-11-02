use eframe::egui::IconData;

/// Generates a sleek 32x32 RGBA icon representing a futuristic AI benchmark chip with cyan/gold bars.
pub fn generate_app_icon() -> IconData {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let in_chip = x >= 2 && x <= 29 && y >= 2 && y <= 29;
            let corner = (x < 5 || x > 26) && (y < 5 || y > 26);
            let corner_dist = if x < 5 && y < 5 {
                (4.0 - x as f32).hypot(4.0 - y as f32)
            } else if x > 26 && y < 5 {
                (x as f32 - 27.0).hypot(4.0 - y as f32)
            } else if x < 5 && y > 26 {
                (4.0 - x as f32).hypot(y as f32 - 27.0)
            } else if x > 26 && y > 26 {
                (x as f32 - 27.0).hypot(y as f32 - 27.0)
            } else {
                0.0
            };

            let is_base = in_chip && (!corner || corner_dist <= 3.2);

            if !is_base {
                // Transparent border area
                rgba.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }

            // Dark space navy background (#0F1522)
            let mut r = 15u8;
            let mut g = 21u8;
            let mut b = 34u8;
            let a = 255u8;

            let is_border = x == 2 || x == 29 || y == 2 || y == 29
                || (corner && corner_dist >= 1.8 && corner_dist <= 3.2);

            if is_border {
                // Vibrant Cyan border
                r = 0; g = 200; b = 230;
            } else {
                // Benchmark graphic: 4 ascending performance bars
                let in_bar1 = (x >= 6 && x <= 8) && (y >= 19 && y <= 24);
                let in_bar2 = (x >= 11 && x <= 13) && (y >= 14 && y <= 24);
                let in_bar3 = (x >= 16 && x <= 18) && (y >= 10 && y <= 24);
                let in_bar4 = (x >= 21 && x <= 23) && (y >= 6 && y <= 24);

                if in_bar1 {
                    r = 0; g = 160; b = 210; // Deep cyan
                } else if in_bar2 {
                    r = 0; g = 210; b = 255; // Electric cyan
                } else if in_bar3 {
                    r = 255; g = 210; b = 60; // Yellow-gold
                } else if in_bar4 {
                    r = 255; g = 180; b = 0; // Peak flagship gold
                } else {
                    // Ascending trend line connecting the peaks
                    let target_y = 24.0 - (x as f32 - 6.0) * 1.15;
                    if x >= 6 && x <= 23 && (y as f32 - target_y).abs() <= 0.8 {
                        r = 255; g = 255; b = 255; // Pure white trendline
                    }
                }
            }

            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }

    IconData { rgba, width, height }
}
