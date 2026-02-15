use crate::app::InvestigationApp;
use crate::ui::styles;

/// Company detail panel — profile, address, SIC codes, accounts info for the selected company.
pub fn render(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    let company_number = match &app.selected_company {
        Some(cn) => cn.clone(),
        None => {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Select a company from the Dashboard or Search to view details.")
                        .color(styles::DIM_TEXT),
                );
            });
            return;
        }
    };

    let profile = match app.company_profiles.get(&company_number) {
        Some(p) => p.clone(),
        None => {
            ui.label("Loading profile...");
            ui.spinner();
            return;
        }
    };

    // Header
    ui.horizontal(|ui| {
        if let Some(node) = app.network.get_node(&company_number) {
            styles::risk_badge(ui, node.risk_level);
        }
        ui.heading(
            egui::RichText::new(profile.display_name())
                .color(styles::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(format!(" ({})", profile.number()))
                .color(styles::ACCENT),
        );
        let status = profile.company_status.as_deref().unwrap_or("unknown");
        ui.label(
            egui::RichText::new(status.to_uppercase())
                .small()
                .strong()
                .color(styles::status_color(status)),
        );
    });

    ui.separator();

    egui::ScrollArea::vertical().id_salt("company_scroll").show(ui, |ui| {
        // Actions
        ui.horizontal(|ui| {
            if ui.button("🔍 Deep Dive (fetch all data)").clicked() {
                app.is_loading = true;
                app.investigate_company(company_number.clone());
            }
            if ui.button("🕸 View in Network").clicked() {
                app.active_view = super::View::Network;
            }
            if ui.button("📄 View Filings").clicked() {
                app.active_view = super::View::Filings;
            }
            if app.config.has_ai() {
                let has_analysis = app.ai_analyses.contains_key(&company_number);
                let btn = if has_analysis { "🤖 View Analysis" } else { "🤖 Run AI Analysis" };
                if ui.button(btn).clicked() {
                    if has_analysis {
                        app.push_view(super::View::Analysis);
                    } else {
                        app.run_ai_analysis(company_number.clone());
                    }
                }
            }
        });

        // AI analysis summary card (if analysis has been run)
        let analysis_preview = app.ai_analyses.get(&company_number).map(|a| {
            let preview: String = a.chars().take(500).collect();
            let truncated = a.len() > 500;
            if truncated { format!("{}...", preview) } else { preview }
        });
        let mut open_analysis = false;
        if let Some(preview) = analysis_preview {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(styles::BG_HOVER)
                .rounding(6.0)
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🤖 AI Analysis Summary")
                                .strong()
                                .color(styles::ACCENT),
                        );
                        if ui.small_button("View full →").clicked() {
                            open_analysis = true;
                        }
                    });
                    ui.label(
                        egui::RichText::new(&preview)
                            .small()
                            .color(styles::TEXT_SECONDARY),
                    );
                });
        }
        if open_analysis {
            app.push_view(super::View::Analysis);
        }

        ui.add_space(8.0);

        // Two-column layout
        ui.columns(2, |cols| {
            // Left column: Company details
            cols[0].group(|ui| {
                ui.label(egui::RichText::new("Company Details").strong());
                ui.add_space(4.0);
                detail_row(ui, "Type", profile.company_type.as_deref().unwrap_or("-"));
                detail_row(ui, "Incorporated", profile.date_of_creation.as_deref().unwrap_or("-"));
                if let Some(cessation) = &profile.date_of_cessation {
                    detail_row(ui, "Dissolved", cessation);
                }
                detail_row(
                    ui,
                    "Jurisdiction",
                    profile.jurisdiction.as_deref().unwrap_or("-"),
                );
                if let Some(sic) = &profile.sic_codes {
                    detail_row(ui, "SIC Codes", &sic.join(", "));
                }
            });

            // Right column: Address + compliance
            cols[1].group(|ui| {
                ui.label(egui::RichText::new("Registered Office").strong());
                ui.add_space(4.0);
                if let Some(addr) = &profile.registered_office_address {
                    ui.label(addr.one_line());
                } else {
                    ui.label("-");
                }
                ui.add_space(12.0);

                ui.label(egui::RichText::new("Compliance").strong());
                ui.add_space(4.0);
                if let Some(accts) = &profile.accounts {
                    let overdue_icon = if accts.overdue == Some(true) { "🔴" } else { "🟢" };
                    detail_row(
                        ui,
                        "Accounts",
                        &format!(
                            "{} Due: {}",
                            overdue_icon,
                            accts.next_due.as_deref().unwrap_or("-")
                        ),
                    );
                }
                if let Some(cs) = &profile.confirmation_statement {
                    let overdue_icon = if cs.overdue == Some(true) { "🔴" } else { "🟢" };
                    detail_row(
                        ui,
                        "Confirmation",
                        &format!(
                            "{} Due: {}",
                            overdue_icon,
                            cs.next_due.as_deref().unwrap_or("-")
                        ),
                    );
                }
                detail_row(
                    ui,
                    "Charges",
                    if profile.has_charges == Some(true) { "Yes" } else { "No" },
                );
                detail_row(
                    ui,
                    "Insolvency",
                    if profile.has_insolvency_history == Some(true) { "Yes ⚠" } else { "No" },
                );
            });
        });

        ui.add_space(12.0);

        // Risk signals
        if let Some(node) = app.network.get_node(&company_number) {
            if !node.risk_signals.is_empty() {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("⚠ Risk Signals").strong().color(styles::ACCENT_YELLOW));
                    for sig in &node.risk_signals {
                        ui.label(egui::RichText::new(format!("  • {}", sig)).color(styles::ACCENT_YELLOW));
                    }
                });
            }
        }

        // ── Group Structure Card ──────────────────────────────────────
        {
            use crate::investigation::group::GroupRole;
            let cn_clone = company_number.clone();

            if let Some(gi) = app.group_info.get(&company_number).cloned() {
                ui.add_space(8.0);
                egui::Frame::none()
                    .fill(styles::BG_CARD)
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(12))
                    .stroke(egui::Stroke::new(1.0, styles::ACCENT))
                    .show(ui, |ui| {
                        // Header
                        let role_label = match gi.role {
                            GroupRole::Subsidiary => "🏢 SUBSIDIARY",
                            GroupRole::Parent => "🏛 PARENT / HOLDING",
                            GroupRole::Unknown => "🏢 GROUP STRUCTURE",
                        };
                        ui.label(egui::RichText::new(role_label).strong().size(14.0).color(styles::ACCENT));
                        ui.add_space(4.0);

                        // Show parent if subsidiary
                        if let Some(parent) = &gi.parent {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Parent:").color(styles::DIM_TEXT));
                                ui.label(egui::RichText::new(&parent.name).strong().color(styles::TEXT_PRIMARY));
                                ui.label(egui::RichText::new(format!("({})", parent.company_number)).small().color(styles::ACCENT));
                                let pn = parent.company_number.clone();
                                if ui.small_button("🔍 Investigate").clicked() {
                                    app.is_loading = true;
                                    app.investigate_company(pn);
                                    app.push_view(super::View::Company);
                                }
                            });
                        }

                        // Consolidated accounts indicator
                        if gi.has_consolidated_accounts {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📊 Parent files consolidated (group) accounts")
                                    .color(egui::Color32::from_rgb(100, 200, 100)));
                            });
                        }

                        // Subsidiaries list
                        if !gi.subsidiaries.is_empty() {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(format!("Subsidiaries ({})", gi.subsidiaries.len()))
                                .strong().color(styles::TEXT_PRIMARY));
                            for sub in &gi.subsidiaries {
                                ui.horizontal(|ui| {
                                    let status_icon = match sub.status.as_str() {
                                        "active" => "🟢",
                                        "dissolved" => "🔴",
                                        _ => "⚪",
                                    };
                                    ui.label(egui::RichText::new(status_icon));
                                    ui.label(egui::RichText::new(&sub.company_name).color(styles::TEXT_PRIMARY));
                                    ui.label(egui::RichText::new(format!("({})", sub.company_number)).small().color(styles::ACCENT));
                                    let sn = sub.company_number.clone();
                                    if ui.small_button("🔍").on_hover_text("Investigate this subsidiary").clicked() {
                                        app.is_loading = true;
                                        app.investigate_company(sn);
                                        app.push_view(super::View::Company);
                                    }
                                });
                            }
                        }

                        // Discover button
                        ui.add_space(4.0);
                        if gi.subsidiaries.is_empty() {
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("🔎 Discover Group Structure").color(styles::BG_DARK)
                                )
                                .fill(styles::ACCENT)
                                .rounding(4.0)
                            ).clicked() {
                                app.discover_group(cn_clone);
                            }
                        } else {
                            // Offer group analysis
                            ui.horizontal(|ui| {
                                if ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("🔄 Re-scan Group").color(styles::TEXT_PRIMARY)
                                    )
                                    .rounding(4.0)
                                ).clicked() {
                                    app.discover_group(cn_clone.clone());
                                }
                                // Investigate all active subsidiaries
                                let active_subs: Vec<String> = gi.subsidiaries.iter()
                                    .filter(|s| s.status == "active")
                                    .map(|s| s.company_number.clone())
                                    .collect();
                                if !active_subs.is_empty() {
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(format!("🔍 Investigate All {} Active", active_subs.len()))
                                                .color(styles::BG_DARK)
                                        )
                                        .fill(styles::ACCENT)
                                        .rounding(4.0)
                                    ).clicked() {
                                        for sub_cn in active_subs {
                                            app.investigate_company(sub_cn);
                                        }
                                        app.push_view(super::View::Dashboard);
                                    }
                                }
                            });
                            // Group analysis button
                            let cn_for_analysis = cn_clone.clone();
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("🧠 Analyse Entire Group").color(styles::BG_DARK)
                                )
                                .fill(egui::Color32::from_rgb(120, 80, 220))
                                .rounding(4.0)
                            ).on_hover_text("Run AI analysis across the whole corporate group").clicked() {
                                app.run_group_analysis(cn_for_analysis);
                            }
                        }
                    });
            }
        }

        // PSC summary
        let pscs_clone = app.company_pscs.get(&company_number).cloned();
        if let Some(pscs) = pscs_clone {
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.label(egui::RichText::new("Persons with Significant Control").strong());
                for psc in &pscs {
                    if !psc.is_active() {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        let kind_icon = if psc.is_corporate() { "🏢" } else { "👤" };
                        let name = psc.name.as_deref().unwrap_or("Unknown");
                        ui.label(format!("{} {}", kind_icon, name));
                        if let Some(natures) = &psc.natures_of_control {
                            for n in natures {
                                ui.label(egui::RichText::new(n).small().color(styles::DIM_TEXT));
                            }
                        }
                    });
                    if psc.is_corporate() {
                        if let Some(reg) = psc.registration_number() {
                            let reg_owned = reg.to_string();
                            ui.indent(&reg_owned, |ui| {
                                if ui
                                    .small_button(format!("🔍 Investigate {}", reg_owned))
                                    .clicked()
                                {
                                    app.is_loading = true;
                                    app.investigate_company(reg_owned.clone());
                                }
                            });
                        }
                    }
                }
            });
        }
    });
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}:", label)).color(styles::DIM_TEXT));
        ui.label(egui::RichText::new(value).color(styles::TEXT_PRIMARY));
    });
}
