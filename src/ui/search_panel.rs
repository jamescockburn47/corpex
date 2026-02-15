use crate::app::{InvestigationApp, SearchMode};
use crate::ui::styles;

/// Full search page view — shown when the user navigates to "Search" in the sidebar.
pub fn render_search(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    ui.add_space(20.0);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("🔍 Search Companies House")
                .size(22.0)
                .strong()
                .color(styles::TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Search by company name, number, or director name")
                .color(styles::DIM_TEXT),
        );
    });

    ui.add_space(12.0);

    // ── Search mode toggle ───────────────────────────────────────────
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            let company_btn = ui.selectable_label(
                app.search_mode == SearchMode::Company,
                egui::RichText::new("🏢 Company Search").size(14.0),
            );
            let officer_btn = ui.selectable_label(
                app.search_mode == SearchMode::Officer,
                egui::RichText::new("👤 Director Search").size(14.0),
            );
            if company_btn.clicked() {
                app.search_mode = SearchMode::Company;
            }
            if officer_btn.clicked() {
                app.search_mode = SearchMode::Officer;
            }
        });
    });

    ui.add_space(12.0);

    let hint = match app.search_mode {
        SearchMode::Company => "e.g. \"Acme Ltd\" or \"12345678\"",
        SearchMode::Officer => "e.g. \"John Smith\" or \"Jane Doe\"",
    };

    // ── Search input ─────────────────────────────────────────────────
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            let search_resp = ui.add_sized(
                [400.0, 32.0],
                egui::TextEdit::singleline(&mut app.search_query)
                    .hint_text(hint)
                    .desired_width(400.0),
            );
            let btn_label = match app.search_mode {
                SearchMode::Company => "Search Companies",
                SearchMode::Officer => "Search Directors",
            };
            if (search_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(btn_label).color(styles::BG_DARK),
                        )
                        .fill(styles::ACCENT)
                        .rounding(4.0)
                        .min_size(egui::vec2(90.0, 32.0)),
                    )
                    .clicked()
            {
                if !app.search_query.trim().is_empty() {
                    let query = app.search_query.trim().to_string();
                    app.is_loading = true;

                    match app.search_mode {
                        SearchMode::Company => {
                            // Clear previous results
                            app.search_results.clear();
                            app.officer_search_results.clear();
                            // Direct navigation for company numbers
                            // Company numbers: all digits, or 2-letter prefix + digits
                            let is_number = query.len() >= 6
                                && query.len() <= 8
                                && (query.chars().all(|c| c.is_ascii_digit())
                                    || (query.len() >= 3
                                        && query[..2].chars().all(|c| c.is_ascii_alphabetic())
                                        && query[2..].chars().all(|c| c.is_ascii_digit())));

                            if is_number {
                                let padded = format!("{:0>8}", query);
                                app.selected_company = Some(padded.clone());
                                app.investigate_company(padded);
                                app.push_view(super::View::Company);
                            } else {
                                app.search_companies(query);
                            }
                        }
                        SearchMode::Officer => {
                            // Clear previous results
                            app.search_results.clear();
                            app.officer_search_results.clear();
                            app.search_officers(query);
                        }
                    }
                }
            }
        });
    });

    ui.add_space(16.0);

    // ── Results ──────────────────────────────────────────────────────
    match app.search_mode {
        SearchMode::Company => render_company_results(app, ui),
        SearchMode::Officer => {
            // If we have selected an officer's appointments, show them
            if app.selected_officer_appointments.is_some() {
                render_officer_appointments(app, ui);
            } else {
                render_officer_results(app, ui);
            }
        }
    }
}

/// Render company search results.
fn render_company_results(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    if !app.search_results.is_empty() {
        ui.separator();
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("{} companies found", app.search_results.len()))
                .small()
                .color(styles::DIM_TEXT),
        );
        ui.add_space(4.0);

        egui::ScrollArea::vertical().id_salt("search_scroll").show(ui, |ui| {
            for result in &app.search_results.clone() {
                let cn = result.company_number.as_deref().unwrap_or("?");
                let name = result.title.as_deref().unwrap_or("Unknown");
                let status = result.company_status.as_deref().unwrap_or("?");

                egui::Frame::none()
                    .fill(styles::BG_CARD)
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(12))
                    .stroke(egui::Stroke::new(1.0, styles::BG_HOVER))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(name)
                                        .strong()
                                        .size(14.0)
                                        .color(styles::TEXT_PRIMARY),
                                );
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(cn)
                                            .small()
                                            .color(styles::ACCENT),
                                    );
                                    ui.label(
                                        egui::RichText::new(status)
                                            .small()
                                            .color(styles::status_color(status)),
                                    );
                                    if let Some(addr) = &result.address_snippet {
                                        ui.label(
                                            egui::RichText::new(addr)
                                                .small()
                                                .color(styles::DIM_TEXT),
                                        );
                                    }
                                });
                                if let Some(desc) = &result.description {
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .small()
                                            .color(styles::DIM_TEXT),
                                    );
                                }
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new("🔍 Investigate")
                                                .rounding(4.0),
                                        )
                                        .clicked()
                                    {
                                        app.is_loading = true;
                                        app.selected_company = Some(cn.to_string());
                                        app.investigate_company(cn.to_string());
                                        app.push_view(super::View::Company);
                                        app.search_results.clear();
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        });
    } else if app.search_query.is_empty() {
        render_tips(ui);
    }
}

