use std::path::Path;

use anyhow::Result;

use super::{Plugin, io_utils::ensure_supported_plugin_path, parse_file_impl};

/// Public read portal for Skyrim plugin files.
pub fn parse_file(path: impl AsRef<Path>) -> Result<Plugin> {
    let path = path.as_ref();
    ensure_supported_plugin_path(path)?;
    parse_file_impl(path)
}
