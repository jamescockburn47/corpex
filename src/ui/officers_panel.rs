use crate::app::InvestigationApp;
use crate::ui::styles;

/// Officers panel — shows directors/secretaries for the selected company.
pub fn render(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    let company_number = match &app.selected_company {
        Some(cn) => cn.clone(),
        None => {
            ui.label(egui::RichText::new("Select a company first.").color(styles::DIM_TEXT));
            return;
        }
    };

    let name = app
        .company_profiles
        .get(&company_number)
        .map(|p| p.display_name())
        .unwrap_or_else(|| company_number.clone());

    ui.heading(format!("👤 Officers — {}", name));
    ui.separator();

    let officers = match app.company_officers.get(&company_number) {
        Some(o) => o.clone(),
        None => {
            ui.label("No officer data loaded yet.");
            return;
        }
    };

    // Stats
    let active = officers.iter().filter(|o| o.is_active()).count();
    let resigned = officers.len() - active;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Active: {}", active)).color(styles::ACCENT_GREEN));
        ui.label(egui::RichText::new(format!("Resigned: {}", resigned)).color(styles::DIM_TEXT));
    });
    ui.add_space(8.0);

    egui::ScrollArea::vertical().id_salt("officers_scroll").show(ui, |ui| {
        for officer in &officers {
            let is_active = officer.is_active();
            let name_color = if is_active {
                styles::TEXT_PRIMARY
            } else {
                styles::DIM_TEXT
            };

            egui::Frame::none()
                .fill(styles::BG_CARD)
                .rounding(6.0)
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let icon = if is_active { "👤" } else { "👻" };
                        ui.label(
                            egui::RichText::new(format!("{} {}", icon, officer.display_name()))
                                .strong()
                                .color(name_color),
                        );
                        if let Some(role) = &officer.officer_role {
                            ui.label(
                                egui::RichText::new(role)
                                    .small()
                                    .color(styles::ACCENT),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        if let Some(appointed) = &officer.appointed_on {
                            ui.label(
                                egui::RichText::new(format!("Appointed: {}", appointed))
                                    .small()
                                    .color(styles::DIM_TEXT),
                            );
                        }
                        if let Some(resigned) = &officer.resigned_on {
                            ui.label(
                                egui::RichText::new(format!("Resigned: {}", resigned))
                                    .small()
                                    .color(styles::ACCENT_RED),
                            );
                        }
                        if let Some(occ) = &officer.occupation {
                            ui.label(
                                egui::RichText::new(occ)
                                    .small()
                                    .color(styles::DIM_TEXT),
                            );
                        }
                        if let Some(nat) = &officer.nationality {
                            ui.label(
                                egui::RichText::new(nat)
                                    .small()
                                    .color(styles::DIM_TEXT),
                            );
                        }
                    });
                });
            ui.add_space(4.0);
        }
    });
}
