use crate::app::InvestigationApp;
use crate::ui::styles;
use crate::investigation::network::RiskLevel;

/// Dashboard — overview of the current investigation.
pub fn render(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    ui.heading("Investigation Dashboard");
    ui.add_space(8.0);

    if app.network.node_count() == 0 {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(
                    egui::RichText::new("🔎 Start an Investigation")
                        .size(28.0)
                        .color(styles::ACCENT),
                );
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(
                        "Type a company name or number in the search bar above.\nThe tool will map corporate networks, identify connections, and flag risks.",
                    )
                    .size(16.0)
                    .color(styles::DIM_TEXT),
                );
                ui.add_space(20.0);
                if !app.config.has_ch_key() {
                    ui.label(
                        egui::RichText::new("⚠ No Companies House API key found. Add CH_API_KEY to your .env file.")
                            .color(styles::ACCENT_YELLOW),
                    );
                }
                if !app.config.has_ai() {
                    ui.label(
                        egui::RichText::new("💡 Configure an AI provider in Settings for deeper analysis.")
                            .color(styles::DIM_TEXT),
                    );
                }
            });
        });
        return;
    }

    // Stats row
    ui.horizontal(|ui| {
        stat_card(ui, "Companies", &app.network.node_count().to_string(), styles::ACCENT);
        stat_card(ui, "Relationships", &app.network.edge_count().to_string(), styles::ACCENT);

        // Count risk levels
        let mut high = 0;
        let mut medium = 0;
        for node in app.network.graph.node_weights() {
            match node.risk_level {
                RiskLevel::High => high += 1,
                RiskLevel::Medium => medium += 1,
                RiskLevel::Low => {}
            }
        }
        stat_card(ui, "High Risk", &high.to_string(), styles::ACCENT_RED);
        stat_card(ui, "Medium Risk", &medium.to_string(), styles::ACCENT_YELLOW);
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // Company list
    ui.heading("Investigated Companies");
    egui::ScrollArea::vertical().id_salt("dashboard_scroll").show(ui, |ui| {
        for node in app.network.graph.node_weights() {
            ui.horizontal(|ui| {
                styles::risk_badge(ui, node.risk_level);
                let label = format!("{} ({})", node.company_name, node.company_number);
                let resp = ui.selectable_label(
                    app.selected_company.as_deref() == Some(&node.company_number),
                    egui::RichText::new(&label).color(styles::TEXT_PRIMARY),
                );
                if resp.clicked() {
                    app.selected_company = Some(node.company_number.clone());
                    app.active_view = super::View::Company;
                }

                ui.label(
                    egui::RichText::new(&node.status)
                        .small()
                        .color(styles::status_color(&node.status)),
                );
            });

            // Show risk signals
            if !node.risk_signals.is_empty() {
                ui.indent(node.company_number.as_str(), |ui| {
                    for sig in &node.risk_signals {
                        ui.label(egui::RichText::new(format!("  ⚠ {}", sig)).small().color(styles::ACCENT_YELLOW));
                    }
                });
            }
        }
    });
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(styles::BG_CARD)
        .rounding(8.0)
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(value).size(24.0).strong().color(color));
                ui.label(egui::RichText::new(label).small().color(styles::DIM_TEXT));
            });
        });
}
