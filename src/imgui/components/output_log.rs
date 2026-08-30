use std::path::PathBuf;

use eframe::egui;

use crate::imgui::utils::reveal_in_folder;

/// One immutable conversion event plus an optional output to reveal.
pub struct OutputLogItem {
    message: String,
    first_output: Option<PathBuf>,
}

/// Newest-first, scrollable conversion history component.
#[derive(Default)]
pub struct OutputLog {
    items: Vec<OutputLogItem>,
}

impl OutputLog {
    pub fn push(&mut self, message: impl Into<String>, first_output: Option<PathBuf>) {
        self.items.insert(
            0,
            OutputLogItem {
                message: message.into(),
                first_output,
            },
        );
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Output log");
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("output_log")
                .max_height(180.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    if self.items.is_empty() {
                        ui.weak("...");
                    }
                    for item in &mut self.items {
                        ui.horizontal_wrapped(|ui| {
                            if let Some(path) = &item.first_output
                                && ui.button("Open folder").clicked()
                                && let Err(error) = reveal_in_folder(path)
                            {
                                item.message =
                                    format!("{} (could not open Explorer: {error})", item.message);
                            }
                            ui.label(&item.message);
                        });
                        ui.separator();
                    }
                });
        });
    }
}
