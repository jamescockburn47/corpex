use crate::app::InvestigationApp;
use crate::ui::styles;

/// Settings panel — rendered inline in the central area (not as a floating overlay).
pub fn render(app: &mut InvestigationApp, ctx: &egui::Context) {
    render_inline(app, ctx);
}

/// Settings rendered as a full panel inside the central area.
pub fn render_inline(app: &mut InvestigationApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(24.0);

            ui.label(
                egui::RichText::new("⚙  Settings")
                    .size(24.0)
                    .strong()
                    .color(styles::TEXT_PRIMARY),
            );
            ui.add_space(16.0);

            // ── Companies House API ────────────────────────────────
            settings_section(ui, "Companies House API", |ui| {
                if app.config.has_ch_key() {
                    ui.label(
                        egui::RichText::new("✓  API key loaded from .env")
                            .size(14.0)
                            .color(styles::ACCENT_GREEN),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("✗  No CH_API_KEY found in .env")
                            .size(14.0)
                            .color(styles::ACCENT_RED),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Add CH_API_KEY=your_key to a .env file in the application directory.",
                        )
                        .size(13.0)
                        .color(styles::DIM_TEXT),
                    );
                }
            });

            ui.add_space(16.0);

            // ── AI Provider ────────────────────────────────────────
            settings_section(ui, "AI Provider (BYOK)", |ui| {
                ui.add_space(4.0);

                settings_row(ui, "Provider", 90.0, |ui| {
                    egui::ComboBox::from_id_salt("ai_provider")
                        .width(250.0)
                        .selected_text(&app.settings_ai_provider)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.settings_ai_provider,
                                "anthropic".to_string(),
                                "Anthropic Claude",
                            );
                            ui.selectable_value(
                                &mut app.settings_ai_provider,
                                "openai".to_string(),
                                "OpenAI GPT",
                            );
                            ui.selectable_value(
                                &mut app.settings_ai_provider,
                                "custom".to_string(),
                                "Custom (OpenAI-compatible)",
                            );
                        });
                });

                ui.add_space(6.0);

                settings_row(ui, "API Key", 90.0, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.settings_ai_key)
                            .password(true)
                            .desired_width(350.0)
                            .hint_text("Enter your API key"),
                    );
                });

                ui.add_space(6.0);

                let hint = match app.settings_ai_provider.as_str() {
                    "anthropic" => "claude-haiku-4-5",
                    "openai" => "gpt-4o-mini",
                    _ => "model-name",
                };

                settings_row(ui, "Model", 90.0, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.settings_ai_model)
                            .desired_width(350.0)
                            .hint_text(hint),
                    );
                });

                if app.settings_ai_provider == "custom" {
                    ui.add_space(6.0);
                    settings_row(ui, "Base URL", 90.0, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut app.settings_ai_base_url)
                                .desired_width(350.0)
                                .hint_text("https://api.example.com/v1"),
                        );
                    });
                }
            });

            ui.add_space(16.0);

            // ── Kanon 2 ────────────────────────────────────────────
            settings_section(ui, "Kanon 2 (Isaacus Legal AI)", |ui| {
                ui.add_space(4.0);

                settings_row(ui, "API Key", 90.0, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.settings_kanon2_key)
                            .password(true)
                            .desired_width(350.0)
                            .hint_text("iuak_v1_..."),
                    );
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "Used for legal document embeddings, classification, and extraction.",
                    )
                    .size(13.0)
                    .color(styles::DIM_TEXT),
                );
            });

            ui.add_space(16.0);

            // ── OCR Config ─────────────────────────────────────────
            settings_section(ui, "OCR Configuration", |ui| {
                ui.add_space(4.0);
                ui.radio_value(
                    &mut app.config.ocr_mode,
                    crate::config::OcrMode::NativeOnly,
                    egui::RichText::new("Native PDF text only (no OCR)").size(14.0),
                );
                ui.add_space(2.0);
                ui.radio_value(
                    &mut app.config.ocr_mode,
                    crate::config::OcrMode::WithOcrs,
                    egui::RichText::new("Native + ocrs ML fallback (recommended)").size(14.0),
                );
                ui.add_space(2.0);
                ui.radio_value(
                    &mut app.config.ocr_mode,
                    crate::config::OcrMode::WithDocling,
                    egui::RichText::new("Native + Docling sidecar (requires Python)").size(14.0),
                );
            });

            ui.add_space(24.0);

            // ── Apply / Cancel ─────────────────────────────────────
            ui.horizontal(|ui| {
                let apply = ui.add(
                    egui::Button::new(
                        egui::RichText::new("💾  Apply Settings")
                            .size(15.0)
                            .color(styles::BG_DARK),
                    )
                    .fill(styles::ACCENT)
                    .corner_radius(6.0)
                    .min_size(egui::vec2(160.0, 36.0)),
                );
                if apply.clicked() {
                    app.apply_settings();
                    app.show_settings = false;
                    app.active_view = super::View::Welcome;
                }

                ui.add_space(8.0);

                let cancel = ui.add(
                    egui::Button::new(egui::RichText::new("Cancel").size(14.0))
                        .corner_radius(6.0)
                        .min_size(egui::vec2(100.0, 36.0)),
                );
                if cancel.clicked() {
                    app.show_settings = false;
                    app.active_view = super::View::Welcome;
                }
            });

            ui.add_space(24.0);
        });
    });
}

// ── Helpers ──────────────────────────────────────────────────────────

fn settings_section(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(styles::BG_CARD)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(title)
                    .size(16.0)
                    .strong()
                    .color(styles::TEXT_PRIMARY),
            );
            ui.add_space(8.0);
            content(ui);
        });
}

fn settings_row(
    ui: &mut egui::Ui,
    label: &str,
    label_width: f32,
    content: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(label_width, 20.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(14.0)
                        .color(styles::TEXT_SECONDARY),
                );
            },
        );
        ui.add_space(8.0);
        content(ui);
    });
}
