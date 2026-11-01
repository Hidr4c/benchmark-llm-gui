use eframe::egui::Color32;

pub struct ThemeTokens;

impl ThemeTokens {
    // Background tokens
    pub const BG_APP: Color32 = Color32::from_rgb(15, 20, 28);
    pub const BG_PANEL: Color32 = Color32::from_rgb(22, 28, 40);
    pub const BG_CARD: Color32 = Color32::from_rgb(28, 36, 52);

    // Text tokens
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 244, 252);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 178, 204);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(110, 130, 155);

    // Accents & Statuses
    pub const PRIMARY: Color32 = Color32::from_rgb(90, 165, 255);
    pub const SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
    pub const WARNING: Color32 = Color32::from_rgb(243, 156, 18);
    pub const DANGER: Color32 = Color32::from_rgb(235, 87, 87);
    pub const CYAN_EDGE: Color32 = Color32::from_rgb(0, 200, 230);
    #[allow(dead_code)]
    pub const PURPLE_ACCENT: Color32 = Color32::from_rgb(175, 110, 245);
    #[allow(dead_code)]
    pub const GOLD_5090: Color32 = Color32::from_rgb(255, 180, 50);
}
