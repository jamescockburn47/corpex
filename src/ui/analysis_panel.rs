use crate::app::InvestigationApp;
use crate::ui::styles;
use std::cell::RefCell;

/// Analysis panel — AI-powered company analysis with chat interface.
pub fn render(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.heading("🤖 AI Analysis");
    ui.separator();

    // ── No AI configured ─────────────────────────────────────────────
    if !app.config.has_ai() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("No AI Provider Configured")
                    .size(18.0)
                    .color(styles::ACCENT_YELLOW),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    "Go to Settings to add your API key.\n\
                     Supports Anthropic Claude, OpenAI GPT, or any OpenAI-compatible endpoint.",
                )
                .color(styles::DIM_TEXT),
            );
            ui.add_space(12.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("⚙ Open Settings").color(styles::BG_DARK),
                    )
                    .fill(styles::ACCENT)
                    .corner_radius(4.0)
                    .min_size(egui::vec2(140.0, 30.0)),
                )
                .clicked()
            {
                app.show_settings = true;
                app.active_view = super::View::Settings;
            }
        });
        return;
    }

    // ── No company selected ──────────────────────────────────────────
    let company_number = match &app.selected_company {
        Some(cn) => cn.clone(),
        None => {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Select a company first")
                        .size(16.0)
                        .color(styles::DIM_TEXT),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "Use Search to investigate a company, then return here for AI analysis.",
                    )
                    .size(13.0)
                    .color(styles::DIM_TEXT),
                );
            });
            return;
        }
    };

    let name = app
        .company_profiles
        .get(&company_number)
        .map(|p| p.display_name())
        .unwrap_or_else(|| company_number.clone());

    // ── Company header + analysis button ─────────────────────────────
    egui::Frame::new()
        .fill(styles::BG_CARD)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&name)
                            .size(16.0)
                            .strong()
                            .color(styles::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(&company_number)
                            .size(13.0)
                            .color(styles::ACCENT),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let has_analysis = app.ai_analyses.contains_key(&company_number);
                    let btn_label = if has_analysis {
                        "🔄 Re-run Analysis"
                    } else {
                        "🔍 Run AI Analysis"
                    };

                    let enabled = !app.is_loading;
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new(
                                egui::RichText::new(btn_label).color(styles::BG_DARK),
                            )
                            .fill(styles::ACCENT_GREEN)
                            .corner_radius(4.0)
                            .min_size(egui::vec2(150.0, 30.0)),
                        )
                        .clicked()
                    {
                        app.run_ai_analysis(company_number.clone());
                    }

                    // Save button (show when analysis exists)
                    if has_analysis {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("💾 Save Results").color(styles::BG_DARK),
                                )
                                .fill(styles::ACCENT)
                                .corner_radius(4.0)
                                .min_size(egui::vec2(120.0, 30.0)),
                            )
                            .clicked()
                        {
                            app.save_project_name = name.clone();
                            app.show_save_dialog = true;
                            app.save_status_message = None;
                        }
                    }
                });
            });

            // Data availability indicators
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Data:").size(12.0).color(styles::DIM_TEXT));
                data_indicator(
                    ui,
                    "Profile",
                    app.company_profiles.contains_key(&company_number),
                );
                data_indicator(
                    ui,
                    "Officers",
                    app.company_officers.contains_key(&company_number),
                );
                data_indicator(
                    ui,
                    "PSCs",
                    app.company_pscs.contains_key(&company_number),
                );
                data_indicator(
                    ui,
                    "Charges",
                    app.company_charges.contains_key(&company_number),
                );
                data_indicator(
                    ui,
                    "Filings",
                    app.company_filings.contains_key(&company_number),
                );
                // Check for extracted docs by matching filing transaction IDs
                let has_text = app.company_filings
                    .get(&company_number)
                    .map(|filings| filings.iter().any(|f| {
                        f.transaction_id.as_ref()
                            .map_or(false, |tid| app.extracted_texts.contains_key(tid))
                    }))
                    .unwrap_or(false);
                data_indicator(ui, "Docs", has_text);
            });

            // ── Year range selector ──────────────────────────────────
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Analysis period:")
                        .size(12.0)
                        .color(styles::DIM_TEXT),
                );

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("From").size(12.0).color(styles::DIM_TEXT),
                );
                egui::ComboBox::from_id_salt("year_from")
                    .width(65.0)
                    .selected_text(app.analysis_year_from.to_string())
                    .show_ui(ui, |ui| {
                        for year in (2015..=2026).rev() {
                            ui.selectable_value(
                                &mut app.analysis_year_from,
                                year,
                                year.to_string(),
                            );
                        }
                    });

                ui.label(
                    egui::RichText::new("To").size(12.0).color(styles::DIM_TEXT),
                );
                egui::ComboBox::from_id_salt("year_to")
                    .width(65.0)
                    .selected_text(app.analysis_year_to.to_string())
                    .show_ui(ui, |ui| {
                        for year in (2015..=2026).rev() {
                            ui.selectable_value(
                                &mut app.analysis_year_to,
                                year,
                                year.to_string(),
                            );
                        }
                    });

                // Clamp: from cannot exceed to
                if app.analysis_year_from > app.analysis_year_to {
                    app.analysis_year_to = app.analysis_year_from;
                }
            });
        });

    ui.add_space(8.0);

    // ── Conversation area ────────────────────────────────────────────
    let conversation = app
        .ai_conversations
        .get(&company_number)
        .cloned()
        .unwrap_or_default();

    if conversation.is_empty() && !app.ai_analyses.contains_key(&company_number) {
        // No analysis yet — show prompt
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("No analysis yet")
                    .size(14.0)
                    .color(styles::DIM_TEXT),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "Click \"Run AI Analysis\" to send all company data to your AI provider.\n\
                     The AI will analyse the profile, officers, filings, charges, and any extracted text.",
                )
                .size(13.0)
                .color(styles::DIM_TEXT),
            );
        });
    } else {
        // Collect clicked [REF:xxx] links
        let clicked_refs: RefCell<Vec<String>> = RefCell::new(Vec::new());

        // Show conversation as scrollable content
        let available = ui.available_height() - 50.0; // Leave space for input
        egui::ScrollArea::vertical()
            .id_salt("analysis_scroll")
            .max_height(available.max(200.0))
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (role, content) in &conversation {
                    if role == "user" {
                        render_user_message(ui, content);
                    } else {
                        render_analysis_report_with_refs(ui, content, &company_number, &clicked_refs);
                    }
                    ui.add_space(8.0);
                }

                // Show loading indicator
                if app.is_loading {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("AI is analysing...")
                                .size(14.0)
                                .color(styles::DIM_TEXT)
                                .italics(),
                        );
                    });
                }
            });

        // Handle ref clicks — open the document externally
        let refs = clicked_refs.into_inner();
        if let Some(filing_id) = refs.first() {
            // Open Companies House document in browser
            let url = format!(
                "https://find-and-update.company-information.service.gov.uk/company/{}/filing-history/{}/document?format=pdf",
                company_number, filing_id
            );
            #[cfg(not(target_arch = "wasm32"))]
            { let _ = open::that(&url); }
            #[cfg(target_arch = "wasm32")]
            { let _ = url; } // WASM: links handled by egui's built-in hyperlink support
        }
    }

    // ── Chat input ───────────────────────────────────────────────────
    if app.ai_analyses.contains_key(&company_number) || !conversation.is_empty() {
        ui.separator();
        ui.horizontal(|ui| {
            let resp = ui.add_sized(
                [ui.available_width() - 80.0, 28.0],
                egui::TextEdit::singleline(&mut app.ai_chat_input)
                    .hint_text("Ask a follow-up question...")
                    .desired_width(ui.available_width() - 80.0),
            );

            let can_send = !app.ai_chat_input.trim().is_empty() && !app.is_loading;
            let enter_pressed =
                resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if (enter_pressed
                || ui
                    .add_enabled(
                        can_send,
                        egui::Button::new(
                            egui::RichText::new("Send").color(styles::BG_DARK),
                        )
                        .fill(styles::ACCENT)
                        .corner_radius(4.0)
                        .min_size(egui::vec2(60.0, 28.0)),
                    )
                    .clicked())
                && can_send
            {
                let msg = app.ai_chat_input.trim().to_string();
                app.ai_chat_input.clear();
                app.send_ai_chat(company_number.clone(), msg);
            }
        });
    }

    // ── Save Dialog ──────────────────────────────────────────────────
    let ctx = ui.ctx().clone();
    render_save_dialog(app, &ctx, &company_number, &name);
}

