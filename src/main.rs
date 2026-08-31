#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod imgui;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use eframe::egui;
use tes5edit_rust_json::{
    Plugin, parse_file, read_json_input, to_json_pretty, validate_plugin, write_file,
    write_json_pack,
};

#[derive(Default)]
struct App {
    plugins: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    json_file: Option<PathBuf>,
    log: Vec<LogEntry>,
}

struct LogEntry {
    message: String,
    first_output: Option<PathBuf>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Skyrim Plugin ⇄ JSON");
            ui.label("Lossless structural converter for .esp, .esm, and .esl files");
            ui.separator();
            ui.heading("Plugin → JSON");
            if ui.button("Browse plugin files…").clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Skyrim plugins", &["esp", "esm", "esl"])
                    .pick_files()
                {
                    self.plugins = files;
                }
            }
            for path in &self.plugins {
                ui.label(path.display().to_string());
            }
            if ui.button("Select output folder…").clicked() {
                self.output_dir = rfd::FileDialog::new().pick_folder();
            }
            if let Some(path) = &self.output_dir {
                ui.label(format!("Output: {}", path.display()));
            }
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
            ui.heading("JSON → Plugin");
            if ui.button("Browse JSON file…").clicked() {
                self.json_file = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file();
            }
            if let Some(path) = &self.json_file {
                ui.label(path.display().to_string());
            }
            if ui
                .add_enabled(
                    self.json_file.is_some(),
                    egui::Button::new("Convert to plugin…"),
                )
                .clicked()
            {
                self.convert_json();
            }
            ui.add_space(16.0);
            ui.separator();
            ui.heading("Output log");
            egui::Frame::group(ui.style()).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("output_log")
                    .max_height(180.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        if self.log.is_empty() {
                            ui.weak("...");
                        }
                        for entry in &mut self.log {
                            ui.horizontal_wrapped(|ui| {
                                if let Some(path) = &entry.first_output
                                    && ui.button("Open folder").clicked()
                                    && let Err(error) = reveal_in_folder(path)
                                {
                                    entry.message = format!(
                                        "{} (could not open Explorer: {error})",
                                        entry.message
                                    );
                                }
                                ui.label(&entry.message);
                            });
                            ui.separator();
                        }
                    });
            });
        });
    }
}

impl App {
    fn parse_plugins(&mut self) {
        let output = self.output_dir.clone().unwrap();
        let result = (|| -> anyhow::Result<(usize, PathBuf)> {
            let mut first_output = None;
            for input in &self.plugins {
                let plugin = parse_file(input)?;
                let name = format!("{}.json", input.file_name().unwrap().to_string_lossy());
                let output_file = output.join(name);
                fs::write(&output_file, serde_json::to_vec_pretty(&plugin)?)?;
                first_output.get_or_insert(output_file);
            }
            Ok((self.plugins.len(), first_output.unwrap()))
        })();
        match result {
            Ok((count, first_output)) => {
                self.push_log(format!("Created {count} JSON file(s)."), Some(first_output))
            }
            Err(error) => self.push_log(format!("Error: {error:#}"), None),
        }
    }

    fn convert_json(&mut self) {
        let input = self.json_file.clone().unwrap();
        let suggested = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output.esp")
            .trim_end_matches(".json");
        let Some(output) = rfd::FileDialog::new()
            .set_file_name(suggested)
            .add_filter("Skyrim plugin", &["esp", "esm", "esl"])
            .save_file()
        else {
            return;
        };
        let result = (|| -> anyhow::Result<()> {
            let plugin: Plugin = serde_json::from_slice(&fs::read(&input)?)?;
            write_file(&plugin, &output)
        })();
        match result {
            Ok(()) => self.push_log("Plugin created successfully.", Some(output)),
            Err(error) => self.push_log(format!("Error: {error:#}"), None),
        }
    }

    fn push_log(&mut self, message: impl Into<String>, first_output: Option<PathBuf>) {
        self.log.insert(
            0,
            LogEntry {
                message: message.into(),
                first_output,
            },
        );
    }
}

fn reveal_in_folder(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe")
            .arg(format!("/select,{}", path.display()))
            .spawn()?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open")
            .arg(path.parent().unwrap_or(path))
            .spawn()?;
        Ok(())
    }
}

fn main() -> eframe::Result<()> {
    if let Err(error) = run_cli() {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
    if std::env::args_os().len() > 1 {
        return Ok(());
    }
    imgui::run()
}

fn run_cli() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        return Ok(());
    }
    if args.len() != 3 {
        anyhow::bail!("usage: tes5edit-rust-json [to-json|to-json-pack|from-json] INPUT OUTPUT");
    }
    match args[0].to_string_lossy().as_ref() {
        "to-json" => {
            let plugin = parse_file(&args[1])?;
            fs::write(&args[2], to_json_pretty(&plugin, false)?)?;
        }
        "to-json-pack" => {
            let plugin = parse_file(&args[1])?;
            write_json_pack(&plugin, &args[2])?;
        }
        "from-json" => {
            let plugin = read_json_input(&args[1])?;
            validate_plugin(&plugin)?;
            write_file(&plugin, &args[2])?;
        }
        command => anyhow::bail!("unknown command {command:?}"),
    }
    Ok(())
}
