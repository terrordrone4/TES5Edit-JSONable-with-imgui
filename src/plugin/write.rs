use std::path::Path;

use anyhow::Result;

use super::{Plugin, io_utils::ensure_supported_plugin_path, write_file_impl};

/// Public write portal for rebuilt Skyrim plugin files.
pub fn write_file(plugin: &Plugin, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    ensure_supported_plugin_path(path)?;
    write_file_impl(plugin, path)
}
