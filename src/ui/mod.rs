pub mod landing;
pub mod dashboard;
pub mod search_panel;
pub mod company_panel;
pub mod network_graph;
pub mod filings_panel;
pub mod officers_panel;
pub mod analysis_panel;
pub mod settings_panel;
pub mod styles;

use crate::app::InvestigationApp;

/// Sidebar navigation sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Welcome,
    Search,
    Dashboard,
    Company,
    Network,
    Filings,
    Officers,
    Analysis,
    Settings,
}

impl View {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Welcome => "🏠",
            Self::Search => "🔍",
            Self::Dashboard => "📊",
            Self::Company => "🏢",
            Self::Network => "🕸",
            Self::Filings => "📄",
            Self::Officers => "👤",
            Self::Analysis => "🤖",
            Self::Settings => "⚙",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Welcome => "Home",
            Self::Search => "Search",
            Self::Dashboard => "Dashboard",
            Self::Company => "Company",
            Self::Network => "Network",
            Self::Filings => "Filings",
            Self::Officers => "Officers",
            Self::Analysis => "AI Analysis",
            Self::Settings => "Settings",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome & getting started",
            Self::Search => "Search Companies House",
            Self::Dashboard => "Investigation overview",
            Self::Company => "Selected company detail",
            Self::Network => "Corporate network graph",
            Self::Filings => "Filing history",
            Self::Officers => "Officers & directors",
            Self::Analysis => "AI-powered analysis",
            Self::Settings => "API keys & configuration",
        }
    }

    /// Navigation items shown in the sidebar (grouped).
    pub const MAIN_NAV: [View; 3] = [View::Welcome, View::Search, View::Dashboard];
    pub const INVESTIGATION_NAV: [View; 5] = [
        View::Company,
        View::Network,
        View::Filings,
        View::Officers,
        View::Analysis,
    ];
}

/// Main render function — called every frame.
pub fn render(app: &mut InvestigationApp, ctx: &egui::Context) {
    styles::apply_theme(ctx);

    // ─── Left sidebar ───
    egui::SidePanel::left("sidebar")
        .resizable(false)
        .exact_width(if app.sidebar_collapsed { 48.0 } else { 180.0 })
        .show(ctx, |ui| {
            render_sidebar(app, ui);
        });

    // ─── Bottom status bar ───
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(22.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&app.status_message)
                        .small()
                        .color(styles::STATUS_TEXT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let ch_dot = if app.config.has_ch_key() { "🟢" } else { "🔴" };
                    let ai_dot = if app.config.has_ai() { "🟢" } else { "⚪" };
                    let cost_str = if app.session_cost_usd > 0.0 {
                        format!("│  💰 ${:.4}", app.session_cost_usd)
                    } else {
                        String::new()
                    };
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  {} CH  {} AI  │  {} companies  {} links  {}",
                            if app.is_loading { "⏳" } else { "" },
                            ch_dot,
                            ai_dot,
                            app.network.node_count(),
                            app.network.edge_count(),
                            cost_str,
                        ))
                        .small()
                        .color(styles::DIM_TEXT),
                    );
                });
            });
        });

    // ─── Main content area ───
    egui::CentralPanel::default().show(ctx, |ui| {
        // Reset scroll position when view changes
        if app.scroll_reset_needed {
            app.scroll_reset_needed = false;
            // Reset scroll state for all known scroll areas by clearing
            // egui's memory for each panel's auto-generated scroll Id.
            // This works because the panel Ids are derived from the Ui Id.
            let panel_id = ui.id();
            for salt in &["company_scroll", "search_scroll", "dashboard_scroll",
                          "filings_scroll", "officers_scroll", "analysis_scroll",
                          "landing_scroll", "network_scroll"] {
                let scroll_id = panel_id.with(*salt);
                if let Some(mut state) = egui::scroll_area::State::load(ctx, scroll_id) {
                    state.offset = egui::Vec2::ZERO;
                    state.store(ctx, scroll_id);
                }
            }
        }
        match app.active_view {
            View::Welcome => landing::render(app, ui),
            View::Search => search_panel::render_search(app, ui),
            View::Dashboard => dashboard::render(app, ui),
            View::Company => company_panel::render(app, ui),
            View::Network => network_graph::render(app, ui),
            View::Filings => filings_panel::render(app, ui),
            View::Officers => officers_panel::render(app, ui),
            View::Analysis => analysis_panel::render(app, ui),
            View::Settings => {
                // Settings renders its own CentralPanel, so just return
            }
        }
    });

    // Settings uses its own CentralPanel so must be called after
    if app.active_view == View::Settings || app.show_settings {
        app.show_settings = true;
        settings_panel::render(app, ctx);
    }
}