// ═══════════════════════════════════════════════════════════════════════
//  RENDERING HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn render_user_message(ui: &mut egui::Ui, content: &str) {
    egui::Frame::new()
        .fill(styles::BG_HOVER)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("You")
                    .strong()
                    .size(12.0)
                    .color(styles::ACCENT),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(content)
                    .size(14.0)
                    .color(styles::TEXT_PRIMARY),
            );
        });
}

/// Parse AI response into sections and render as a visual report with cards, capturing [REF:xxx] clicks.
fn render_analysis_report_with_refs(
    ui: &mut egui::Ui,
    content: &str,
    company_number: &str,
    clicked_refs: &RefCell<Vec<String>>,
) {
    let sections = parse_sections(content);

    if sections.len() > 1 {
        let first = &sections[0];
        if !first.body.trim().is_empty() {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(20, 35, 45))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::same(16))
                .stroke(egui::Stroke::new(1.0, styles::ACCENT))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let heading = if first.heading.is_empty() {
                        "Executive Summary"
                    } else {
                        &first.heading
                    };
                    ui.label(
                        egui::RichText::new(heading)
                            .size(16.0)
                            .strong()
                            .color(styles::ACCENT),
                    );
                    ui.add_space(8.0);
                    render_body_text_with_refs(ui, &first.body, 14.0, styles::TEXT_PRIMARY, company_number, clicked_refs);
                });
            ui.add_space(10.0);
        }

        for section in sections.iter().skip(1) {
            render_section_card_with_refs(ui, section, company_number, clicked_refs);
            ui.add_space(6.0);
        }
    } else {
        egui::Frame::new()
            .fill(styles::BG_CARD)
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    egui::RichText::new("AI Analyst")
                        .strong()
                        .size(12.0)
                        .color(styles::ACCENT_GREEN),
                );
                ui.add_space(6.0);
                render_body_text_with_refs(ui, content, 13.5, styles::TEXT_SECONDARY, company_number, clicked_refs);
            });
    }
}

