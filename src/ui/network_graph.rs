use crate::app::InvestigationApp;
use crate::ui::styles;
use crate::investigation::network::RiskLevel;
use petgraph::visit::EdgeRef;

/// Network graph panel — interactive visualisation of the corporate network.
/// Uses a simple custom renderer since egui_graphs may need specific wiring.
pub fn render(app: &mut InvestigationApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Corporate Network Graph");
        ui.label(
            egui::RichText::new(format!(
                "  ({} entities, {} connections)",
                app.network.node_count(),
                app.network.edge_count()
            ))
            .small()
            .color(styles::DIM_TEXT),
        );
    });
    ui.separator();

    if app.network.node_count() == 0 {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new("No companies in the network yet. Search and investigate a company to begin.")
                    .color(styles::DIM_TEXT),
            );
        });
        return;
    }

    // Legend
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Legend:").small().color(styles::DIM_TEXT));
        ui.label(egui::RichText::new("🟢 Low risk").small().color(styles::ACCENT_GREEN));
        ui.label(egui::RichText::new("🟡 Medium risk").small().color(styles::ACCENT_YELLOW));
        ui.label(egui::RichText::new("🔴 High risk").small().color(styles::ACCENT_RED));
        ui.label(egui::RichText::new("— Ownership").small().color(styles::ACCENT));
        ui.label(egui::RichText::new("--- Director link").small().color(styles::DIM_TEXT));
    });
    ui.add_space(8.0);

    // Render the network as a table/list view with relationship details
    // (Full force-directed canvas renderer would be added in a follow-up iteration)
    egui::ScrollArea::vertical().id_salt("network_scroll").show(ui, |ui| {
        // Show each node and its connections
        let node_indices: Vec<_> = app.network.graph.node_indices().collect();

        for &idx in &node_indices {
            let node = match app.network.graph.node_weight(idx) {
                Some(n) => n.clone(),
                None => continue,
            };

            egui::Frame::none()
                .fill(styles::BG_CARD)
                .rounding(8.0)
                .inner_margin(egui::Margin::same(10))
                .stroke(egui::Stroke::new(
                    1.5,
                    match node.risk_level {
                        RiskLevel::High => styles::ACCENT_RED,
                        RiskLevel::Medium => styles::ACCENT_YELLOW,
                        RiskLevel::Low => styles::BG_HOVER,
                    },
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        styles::risk_badge(ui, node.risk_level);
                        let resp = ui.selectable_label(
                            app.selected_company.as_deref() == Some(&node.company_number),
                            egui::RichText::new(format!(
                                "{} ({})",
                                node.company_name, node.company_number
                            ))
                            .strong()
                            .color(styles::TEXT_PRIMARY),
                        );
                        if resp.clicked() {
                            app.selected_company = Some(node.company_number.clone());
                            app.active_view = super::View::Company;
                        }
                        if resp.double_clicked() {
                            // Double-click to investigate further
                            app.is_loading = true;
                            app.investigate_company(node.company_number.clone());
                        }

                        ui.label(
                            egui::RichText::new(&node.status)
                                .small()
                                .color(styles::status_color(&node.status)),
                        );
                    });

                    // Show outgoing edges (this company → others)
                    let outgoing: Vec<_> = app
                        .network
                        .graph
                        .edges_directed(idx, petgraph::Direction::Outgoing)
                        .map(|e| {
                            let target = e.target();
                            let target_node = app.network.graph.node_weight(target);
                            (e.weight().label(), target_node.map(|n| n.company_name.clone()))
                        })
                        .collect();

                    let incoming: Vec<_> = app
                        .network
                        .graph
                        .edges_directed(idx, petgraph::Direction::Incoming)
                        .map(|e| {
                            let source = e.source();
                            let source_node = app.network.graph.node_weight(source);
                            (e.weight().label(), source_node.map(|n| n.company_name.clone()))
                        })
                        .collect();

                    if !outgoing.is_empty() || !incoming.is_empty() {
                        ui.indent(format!("edges_{}", node.company_number), |ui| {
                            for (label, target_name) in &outgoing {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "  → {} → {}",
                                        label,
                                        target_name.as_deref().unwrap_or("?")
                                    ))
                                    .small()
                                    .color(styles::ACCENT),
                                );
                            }
                            for (label, source_name) in &incoming {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "  ← {} ← {}",
                                        label,
                                        source_name.as_deref().unwrap_or("?")
                                    ))
                                    .small()
                                    .color(styles::ACCENT_GREEN),
                                );
                            }
                        });
                    }

                    // Risk signals
                    if !node.risk_signals.is_empty() {
                        for sig in &node.risk_signals {
                            ui.label(
                                egui::RichText::new(format!("  ⚠ {}", sig))
                                    .small()
                                    .color(styles::ACCENT_YELLOW),
                            );
                        }
                    }
                });
            ui.add_space(6.0);
        }
    });
}
