//! Corpex — Companies House corporate investigation tool.
//!
//! This `lib.rs` provides the WASM entry point for the web build
//! and declares all shared modules.

#[macro_use]
pub mod platform;
pub mod app;
pub mod config;
pub mod ui;
pub mod ch_api;
pub mod extraction;
pub mod investigation;
pub mod ai;
pub mod export;
pub mod cache;

// ── WASM entry point ────────────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
mod wasm_entry {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub async fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "corpex_canvas",
                    web_options,
                    Box::new(|cc| Ok(Box::new(crate::app::InvestigationApp::new(cc)))),
                )
                .await
                .expect("Failed to start eframe");
        });

        Ok(())
    }
}
