/// Visual theme constants.

pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(88, 166, 255);     // Blue
pub const ACCENT_GREEN: egui::Color32 = egui::Color32::from_rgb(63, 185, 80);
pub const ACCENT_RED: egui::Color32 = egui::Color32::from_rgb(248, 81, 73);
pub const ACCENT_YELLOW: egui::Color32 = egui::Color32::from_rgb(210, 153, 34);
pub const ACCENT_ORANGE: egui::Color32 = egui::Color32::from_rgb(219, 171, 9);

pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(13, 17, 23);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(22, 27, 34);
pub const BG_CARD: egui::Color32 = egui::Color32::from_rgb(33, 38, 45);
pub const BG_HOVER: egui::Color32 = egui::Color32::from_rgb(48, 54, 61);

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 237, 243);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(175, 184, 193);
pub const DIM_TEXT: egui::Color32 = egui::Color32::from_rgb(139, 148, 158);
pub const STATUS_TEXT: egui::Color32 = egui::Color32::from_rgb(110, 118, 129);

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.4);
    visuals.hyperlink_color = ACCENT;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, DIM_TEXT);
    visuals.extreme_bg_color = BG_DARK;
    visuals.faint_bg_color = BG_CARD;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_style(style);
}

/// Status badge for company status.
pub fn status_color(status: &str) -> egui::Color32 {
    match status {
        "active" => ACCENT_GREEN,
        "dissolved" => ACCENT_RED,
        "liquidation" => ACCENT_RED,
        "administration" => ACCENT_ORANGE,
        "insolvency-proceedings" => ACCENT_RED,
        _ => DIM_TEXT,
    }
}

/// Risk badge.
pub fn risk_badge(ui: &mut egui::Ui, level: crate::investigation::network::RiskLevel) {
    let (icon, color) = match level {
        crate::investigation::network::RiskLevel::High => ("🔴", ACCENT_RED),
        crate::investigation::network::RiskLevel::Medium => ("🟡", ACCENT_YELLOW),
        crate::investigation::network::RiskLevel::Low => ("🟢", ACCENT_GREEN),
    };
    ui.label(egui::RichText::new(icon).color(color));
}
