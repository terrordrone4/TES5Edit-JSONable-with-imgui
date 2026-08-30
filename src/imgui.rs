//! GUI portal. The rest of the application only calls [`run`].

#[path = "imgui/app.rs"]
mod app;
#[path = "imgui/components/mod.rs"]
mod components;
#[path = "imgui/utils.rs"]
mod utils;

pub fn run() -> eframe::Result<()> {
    eframe::run_native(
        "TES5Edit Rust JSON",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::<app::App>::default())),
    )
}