/// Renders the sidebar navigation.
fn render_sidebar(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    let collapsed = app.sidebar_collapsed;

    ui.vertical(|ui| {
        ui.add_space(8.0);

        // App title / collapse toggle
        ui.horizontal(|ui| {
            if collapsed {
                if ui
                    .add(egui::Button::new("▸").frame(false))
                    .on_hover_text("Expand sidebar")
                    .clicked()
                {
                    app.sidebar_collapsed = false;
                }
            } else {
                ui.label(
                    egui::RichText::new("Corpex")
                        .strong()
                        .size(14.0)
                        .color(styles::ACCENT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new("◂").frame(false))
                        .on_hover_text("Collapse sidebar")
                        .clicked()
                    {
                        app.sidebar_collapsed = true;
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // ── Quick search bar (always visible) ─────────────────────────
        if !collapsed {
            ui.horizontal(|ui| {
                let hint = match app.search_mode {
                    crate::app::SearchMode::Company => "🏢 Search companies...",
                    crate::app::SearchMode::Officer => "👤 Search directors...",
                };
                let resp = ui.add_sized(
                    [ui.available_width() - 4.0, 26.0],
                    egui::TextEdit::singleline(&mut app.search_query)
                        .hint_text(hint)
                        .desired_width(ui.available_width() - 4.0),
                );
                // Press Enter → run search and jump to Search view
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !app.search_query.trim().is_empty() {
                        let query = app.search_query.trim().to_string();
                        app.is_loading = true;

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
                            app.active_view = crate::ui::View::Company;
                            app.scroll_reset_needed = true;
                        } else {
                            // Clear previous results before new search
                            app.search_results.clear();
                            app.officer_search_results.clear();
                            match app.search_mode {
                                crate::app::SearchMode::Company => app.search_companies(query),
                                crate::app::SearchMode::Officer => app.search_officers(query),
                            }
                            app.active_view = crate::ui::View::Search;
                            app.scroll_reset_needed = true;
                        }
                    }
                }
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
        }

        if !collapsed {
            ui.label(
                egui::RichText::new("NAVIGATE")
                    .small()
                    .color(styles::DIM_TEXT),
            );
        }

        // ← Back button (only shown if there's view history)
        if !app.view_history.is_empty() {
            let back_text = if collapsed {
                egui::RichText::new("←").size(16.0).color(styles::ACCENT_YELLOW)
            } else {
                egui::RichText::new("← Back").color(styles::ACCENT_YELLOW)
            };
            if ui.add(
                egui::Button::new(back_text)
                    .frame(false)
                    .min_size(egui::vec2(if collapsed { 32.0 } else { 160.0 }, 28.0)),
            )
            .on_hover_text("Go back to previous view")
            .clicked()
            {
                app.pop_view();
            }
        }

        for &view in &View::MAIN_NAV {
            sidebar_button(app, ui, view, collapsed);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        if !collapsed {
            ui.label(
                egui::RichText::new("INVESTIGATE")
                    .small()
                    .color(styles::DIM_TEXT),
            );
        }
        for &view in &View::INVESTIGATION_NAV {
            let enabled = app.network.node_count() > 0 || view == View::Network;
            if !enabled {
                // Still show but greyed out
                let text = if collapsed {
                    egui::RichText::new(view.icon()).color(styles::BG_HOVER)
                } else {
                    egui::RichText::new(format!("{}  {}", view.icon(), view.label()))
                        .color(styles::BG_HOVER)
                };
                ui.add_enabled(false, egui::Button::new(text).frame(false))
                    .on_disabled_hover_text("Search for a company first");
            } else {
                sidebar_button(app, ui, view, collapsed);
            }
        }

        // Push settings to bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(8.0);
            sidebar_button(app, ui, View::Settings, collapsed);
            ui.separator();
        });
    });
}

fn sidebar_button(app: &mut InvestigationApp, ui: &mut egui::Ui, view: View, collapsed: bool) {
    let is_active = app.active_view == view;

    let text = if collapsed {
        egui::RichText::new(view.icon()).size(16.0)
    } else {
        egui::RichText::new(format!("{}  {}", view.icon(), view.label()))
    };

    let text = if is_active {
        text.strong().color(styles::ACCENT)
    } else {
        text.color(styles::TEXT_PRIMARY)
    };

    let btn = egui::Button::new(text)
        .frame(false)
        .min_size(egui::vec2(if collapsed { 32.0 } else { 160.0 }, 28.0));

    let resp = ui.add(btn).on_hover_text(view.tooltip());

    // Active indicator bar
    if is_active {
        let rect = resp.rect;
        let bar = egui::Rect::from_min_size(
            rect.left_top(),
            egui::vec2(3.0, rect.height()),
        );
        ui.painter().rect_filled(bar, 0.0, styles::ACCENT);
    }

    if resp.clicked() {
        app.active_view = view;
        app.scroll_reset_needed = true;
        if view == View::Settings {
            app.show_settings = true;
        }
    }
}
