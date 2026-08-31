use std::path::Path;

use anyhow::Result;

use super::{Plugin, io_utils::ensure_supported_plugin_path, parse_file_impl};

#[derive(Debug, Clone, Copy, Default)]
pub struct ParseOptions {
    pub preserve_original_compression: bool,
}

/// Public read portal for Skyrim plugin files.
pub fn parse_file(path: impl AsRef<Path>) -> Result<Plugin> {
    parse_file_with_options(path, ParseOptions::default())
}

pub fn parse_file_with_options(path: impl AsRef<Path>, options: ParseOptions) -> Result<Plugin> {
    let path = path.as_ref();
    ensure_supported_plugin_path(path)?;
    parse_file_impl(path, options)
}