/// Render officer search results.
fn render_officer_results(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    if !app.officer_search_results.is_empty() {
        ui.separator();
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("{} directors found", app.officer_search_results.len()))
                .small()
                .color(styles::DIM_TEXT),
        );
        ui.add_space(4.0);

        egui::ScrollArea::vertical().id_salt("search_scroll").show(ui, |ui| {
            for result in &app.officer_search_results.clone() {
                let name = result.title.as_deref().unwrap_or("Unknown");
                let desc = result.description.as_deref().unwrap_or("");

                // DOB
                let dob_str = result.date_of_birth.as_ref().map(|dob| {
                    let month_name = match dob.month.unwrap_or(0) {
                        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                        5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                        9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                        _ => "?",
                    };
                    format!("Born {} {}", month_name, dob.year.unwrap_or(0))
                });

                // Address
                let addr_str = result.address.as_ref().map(|a| a.one_line());

                // Appointment count
                let appt_count = result.appointment_count.unwrap_or(0);

                egui::Frame::none()
                    .fill(styles::BG_CARD)
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(12))
                    .stroke(egui::Stroke::new(1.0, styles::BG_HOVER))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("👤 {}", name))
                                        .strong()
                                        .size(14.0)
                                        .color(styles::TEXT_PRIMARY),
                                );
                                ui.horizontal(|ui| {
                                    if let Some(dob) = &dob_str {
                                        ui.label(
                                            egui::RichText::new(dob)
                                                .small()
                                                .color(styles::ACCENT),
                                        );
                                    }
                                    ui.label(
                                        egui::RichText::new(format!("{} appointments", appt_count))
                                            .small()
                                            .color(styles::ACCENT_YELLOW),
                                    );
                                });
                                if let Some(addr) = &addr_str {
                                    ui.label(
                                        egui::RichText::new(format!("📍 {}", addr))
                                            .small()
                                            .color(styles::DIM_TEXT),
                                    );
                                }
                                if !desc.is_empty() {
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .small()
                                            .color(styles::DIM_TEXT),
                                    );
                                }
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Get the appointments path from links
                                    let appt_path = result.links.as_ref()
                                        .and_then(|l| l.self_link.as_ref())
                                        .cloned();

                                    if let Some(path) = appt_path {
                                        if ui
                                            .add(
                                                egui::Button::new("📋 View Appointments →")
                                                    .rounding(4.0),
                                            )
                                            .clicked()
                                        {
                                            app.is_loading = true;
                                            app.fetch_officer_appointments(
                                                name.to_string(),
                                                path,
                                            );
                                        }
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        });
    } else if app.search_query.is_empty() {
        render_tips(ui);
    }
}

