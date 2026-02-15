use crate::app::InvestigationApp;
use crate::ui::styles;

/// Welcome / Landing page — shown on first open and when navigating "Home".
pub fn render(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().id_salt("landing_scroll").show(ui, |ui| {
        ui.set_width(ui.available_width());
        let max_w = 700.0_f32.min(ui.available_width() - 32.0);

        ui.add_space(32.0);

        // ── Hero ─────────────────────────────────────────────────────
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("🔎 Corpex")
                    .size(36.0)
                    .strong()
                    .color(styles::ACCENT),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Corporate Intelligence Tool")
                    .size(18.0)
                    .color(styles::DIM_TEXT),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    "Investigate UK companies through Companies House data.\n\
                     Map corporate networks, trace ownership, and flag risks.",
                )
                .size(14.0)
                .color(styles::TEXT_SECONDARY),
            );
        });

        ui.add_space(28.0);

        // ── Setup Status ─────────────────────────────────────────────
        let ch_ok = app.config.has_ch_key();
        let ai_ok = app.config.has_ai();

        card(ui, |ui| {
            ui.label(
                egui::RichText::new("⚡ Setup Status")
                    .size(16.0)
                    .strong()
                    .color(styles::TEXT_PRIMARY),
            );
            ui.add_space(10.0);

            status_row(
                ui,
                ch_ok,
                "Companies House API",
                if ch_ok {
                    "Loaded ✓"
                } else {
                    "Add CH_API_KEY to .env file"
                },
            );
            if !ch_ok {
                ui.indent("ch_help", |ui| {
                    ui.label(
                        egui::RichText::new(
                            "Get a free key from developer.company-information.service.gov.uk",
                        )
                        .size(13.0)
                        .color(styles::DIM_TEXT),
                    );
                });
            }

            ui.add_space(4.0);

            status_row(
                ui,
                ai_ok,
                "AI Analysis (optional)",
                if ai_ok {
                    "Configured ✓"
                } else {
                    "Add AI key in Settings for deeper analysis"
                },
            );
            if !ai_ok {
                ui.indent("ai_help", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Supports Anthropic Claude & OpenAI.")
                                .size(13.0)
                                .color(styles::DIM_TEXT),
                        );
                        if ui.small_button("⚙ Configure").clicked() {
                            app.show_settings = true;
                            app.active_view = super::View::Settings;
                        }
                    });
                });
            }
        });

        ui.add_space(24.0);

        // ── How To Search ────────────────────────────────────────────
        section_header(ui, "🔍 How to Search");

        card(ui, |ui| {
            subsection(ui, "Company Search");
            ui.add_space(4.0);
            tip_row(ui, "By name:", "\"Barclays\", \"Tesco PLC\", \"Apple UK\"");
            tip_row(
                ui,
                "By number:",
                "\"00102498\" or \"SC123456\" — goes directly to profile",
            );
            tip_row(
                ui,
                "Partial:",
                "\"legal\" matches all companies containing that word",
            );

            ui.add_space(14.0);

            subsection(ui, "Director / Officer Search");
            ui.add_space(4.0);
            tip_row(
                ui,
                "Switch mode:",
                "Click \"👤 Director Search\" on the Search page. The sidebar search bar shows which mode is active.",
            );
            tip_row(
                ui,
                "By name:",
                "\"John Smith\" to find all officers with that name",
            );
            tip_row(
                ui,
                "View appointments:",
                "Click any director to see every company they serve",
            );
            tip_row(
                ui,
                "Investigate:",
                "One click to pull full data on any linked company",
            );
            tip_row(
                ui,
                "Bulk investigate:",
                "\"Investigate All Active\" to load all current companies at once",
            );

            ui.add_space(14.0);

            ui.label(
                egui::RichText::new("💡 Search Strategy Tips")
                    .size(14.0)
                    .strong()
                    .color(styles::ACCENT_YELLOW),
            );
            ui.add_space(6.0);
            bullet(ui, "Start with a company search, then explore its directors");
            bullet(
                ui,
                "From a director's appointments, investigate their other companies",
            );
            bullet(
                ui,
                "This builds a network graph of connected entities automatically",
            );
            bullet(
                ui,
                "Use the Network view to visualise shared directors & ownership chains",
            );
        });

        ui.add_space(24.0);

        // ── What Data is Available ───────────────────────────────────
        section_header(ui, "📊 Data Available from Companies House");

        // Vertical stack of data category cards (not squished horizontal)
        data_card(
            ui,
            "🏢 Company Profile",
            &[
                "Registered name, number, status (active / dissolved / liquidation)",
                "Incorporation & dissolution dates",
                "Registered office address",
                "Company type (Ltd, PLC, LLP, etc)",
                "SIC codes (industry classification)",
                "Accounts overdue / annual return status",
            ],
        );
        ui.add_space(8.0);

        data_card(
            ui,
            "👤 Officers & Directors",
            &[
                "Current & resigned directors, secretaries, LLP members",
                "Date of birth (month/year), nationality",
                "Correspondence address",
                "Appointment & resignation dates",
                "Cross-reference: all companies a person serves",
            ],
        );
        ui.add_space(8.0);

        data_card(
            ui,
            "📄 Filing History",
            &[
                "Annual accounts (full, abbreviated, micro)",
                "Confirmation statements",
                "Director appointments & resignations",
                "Charges, mortgages, resolutions",
                "Document text extraction (iXBRL, XHTML, PDF)",
            ],
        );
        ui.add_space(8.0);

        data_card(
            ui,
            "🔗 Persons with Significant Control",
            &[
                "Individuals or entities with >25% shares / voting rights",
                "Nature of control (shares, voting, right to appoint)",
                "Nationality, country of residence",
            ],
        );
        ui.add_space(8.0);

        data_card(
            ui,
            "💰 Charges & Mortgages",
            &[
                "Secured lending (charge holder, created / delivered dates)",
                "Charge status (outstanding / satisfied / part-satisfied)",
                "Particulars of the charge",
            ],
        );
        ui.add_space(8.0);

        data_card(
            ui,
            "⚠ Insolvency",
            &[
                "Insolvency cases (CVL, compulsory, administration, etc)",
                "Insolvency practitioners (name, address, role)",
                "Key dates (winding-up order, appointment of IP, etc)",
            ],
        );

        ui.add_space(24.0);

        // ── Analysis & Intelligence ────────────────────────────────────
        section_header(ui, "🤖 Analysis & Intelligence");

        card(ui, |ui| {
            analysis_item(
                ui,
                "🚩 Risk Detection (automatic)",
                "Flags overdue accounts, insolvency history, dissolved status, \
                 and other compliance red flags from raw CH data.",
            );
            ui.add_space(10.0);
            analysis_item(
                ui,
                "📄 Document Text Extraction",
                "Extracts text from iXBRL accounts, XHTML filings, and PDFs. \
                 Preserves table structures, financial data, and key-value pairs.",
            );
            ui.add_space(10.0);
            analysis_item(
                ui,
                "🤖 AI-Powered Analysis (with API key)",
                "Send company data to Claude or GPT for deeper investigation. \
                 Summarise financials, identify unusual patterns, and ask follow-up questions. \
                 Supports Anthropic Claude (Haiku, Sonnet, Opus) and OpenAI (GPT-4o).",
            );
            ui.add_space(10.0);
            analysis_item(
                ui,
                "💰 AI Cost Tracking",
                "Every AI call shows input/output token counts and cost at current \
                 model rates. Session-wide cumulative cost is displayed in the bottom status \
                 bar. Rates: Claude Haiku 4.5 ($0.80/M input, $4.00/M output), \
                 Sonnet ($3/$15), GPT-4o ($2.50/$10).",
            );
            ui.add_space(10.0);
            analysis_item(
                ui,
                "🏢 Corporate Group Detection",
                "Automatically detects parent/subsidiary relationships from Corporate PSC data. \
                 When a company has a corporate Person with Significant Control, the parent \
                 company is identified and you can discover all sibling subsidiaries.",
            );
            ui.add_space(10.0);
            analysis_item(
                ui,
                "🧠 Group-Level AI Analysis",
                "Analyse an entire corporate group in one AI query. If the parent files \
                 consolidated (group) accounts, these are used as the primary data source \
                 to save tokens. Individual subsidiary data is capped at 3k chars each \
                 to prevent token explosion.",
            );
            ui.add_space(10.0);
            analysis_item(
                ui,
                "🕸 Network Intelligence",
                "Automatically maps shared directors, PSC ownership chains, \
                 and cross-company connections for visualisation.",
            );
        });

        ui.add_space(24.0);

        // ── Privacy ──────────────────────────────────────────────────
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔒").size(20.0));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Privacy First")
                            .size(15.0)
                            .strong()
                            .color(styles::TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "All data stays on your machine. No server, no telemetry. \
                             API calls go directly to Companies House and your chosen AI provider. \
                             Your API keys are stored locally and never shared.",
                        )
                        .size(13.0)
                        .color(styles::DIM_TEXT),
                    );
                });
            });
        });

        ui.add_space(24.0);

        // ── Quick Action ─────────────────────────────────────────────
        if ch_ok {
            ui.vertical_centered(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("🔍  Start Searching")
                                .size(16.0)
                                .color(styles::BG_DARK),
                        )
                        .fill(styles::ACCENT)
                        .corner_radius(8.0)
                        .min_size(egui::vec2(220.0, 40.0)),
                    )
                    .clicked()
                {
                    app.active_view = super::View::Search;
                }
            });
        }

        ui.add_space(32.0);
    });
}

