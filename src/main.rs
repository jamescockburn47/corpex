mod app;
mod config;

mod ui;
mod ch_api;
mod extraction;
mod investigation;
mod ai;
mod export;
mod cache;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "corpex=info,warn".into()),
        )
        .init();

    // Load .env if present
    dotenvy::dotenv().ok();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 700.0])
            .with_title("Corpex — Corporate Intelligence"),
        ..Default::default()
    };

    eframe::run_native(
        "Corpex",
        native_options,
        Box::new(|cc| Ok(Box::new(app::InvestigationApp::new(cc)))),
    )
    .expect("Failed to launch application");
}