/// Render officer appointments detail view.
fn render_officer_appointments(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    let response = match &app.selected_officer_appointments {
        Some(r) => r.clone(),
        None => return,
    };

    let officer_name = app.selected_officer_name.clone().unwrap_or("Unknown".into());

    // ── Header ───────────────────────────────────────────────────────
    ui.separator();
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("← Back to results").clicked() {
            app.selected_officer_appointments = None;
            app.selected_officer_name = None;
        }
        ui.separator();
        ui.label(
            egui::RichText::new(format!("👤 {}", officer_name))
                .strong()
                .size(16.0)
                .color(styles::TEXT_PRIMARY),
        );
    });

    ui.add_space(8.0);

    // ── Officer summary ──────────────────────────────────────────────
    egui::Frame::none()
        .fill(styles::BG_CARD)
        .rounding(8.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if let Some(dob) = &response.date_of_birth {
                    let month_name = match dob.month.unwrap_or(0) {
                        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                        5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                        9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                        _ => "?",
                    };
                    ui.label(egui::RichText::new(format!("🎂 Born {} {}", month_name, dob.year.unwrap_or(0)))
                        .color(styles::TEXT_PRIMARY));
                    ui.separator();
                }
                if let Some(active) = response.active_count {
                    ui.label(egui::RichText::new(format!("🟢 {} active", active))
                        .color(egui::Color32::from_rgb(100, 200, 100)));
                }
                if let Some(inactive) = response.inactive_count {
                    ui.label(egui::RichText::new(format!("⚪ {} inactive", inactive))
                        .color(styles::DIM_TEXT));
                }
                if let Some(resigned) = response.resigned_count {
                    if resigned > 0 {
                        ui.label(egui::RichText::new(format!("🔴 {} resigned", resigned))
                            .color(egui::Color32::from_rgb(200, 100, 100)));
                    }
                }
                if response.is_corporate_officer == Some(true) {
                    ui.separator();
                    ui.label(egui::RichText::new("🏢 Corporate Officer")
                        .color(styles::ACCENT_YELLOW));
                }
                ui.separator();
                ui.label(egui::RichText::new(format!("{} total appointments",
                    response.total_results.unwrap_or(0)))
                    .color(styles::DIM_TEXT));
            });
        });

    ui.add_space(8.0);

    // ── Bulk action ──────────────────────────────────────────────────
    let active_companies: Vec<_> = response.items.as_ref().map(|items| {
        items.iter().filter(|a| {
            a.resigned_on.is_none() &&
            a.appointed_to.as_ref().map(|t| t.company_status.as_deref() != Some("dissolved")).unwrap_or(false)
        }).collect::<Vec<_>>()
    }).unwrap_or_default();

    if active_companies.len() > 1 {
        if ui.add(
            egui::Button::new(
                egui::RichText::new(format!("🔍 Investigate All {} Active Companies", active_companies.len()))
                    .color(styles::BG_DARK)
            )
            .fill(styles::ACCENT)
            .rounding(4.0)
        ).clicked() {
            for appt in &active_companies {
                if let Some(co) = &appt.appointed_to {
                    if let Some(cn) = &co.company_number {
                        app.investigate_company(cn.clone());
                    }
                }
            }
            app.push_view(super::View::Dashboard);
        }
    }

    ui.add_space(8.0);

    // ── Appointments list ────────────────────────────────────────────
    egui::ScrollArea::vertical().show(ui, |ui| {
        let items = response.items.unwrap_or_default();
        for appt in &items {
            let company_name = appt.appointed_to.as_ref()
                .and_then(|t| t.company_name.as_deref())
                .unwrap_or("Unknown Company");
            let company_number = appt.appointed_to.as_ref()
                .and_then(|t| t.company_number.as_deref())
                .unwrap_or("?");
            let company_status = appt.appointed_to.as_ref()
                .and_then(|t| t.company_status.as_deref())
                .unwrap_or("?");
            let role = appt.officer_role.as_deref().unwrap_or("unknown");
            let appointed = appt.appointed_on.as_deref().unwrap_or("?");
            let resigned = appt.resigned_on.as_deref();
            let nationality = appt.nationality.as_deref();
            let country = appt.country_of_residence.as_deref();
            let occupation = appt.occupation.as_deref();
            let is_active = resigned.is_none();

            // Status indicator
            let status_color = match company_status {
                "active" => egui::Color32::from_rgb(100, 200, 100),
                "dissolved" => egui::Color32::from_rgb(200, 100, 100),
                "liquidation" => egui::Color32::from_rgb(200, 150, 50),
                _ => styles::DIM_TEXT,
            };

            egui::Frame::none()
                .fill(if is_active { styles::BG_CARD } else { styles::BG_DARK })
                .rounding(8.0)
                .inner_margin(egui::Margin::same(12))
                .stroke(egui::Stroke::new(
                    if is_active { 1.0 } else { 0.5 },
                    if is_active { styles::ACCENT } else { styles::BG_HOVER },
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            // Company name + number
                            ui.label(
                                egui::RichText::new(company_name)
                                    .strong()
                                    .size(14.0)
                                    .color(if is_active { styles::TEXT_PRIMARY } else { styles::DIM_TEXT }),
                            );

                            // Status + number row
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(company_number)
                                        .small()
                                        .color(styles::ACCENT),
                                );
                                let status_icon = match company_status {
                                    "active" => "🟢",
                                    "dissolved" => "🔴",
                                    "liquidation" => "🟡",
                                    _ => "⚪",
                                };
                                ui.label(
                                    egui::RichText::new(format!("{} {}", status_icon, company_status))
                                        .small()
                                        .color(status_color),
                                );
                            });

                            // Role + dates
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("📋 Role: {}", role))
                                        .small()
                                        .color(styles::TEXT_PRIMARY),
                                );
                                ui.separator();
                                ui.label(
                                    egui::RichText::new(format!("Appointed: {}", appointed))
                                        .small()
                                        .color(styles::DIM_TEXT),
                                );
                                if let Some(r) = resigned {
                                    ui.label(
                                        egui::RichText::new(format!("Resigned: {}", r))
                                            .small()
                                            .color(egui::Color32::from_rgb(200, 100, 100)),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new("✓ Current")
                                            .small()
                                            .color(egui::Color32::from_rgb(100, 200, 100)),
                                    );
                                }
                            });

                            // Nationality + country + occupation
                            ui.horizontal(|ui| {
                                if let Some(nat) = nationality {
                                    ui.label(
                                        egui::RichText::new(format!("🌍 {}", nat))
                                            .small()
                                            .color(styles::DIM_TEXT),
                                    );
                                }
                                if let Some(c) = country {
                                    ui.label(
                                        egui::RichText::new(format!("📍 {}", c))
                                            .small()
                                            .color(styles::DIM_TEXT),
                                    );
                                }
                                if let Some(occ) = occupation {
                                    ui.label(
                                        egui::RichText::new(format!("💼 {}", occ))
                                            .small()
                                            .color(styles::DIM_TEXT),
                                    );
                                }
                            });

                            // Address
                            if let Some(addr) = &appt.address {
                                ui.label(
                                    egui::RichText::new(format!("🏠 {}", addr.one_line()))
                                        .small()
                                        .color(styles::DIM_TEXT),
                                );
                            }

                            // Name elements if different from title
                            if let Some(ne) = &appt.name_elements {
                                let parts: Vec<String> = [
                                    ne.title.as_deref().map(|s| s.to_string()),
                                    ne.forename.as_deref().map(|s| s.to_string()),
                                    ne.other_forenames.as_deref().map(|s| s.to_string()),
                                    ne.surname.as_deref().map(|s| s.to_string()),
                                ].iter().filter_map(|p| p.clone()).collect();
                                if !parts.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!("Name: {}", parts.join(" ")))
                                            .small()
                                            .color(styles::DIM_TEXT),
                                    );
                                }
                            }
                        });

                        // Investigate button
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui
                                    .add(
                                        egui::Button::new("🔍 Investigate")
                                            .rounding(4.0),
                                    )
                                    .clicked()
                                {
                                    app.is_loading = true;
                                    app.selected_company = Some(company_number.to_string());
                                    app.investigate_company(company_number.to_string());
                                    app.push_view(super::View::Company);
                                }
                            },
                        );
                    });
                });
            ui.add_space(4.0);
        }
    });
}