/// Render a section card with [REF:xxx] link support.
fn render_section_card_with_refs(
    ui: &mut egui::Ui,
    section: &Section,
    company_number: &str,
    clicked_refs: &RefCell<Vec<String>>,
) {
    let accent = section_accent(&section.heading);

    egui::Frame::new()
        .fill(styles::BG_CARD)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::same(14))
        .stroke(egui::Stroke::new(0.5, egui::Color32::from_gray(50)))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            if !section.heading.is_empty() {
                ui.label(
                    egui::RichText::new(&section.heading)
                        .size(15.0)
                        .strong()
                        .color(accent),
                );
                ui.add_space(6.0);
            }

            render_body_text_with_refs(ui, &section.body, 13.5, styles::TEXT_SECONDARY, company_number, clicked_refs);
        });
}

/// Render body text with [REF:xxx] link parsing.
fn render_body_text_with_refs(
    ui: &mut egui::Ui,
    text: &str,
    base_size: f32,
    base_color: egui::Color32,
    _company_number: &str,
    clicked_refs: &RefCell<Vec<String>>,
) {
    let ref_re = regex::Regex::new(r"\[REF:([^\]]+)\]").unwrap();

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            ui.add_space(4.0);
            continue;
        }

        // Separator lines
        if trimmed.starts_with("---") || trimmed.starts_with("===") || trimmed.chars().all(|c| c == '-' || c == '=') && trimmed.len() > 2 {
            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);
            continue;
        }

        let clean = strip_markdown(trimmed);

        // Sub-headings
        if trimmed.starts_with("### ")
            || (trimmed.starts_with("**") && trimmed.ends_with("**"))
            || (trimmed.starts_with("**") && trimmed.contains(":**"))
        {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(&clean)
                    .size(base_size + 0.5)
                    .strong()
                    .color(styles::TEXT_PRIMARY),
            );
            ui.add_space(2.0);
            continue;
        }

        // Check if line contains [REF:xxx]
        if ref_re.is_match(&clean) {
            // Split line into text and ref segments
            render_line_with_refs(ui, &clean, base_size, base_color, &ref_re, clicked_refs);
            continue;
        }

        // Bullet points
        if trimmed.starts_with("- ") || trimmed.starts_with("• ") || trimmed.starts_with("· ") {
            let bullet_text = strip_markdown(&trimmed[2..]);
            if ref_re.is_match(&bullet_text) {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("•  ").size(base_size).color(base_color));
                    render_line_with_refs(ui, &bullet_text, base_size, base_color, &ref_re, clicked_refs);
                });
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("•  {}", bullet_text))
                            .size(base_size)
                            .color(base_color),
                    );
                });
            }
            continue;
        }

        // Indented bullets
        if (trimmed.starts_with("  -") || trimmed.starts_with("  •"))
            && trimmed.len() > 3
        {
            let bullet_text = strip_markdown(
                trimmed
                    .trim_start_matches(|c: char| c == ' ' || c == '-' || c == '•' || c == '·'),
            );
            ui.horizontal_wrapped(|ui| {
                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new(format!("◦  {}", bullet_text))
                        .size(base_size - 0.5)
                        .color(styles::DIM_TEXT),
                );
            });
            continue;
        }

        // Table rows
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            if trimmed.contains("---") {
                continue;
            }
            let cols: Vec<&str> = trimmed
                .trim_matches('|')
                .split('|')
                .map(|s| s.trim())
                .collect();
            let row_text = cols
                .iter()
                .map(|c| strip_markdown(c))
                .collect::<Vec<_>>()
                .join("    ");
            ui.horizontal_wrapped(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(&row_text)
                        .size(base_size - 0.5)
                        .color(base_color)
                        .family(egui::FontFamily::Monospace),
                );
            });
            continue;
        }

        // Regular paragraph text
        ui.label(
            egui::RichText::new(&clean)
                .size(base_size)
                .color(base_color),
        );
    }
}

