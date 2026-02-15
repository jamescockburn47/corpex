use crate::app::InvestigationApp;
use crate::ui::styles;

/// Filings panel — browse and inspect filing history for the selected company.
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

    ui.heading(format!("📄 Filings — {}", name));
    ui.separator();

    let filings = match app.company_filings.get(&company_number) {
        Some(f) => f.clone(),
        None => {
            ui.label("No filings loaded. Click 'Deep Dive' on the Company tab to fetch filings.");
            if ui.button("Fetch Filings").clicked() {
                app.is_loading = true;
                app.investigate_company(company_number);
            }
            return;
        }
    };

    if filings.is_empty() {
        ui.label("No filings found.");
        return;
    }

    // Clone what we need from app to avoid borrow conflicts
    let extracted_texts = app.extracted_texts.clone();
    let filing_summaries = app.filing_summaries.clone();
    let has_ai = app.config.has_ai();
    let is_loading = app.is_loading;

    // Count cached docs
    let cached_count = filings.iter().filter(|f| {
        f.transaction_id.as_ref().map_or(false, |tid| extracted_texts.contains_key(tid))
    }).count();

    if cached_count > 0 {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("📚 {} documents extracted & cached", cached_count))
                    .small()
                    .color(styles::ACCENT_GREEN),
            );
        });
        ui.add_space(4.0);
    }

    // Deferred actions to avoid borrow conflicts
    let mut summarise_action: Option<(String, String, String)> = None;
    let mut extract_action: Option<(String, String, String)> = None;

    egui::ScrollArea::vertical().id_salt("filings_scroll").show(ui, |ui| {
        for filing in &filings {
            let date = filing.date.as_deref().unwrap_or("-");
            let category = filing.category.as_deref().unwrap_or("-");
            let desc = filing.description.as_deref().unwrap_or("No description");
            let ftype = filing.filing_type.as_deref().unwrap_or("-");

            let cat_color = match category {
                "accounts" => styles::ACCENT_GREEN,
                "confirmation-statement" => styles::ACCENT,
                "officers" => styles::ACCENT_YELLOW,
                "capital" => styles::ACCENT_ORANGE,
                "insolvency" => styles::ACCENT_RED,
                "charges" => styles::ACCENT_RED,
                _ => styles::DIM_TEXT,
            };

            egui::Frame::none()
                .fill(styles::BG_CARD)
                .rounding(6.0)
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(date)
                                .small()
                                .color(styles::DIM_TEXT),
                        );
                        ui.label(
                            egui::RichText::new(format!("[{}]", category))
                                .small()
                                .strong()
                                .color(cat_color),
                        );
                        ui.label(
                            egui::RichText::new(ftype)
                                .small()
                                .color(styles::DIM_TEXT),
                        );
                    });
                    ui.label(egui::RichText::new(desc).color(styles::TEXT_PRIMARY));

                    if let Some(tid) = &filing.transaction_id {
                        if let Some(text) = extracted_texts.get(tid) {
                            // We have extracted text — show status and AI button
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("✓ Cached")
                                        .small()
                                        .color(styles::ACCENT_GREEN),
                                );

                                if has_ai {
                                    if filing_summaries.contains_key(tid) {
                                        ui.label(
                                            egui::RichText::new("✓ Summarised")
                                                .small()
                                                .color(styles::ACCENT),
                                        );
                                    } else if !is_loading && summarise_action.is_none() {
                                        if ui.small_button("🤖 Summarise").clicked() {
                                            summarise_action = Some((
                                                tid.clone(),
                                                desc.to_string(),
                                                text.clone(),
                                            ));
                                        }
                                    }
                                }
                            });

                            // Show AI summary if available
                            if let Some(summary) = filing_summaries.get(tid) {
                                egui::Frame::none()
                                    .fill(styles::BG_HOVER)
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::same(6))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new("🤖 AI Summary")
                                                .small()
                                                .strong()
                                                .color(styles::ACCENT),
                                        );
                                        ui.label(
                                            egui::RichText::new(summary)
                                                .small()
                                                .color(styles::TEXT_PRIMARY),
                                        );
                                    });
                            }

                            // Collapsible raw text
                            ui.collapsing("📝 Extracted Text", |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(text)
                                                .small()
                                                .color(styles::TEXT_SECONDARY),
                                        );
                                    });
                            });
                        } else if let Some(links) = &filing.links {
                            if let Some(doc_meta) = &links.document_metadata {
                                if extract_action.is_none() {
                                    if ui.small_button("📥 Extract Text").clicked() {
                                        extract_action = Some((
                                            company_number.clone(),
                                            tid.clone(),
                                            doc_meta.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                });
            ui.add_space(4.0);
        }
    });

    // Execute deferred actions
    if let Some((tid, desc, text)) = summarise_action {
        app.summarise_filing(tid, desc, text);
    }
    if let Some((cn, tid, doc_meta)) = extract_action {
        app.is_loading = true;
        app.extract_filing_text(cn, tid, doc_meta);
    }
}