/// Search results overlay (used when searching from other views).
pub fn render_results(app: &mut InvestigationApp, ctx: &egui::Context) {
    egui::Window::new("Search Results")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 40.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} results", app.search_results.len()))
                        .small()
                        .color(styles::DIM_TEXT),
                );
                if ui.small_button("✕ Close").clicked() {
                    app.search_results.clear();
                }
                if ui.small_button("View in Search panel").clicked() {
                    app.active_view = super::View::Search;
                }
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for result in &app.search_results.clone() {
                        let cn = result.company_number.as_deref().unwrap_or("?");
                        let name = result.title.as_deref().unwrap_or("Unknown");
                        let status = result.company_status.as_deref().unwrap_or("?");

                        egui::Frame::none()
                            .fill(styles::BG_CARD)
                            .rounding(6.0)
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(name)
                                                .strong()
                                                .color(styles::TEXT_PRIMARY),
                                        );
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(cn)
                                                    .small()
                                                    .color(styles::ACCENT),
                                            );
                                            ui.label(
                                                egui::RichText::new(status)
                                                    .small()
                                                    .color(styles::status_color(status)),
                                            );
                                        });
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("🔍 Investigate").clicked() {
                                                app.is_loading = true;
                                                app.selected_company = Some(cn.to_string());
                                                app.investigate_company(cn.to_string());
                                                app.push_view(super::View::Company);
                                                app.search_results.clear();
                                            }
                                        },
                                    );
                                });
                            });
                        ui.add_space(4.0);
                    }
                });
        });
}

fn render_tips(ui: &mut egui::Ui) {
    ui.add_space(16.0);
    egui::Frame::none()
        .fill(styles::BG_CARD)
        .rounding(8.0)
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("💡 Search Tips")
                    .strong()
                    .color(styles::TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            tip_row(ui, "Company name:", "\"Barclays\", \"Tesco PLC\"");
            tip_row(ui, "Company number:", "\"00102498\", \"SC123456\"");
            tip_row(ui, "Director name:", "Switch to Director Search to find people");
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Tip: Enter an 8-digit company number to go directly to the profile.")
                    .small()
                    .color(styles::DIM_TEXT),
            );
        });
}

fn tip_row(ui: &mut egui::Ui, label: &str, example: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).strong().color(styles::DIM_TEXT));
        ui.label(egui::RichText::new(example).color(styles::TEXT_SECONDARY));
    });
}