/// Render a single line that contains [REF:xxx] markers, splitting into text + clickable links.
fn render_line_with_refs(
    ui: &mut egui::Ui,
    line: &str,
    base_size: f32,
    base_color: egui::Color32,
    ref_re: &regex::Regex,
    clicked_refs: &RefCell<Vec<String>>,
) {
    ui.horizontal_wrapped(|ui| {
        let mut last_end = 0;
        for m in ref_re.find_iter(line) {
            // Text before the ref
            let before = &line[last_end..m.start()];
            if !before.is_empty() {
                ui.label(
                    egui::RichText::new(before)
                        .size(base_size)
                        .color(base_color),
                );
            }
            // Extract filing_id from [REF:xxx]
            if let Some(caps) = ref_re.captures(m.as_str()) {
                let filing_id = caps.get(1).unwrap().as_str();
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!("📄 {}", filing_id))
                                .size(base_size - 1.0)
                                .color(styles::ACCENT)
                                .underline(),
                        )
                        .frame(false)
                    )
                    .on_hover_text(format!("Open original filing {} on Companies House", filing_id))
                    .clicked()
                {
                    clicked_refs.borrow_mut().push(filing_id.to_string());
                }
            }
            last_end = m.end();
        }
        // Remaining text after last ref
        let after = &line[last_end..];
        if !after.is_empty() {
            ui.label(
                egui::RichText::new(after)
                    .size(base_size)
                    .color(base_color),
            );
        }
    });
}

// ═══════════════════════════════════════════════════════════════════════
//  SECTION PARSER
// ═══════════════════════════════════════════════════════════════════════

struct Section {
    heading: String,
    body: String,
}