// ── Helper functions ─────────────────────────────────────────────────

fn card(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(styles::BG_CARD)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            content(ui);
        });
}

fn data_card(ui: &mut egui::Ui, title: &str, items: &[&str]) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new(title)
                .size(14.0)
                .strong()
                .color(styles::ACCENT),
        );
        ui.add_space(6.0);
        for item in items {
            bullet(ui, item);
        }
    });
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(18.0)
            .strong()
            .color(styles::TEXT_PRIMARY),
    );
    ui.add_space(10.0);
}

fn subsection(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(14.0)
            .strong()
            .color(styles::ACCENT),
    );
}

fn status_row(ui: &mut egui::Ui, ok: bool, label: &str, detail: &str) {
    ui.horizontal(|ui| {
        let (icon, color) = if ok {
            ("✓", styles::ACCENT_GREEN)
        } else {
            ("○", styles::DIM_TEXT)
        };
        ui.label(egui::RichText::new(icon).size(14.0).color(color).strong());
        ui.label(
            egui::RichText::new(format!("{}:", label))
                .size(14.0)
                .strong()
                .color(styles::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(detail).size(14.0).color(if ok {
                styles::ACCENT_GREEN
            } else {
                styles::TEXT_SECONDARY
            }),
        );
    });
}

fn tip_row(ui: &mut egui::Ui, label: &str, example: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(14.0)
                .strong()
                .color(styles::DIM_TEXT),
        );
        ui.label(egui::RichText::new(example).size(14.0).color(styles::TEXT_SECONDARY));
    });
}

fn bullet(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(format!("  •  {}", text))
            .size(13.0)
            .color(styles::TEXT_SECONDARY),
    );
}

fn analysis_item(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(14.0)
            .strong()
            .color(styles::TEXT_PRIMARY),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(description)
            .size(13.0)
            .color(styles::TEXT_SECONDARY),
    );
}
