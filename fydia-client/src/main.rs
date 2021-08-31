#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::FydiaApp;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let app = crate::FydiaApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
