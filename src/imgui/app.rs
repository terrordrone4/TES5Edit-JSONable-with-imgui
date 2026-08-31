use std::{fs, path::PathBuf};

use eframe::egui;
use tes5edit_rust_json::{
    ParseOptions, inspect_json_input, parse_file_with_options, read_json_input, to_json_pretty,
    write_file, write_json_pack_with_options,
};

use super::components::output_log::OutputLog;

pub struct App {
    plugins: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    split_json_pack: bool,
    preserve_byte_identical: bool,
    trim_default_values: bool,
    json_input: Option<PathBuf>,
    json_input_description: String,
    output_log: OutputLog,
}

impl Default for App {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            output_dir: None,
            split_json_pack: true,
            preserve_byte_identical: false,
            trim_default_values: true,
            json_input: None,
            json_input_description: String::new(),
            output_log: OutputLog::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Skyrim Plugin <-> JSON");
            ui.label("Lossless structural converter for .esp, .esm, and .esl files");
            ui.separator();
            ui.heading("Plugin -> JSON");
            if ui.button("Browse plugin files...").clicked()
                && let Some(files) = rfd::FileDialog::new()
                    .add_filter("Skyrim plugins", &["esp", "esm", "esl"])
                    .pick_files()
            {
                self.plugins = files;
            }
            for path in &self.plugins {
                ui.label(path.display().to_string());
            }
            if ui.button("Select output folder...").clicked() {
                self.output_dir = rfd::FileDialog::new().pick_folder();
            }
            if let Some(path) = &self.output_dir {
                ui.label(format!("Output: {}", path.display()));
            }
            ui.checkbox(
                &mut self.split_json_pack,
                "Build multiple JSON files grouped by top-level signature",
            );
            ui.checkbox(
                &mut self.preserve_byte_identical,
                "Preserve original compressed bytes for byte-identical round trips",
            );
            ui.checkbox(&mut self.trim_default_values, "Trim default values");
            if ui
                .add_enabled(
                    !self.plugins.is_empty() && self.output_dir.is_some(),
                    egui::Button::new("Parse to JSON"),
                )
                .clicked()
            {
                self.parse_plugins();
            }

            ui.add_space(16.0);
            ui.separator();
            ui.heading("JSON -> Plugin");
            ui.horizontal(|ui| {
                if ui.button("Browse JSON file...").clicked() {
                    self.json_input = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file();
                    self.refresh_json_input_description();
                }
                if ui.button("Browse JSON pack folder...").clicked() {
                    self.json_input = rfd::FileDialog::new().pick_folder();
                    self.refresh_json_input_description();
                }
            });
            ui.small(if self.json_input_description.is_empty() {
                "Input type: select one JSON file or one JSON-pack folder"
            } else {
                &self.json_input_description
            });
            if let Some(path) = &self.json_input {
                ui.label(path.display().to_string());
            }
            if ui
                .add_enabled(
                    self.json_input.is_some(),
                    egui::Button::new("Convert to plugin..."),
                )
                .clicked()
            {
                self.convert_json();
            }

            ui.add_space(16.0);
            ui.separator();
            self.output_log.show(ui);
        });
    }
}

impl App {
    fn parse_plugins(&mut self) {
        let output = self.output_dir.clone().unwrap();
        let result = (|| -> anyhow::Result<(usize, PathBuf, PathBuf)> {
            let mut first_output = None;
            let mut json_file_count = 0;
            let mut exact_destination = None;
            for input in &self.plugins {
                let plugin = parse_file_with_options(
                    input,
                    ParseOptions {
                        preserve_original_compression: self.preserve_byte_identical,
                    },
                )?;
                let name = format!("{}.json", input.file_name().unwrap().to_string_lossy());
                if self.split_json_pack {
                    let pack_dir = output.join(format!("{name}-pack"));
                    let written =
                        write_json_pack_with_options(&plugin, pack_dir, self.trim_default_values)?;
                    json_file_count += written.json_file_count;
                    first_output.get_or_insert(written.first_output);
                    if self.plugins.len() == 1 {
                        exact_destination = Some(written.pack_dir);
                    }
                } else {
                    let output_file = output.join(name);
                    fs::write(
                        &output_file,
                        to_json_pretty(&plugin, self.trim_default_values)?,
                    )?;
                    first_output.get_or_insert(output_file);
                    json_file_count += 1;
                }
            }
            Ok((
                json_file_count,
                first_output.unwrap(),
                exact_destination.unwrap_or(output),
            ))
        })();
        match result {
            Ok((count, first_output, destination)) => self.output_log.push(
                format!("Created {count} JSON file(s) in {}.", destination.display()),
                Some(first_output),
            ),
            Err(error) => self.output_log.push(format!("Error: {error:#}"), None),
        }
    }

    fn convert_json(&mut self) {
        let input = self.json_input.clone().unwrap();
        let plugin = match read_json_input(&input) {
            Ok(plugin) => plugin,
            Err(error) => {
                self.output_log.push(format!("Error: {error:#}"), None);
                return;
            }
        };
        let fallback = input
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("output.esp")
            .trim_end_matches(".json-pack")
            .trim_end_matches(".json");
        let suggested = plugin.source_file.as_deref().unwrap_or(fallback);
        let Some(output) = rfd::FileDialog::new()
            .set_file_name(suggested)
            .add_filter("Skyrim plugin", &["esp", "esm", "esl"])
            .save_file()
        else {
            return;
        };
        let result = (|| -> anyhow::Result<()> { write_file(&plugin, &output) })();
        match result {
            Ok(()) => self
                .output_log
                .push("Plugin created successfully.", Some(output)),
            Err(error) => self.output_log.push(format!("Error: {error:#}"), None),
        }
    }

    fn refresh_json_input_description(&mut self) {
        self.json_input_description = match self.json_input.as_ref() {
            None => String::new(),
            Some(path) => match inspect_json_input(path) {
                Ok(info) if info.is_pack => format!(
                    "Input type: multiple JSON files ({} different label_signature values)",
                    info.label_signature_count
                ),
                Ok(_) => "Input type: single JSON file".into(),
                Err(error) => format!("Input type: invalid JSON input ({error:#})"),
            },
        };
    }
}