/// Parse AI response text into sections split by headings.
fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading = String::new();
    let mut current_body = String::new();

    for line in text.lines() {
        let trimmed = line.trim();

        if is_section_heading(trimmed) {
            // Save previous section
            if !current_heading.is_empty() || !current_body.trim().is_empty() {
                sections.push(Section {
                    heading: current_heading.clone(),
                    body: current_body.trim().to_string(),
                });
            }
            current_heading = strip_heading(trimmed);
            current_body.clear();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    // Push final section
    if !current_heading.is_empty() || !current_body.trim().is_empty() {
        sections.push(Section {
            heading: current_heading,
            body: current_body.trim().to_string(),
        });
    }

    sections
}

/// Detect if a line is a major section heading.
fn is_section_heading(line: &str) -> bool {
    let trimmed = line.trim();

    // Markdown headings: # or ##
    if trimmed.starts_with("# ") || trimmed.starts_with("## ") {
        return true;
    }

    // Numbered sections like "1. COMPANY OVERVIEW" or "1. Company Overview"
    let after_num = trimmed
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim();
    if after_num != trimmed && after_num.len() > 3 && !after_num.starts_with('-') {
        // Check if it looks like a heading (mostly uppercase, or title case)
        let alpha: Vec<char> = after_num.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha.len() >= 3 {
            let upper = alpha.iter().filter(|c| c.is_uppercase()).count();
            if upper as f32 / alpha.len() as f32 > 0.5 {
                return true;
            }
        }
    }

    // ALL CAPS lines (short) like "EXECUTIVE SUMMARY", "FINANCIAL HEALTH"
    if trimmed.len() > 4 && trimmed.len() < 60 {
        let alpha: Vec<char> = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha.len() >= 3 {
            let upper = alpha.iter().filter(|c| c.is_uppercase()).count();
            if upper as f32 / alpha.len() as f32 > 0.7 {
                return true;
            }
        }
    }

    false
}

/// Strip heading markers (##, numbering) from a heading line.
fn strip_heading(line: &str) -> String {
    let mut s = line.trim().to_string();

    // Remove # markers
    while s.starts_with('#') {
        s = s.trim_start_matches('#').to_string();
    }
    s = s.trim().to_string();

    // Remove numbering like "1. " or "2."
    let after_num = s
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim()
        .to_string();
    if after_num != s && after_num.len() > 2 {
        s = after_num;
    }

    // Remove bold markers
    s = s.replace("**", "");

    // Remove trailing em-dash descriptions for cleaner headings
    // e.g. "COMPANY OVERVIEW — what this company does" → "COMPANY OVERVIEW"
    if let Some(pos) = s.find('—') {
        let before = s[..pos].trim();
        if before.len() > 3 {
            s = before.to_string();
        }
    }

    s.trim().to_string()
}

/// Choose accent colour based on section heading content.
fn section_accent(heading: &str) -> egui::Color32 {
    let h = heading.to_uppercase();
    if h.contains("RISK") || h.contains("INSOLVENCY") || h.contains("WARNING") {
        styles::ACCENT_RED
    } else if h.contains("FINANCIAL") || h.contains("HEALTH") || h.contains("ACCOUNTS") {
        styles::ACCENT_YELLOW
    } else if h.contains("RECOMMENDATION") || h.contains("NEXT") || h.contains("FURTHER") {
        styles::ACCENT_GREEN
    } else if h.contains("NETWORK") || h.contains("CONNECTION") {
        styles::ACCENT
    } else {
        styles::TEXT_PRIMARY
    }
}

/// Strip common markdown formatting from text.
fn strip_markdown(text: &str) -> String {
    let mut s = text.to_string();

    // Remove heading markers
    while s.starts_with('#') {
        s = s.trim_start_matches('#').to_string();
    }
    s = s.trim().to_string();

    // Remove bold markers **text**
    s = s.replace("**", "");

    // Remove inline code `text`
    s = s.replace('`', "");

    s.trim().to_string()
}

// ── Helper: data availability indicator ──────────────────────────────

fn data_indicator(ui: &mut egui::Ui, label: &str, available: bool) {
    let (icon, color) = if available {
        ("✓", styles::ACCENT_GREEN)
    } else {
        ("○", styles::DIM_TEXT)
    };
    ui.label(
        egui::RichText::new(format!("{} {}", icon, label))
            .size(12.0)
            .color(color),
    );
}

// ═══════════════════════════════════════════════════════════════════════
//  SAVE DIALOG
// ═══════════════════════════════════════════════════════════════════════

fn render_save_dialog(
    app: &mut InvestigationApp,
    ctx: &egui::Context,
    company_number: &str,
    company_name: &str,
) {
    if !app.show_save_dialog {
        return;
    }

    let mut still_open = true;
    egui::Window::new("💾 Save Investigation Results")
        .open(&mut still_open)
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("Save results for {} ({})", company_name, company_number))
                    .size(14.0)
                    .color(styles::TEXT_PRIMARY),
            );
            ui.add_space(12.0);

            // Existing projects
            let projects = crate::export::list_projects();
            if !projects.is_empty() {
                ui.label(
                    egui::RichText::new("Existing projects:")
                        .size(12.0)
                        .color(styles::DIM_TEXT),
                );
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for project in &projects {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!("📁 {}", project))
                                            .size(13.0)
                                            .color(if app.save_project_name == *project {
                                                styles::ACCENT
                                            } else {
                                                styles::TEXT_SECONDARY
                                            }),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                app.save_project_name = project.clone();
                            }
                        }
                    });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
            }

            // Project name input
            ui.label(
                egui::RichText::new("Project / folder name:")
                    .size(12.0)
                    .color(styles::DIM_TEXT),
            );
            ui.add_space(4.0);
            ui.add_sized(
                [ui.available_width(), 28.0],
                egui::TextEdit::singleline(&mut app.save_project_name)
                    .hint_text("Enter project name or leave for company name"),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!(
                    "Save to: ~/Corpex Investigations/{}/",
                    if app.save_project_name.trim().is_empty() {
                        company_name
                    } else {
                        app.save_project_name.trim()
                    }
                ))
                .size(11.0)
                .color(styles::DIM_TEXT)
                .italics(),
            );

            ui.add_space(12.0);

            // Status message
            if let Some(msg) = &app.save_status_message {
                let color = if msg.starts_with("✓") {
                    styles::ACCENT_GREEN
                } else {
                    styles::ACCENT_RED
                };
                ui.label(
                    egui::RichText::new(msg.as_str())
                        .size(13.0)
                        .color(color),
                );
                ui.add_space(8.0);
            }

            // Buttons
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("💾 Save").color(styles::BG_DARK),
                        )
                        .fill(styles::ACCENT_GREEN)
                        .corner_radius(4.0)
                        .min_size(egui::vec2(100.0, 30.0)),
                    )
                    .clicked()
                {
                    let project = if app.save_project_name.trim().is_empty() {
                        company_name.to_string()
                    } else {
                        app.save_project_name.trim().to_string()
                    };

                    // Collect data for export
                    let profile_json = app.company_profiles.get(company_number)
                        .map(|p| format!("{:#?}", p));

                    let ai_analysis = app.ai_analyses.get(company_number).cloned();

                    let chat_history: Option<Vec<(String, String)>> = app
                        .ai_conversations
                        .get(company_number)
                        .cloned();

                    // Collect extracted texts for this company
                    let mut company_texts = std::collections::HashMap::new();
                    let mut filing_descs = std::collections::HashMap::new();
                    if let Some(filings) = app.company_filings.get(company_number) {
                        for f in filings {
                            if let Some(tid) = &f.transaction_id {
                                if let Some(text) = app.extracted_texts.get(tid) {
                                    company_texts.insert(tid.clone(), text.clone());
                                    if let Some(desc) = &f.description {
                                        filing_descs.insert(tid.clone(), desc.clone());
                                    }
                                }
                            }
                        }
                    }

                    let chat_refs: Option<Vec<(String, String)>> = chat_history.as_ref().map(|v| {
                        v.iter().map(|(r, c)| (r.clone(), c.clone())).collect()
                    });
                    let chat_slice: Option<&[(String, String)]> = chat_refs.as_deref();

                    match crate::export::export_company(
                        &project,
                        company_number,
                        company_name,
                        profile_json.as_deref(),
                        ai_analysis.as_deref(),
                        chat_slice,
                        &company_texts,
                        &filing_descs,
                    ) {
                        Ok(path) => {
                            app.save_status_message = Some(format!(
                                "✓ Saved to {}",
                                path.display()
                            ));
                        }
                        Err(e) => {
                            app.save_status_message = Some(format!("✗ Error: {}", e));
                        }
                    }
                }

                if ui
                    .add(
                        egui::Button::new("Cancel")
                            .corner_radius(4.0)
                            .min_size(egui::vec2(80.0, 30.0)),
                    )
                    .clicked()
                {
                    app.show_save_dialog = false;
                    app.save_status_message = None;
                }

                // Open folder button (show after successful save)
                if let Some(msg) = &app.save_status_message {
                    if msg.starts_with("✓") {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("📂 Open Folder").color(styles::BG_DARK),
                                )
                                .fill(styles::ACCENT)
                                .corner_radius(4.0)
                                .min_size(egui::vec2(110.0, 30.0)),
                            )
                            .clicked()
                        {
                            let project = if app.save_project_name.trim().is_empty() {
                                company_name.to_string()
                            } else {
                                app.save_project_name.trim().to_string()
                            };
                            let path = crate::export::export_root().join(&project);
                            #[cfg(not(target_arch = "wasm32"))]
                            { let _ = open::that(path); }
                            #[cfg(target_arch = "wasm32")]
                            { let _ = path; }
                        }
                    }
                }
            });
        });

    if !still_open {
        app.show_save_dialog = false;
        app.save_status_message = None;
    }
}
