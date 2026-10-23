#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod calculator;
mod history;
mod input;
mod settings;
mod ui;

fn main() -> eframe::Result {
    let icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Calculator")
            .with_icon(icon)
            .with_inner_size([480.0, 680.0])
            .with_min_inner_size([400.0, 660.0])
            // Keep the existing ID so preferences and history stay in the same location.
            .with_app_id("com.cindral.calculator"),
        ..Default::default()
    };
    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| Ok(Box::new(app::CalculatorApp::new(cc)))),
    )
}
