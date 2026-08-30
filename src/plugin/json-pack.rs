use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use super::{Element, Plugin};

const MANIFEST_NAME: &str = "pack-manifest.json";
const BLOBS_NAME: &str = "__blobs__.json";

#[derive(Debug, Clone)]
pub struct JsonInputInfo {
    pub is_pack: bool,
    pub label_signature_count: usize,
}

#[derive(Debug, Clone)]
pub struct JsonPackWriteResult {
    pub first_output: PathBuf,
    pub pack_dir: PathBuf,
    pub json_file_count: usize,
}

#[derive(Serialize, Deserialize)]
struct PackManifest {
    format: String,
    source_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blob_file: Option<String>,
    fragments: Vec<PackFragment>,
}

#[derive(Serialize, Deserialize)]
struct PackFragment {
    signature: String,
    file: String,
    element_count: usize,
}

pub fn write_json_pack(plugin: &Plugin, pack_dir: impl AsRef<Path>) -> Result<JsonPackWriteResult> {
    let pack_dir = pack_dir.as_ref();
    fs::create_dir_all(pack_dir)
        .with_context(|| format!("creating JSON pack {}", pack_dir.display()))?;
    remove_previous_pack_files(pack_dir)?;

    let mut buckets: Vec<(String, Vec<Element>)> = Vec::new();
    for element in &plugin.elements {
        let signature = top_signature(element);
        if let Some((_, elements)) = buckets.iter_mut().find(|(name, _)| name == &signature) {
            elements.push(element.clone());
        } else {
            buckets.push((signature, vec![element.clone()]));
        }
    }

    let mut fragments = Vec::new();
    let mut first_output = None;
    for (signature, elements) in buckets {
        let file = format!("{}.json", safe_file_stem(&signature));
        let output = pack_dir.join(&file);
        for id in referenced_blobs(&elements) {
            ensure!(
                plugin.blobs.contains_key(&id),
                "missing blob {id:?} while creating pack"
            );
        }
        let fragment = Plugin {
            format: plugin.format.clone(),
            source_file: plugin.source_file.clone(),
            elements,
            blobs: BTreeMap::new(),
        };
        fs::write(&output, serde_json::to_vec_pretty(&fragment)?)
            .with_context(|| format!("writing {}", output.display()))?;
        first_output.get_or_insert(output);
        fragments.push(PackFragment {
            signature,
            file,
            element_count: fragment.elements.len(),
        });
    }

    let json_file_count = fragments.len() + 2; // fragments + __blobs__ + manifest
    let manifest = PackManifest {
        format: "tes5edit-rust-json-pack/v1".into(),
        source_file: plugin.source_file.clone(),
        blob_file: Some(BLOBS_NAME.into()),
        fragments,
    };
    fs::write(
        pack_dir.join(BLOBS_NAME),
        serde_json::to_vec_pretty(&plugin.blobs)?,
    )?;
    fs::write(
        pack_dir.join(MANIFEST_NAME),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(JsonPackWriteResult {
        first_output: first_output.context("cannot create a JSON pack from an empty plugin")?,
        pack_dir: pack_dir.to_path_buf(),
        json_file_count,
    })
}

pub fn read_json_input(path: impl AsRef<Path>) -> Result<Plugin> {
    let path = path.as_ref();
    if path.is_file() {
        return serde_json::from_slice(&fs::read(path)?)
            .with_context(|| format!("reading JSON file {}", path.display()));
    }
    ensure!(
        path.is_dir(),
        "JSON input does not exist: {}",
        path.display()
    );
    let manifest_path = path.join(MANIFEST_NAME);
    ensure!(
        manifest_path.is_file(),
        "JSON pack is missing {MANIFEST_NAME}: {}",
        path.display()
    );
    let manifest: PackManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    ensure!(
        manifest.format == "tes5edit-rust-json-pack/v1",
        "unsupported JSON pack format {}",
        manifest.format
    );
    let mut elements = Vec::new();
    let mut blobs: BTreeMap<String, String> = match manifest.blob_file.as_deref() {
        Some(blob_file) => {
            let blob_path = path.join(blob_file);
            serde_json::from_slice(&fs::read(&blob_path)?)
                .with_context(|| format!("reading shared blob file {}", blob_path.display()))?
        }
        None => BTreeMap::new(),
    };
    let mut plugin_format = None;
    for entry in &manifest.fragments {
        let fragment_path = path.join(&entry.file);
        let fragment: Plugin = serde_json::from_slice(&fs::read(&fragment_path)?)
            .with_context(|| format!("reading pack fragment {}", fragment_path.display()))?;
        ensure!(
            fragment
                .elements
                .iter()
                .all(|e| top_signature(e) == entry.signature),
            "fragment {} contains an element outside signature {}",
            entry.file,
            entry.signature
        );
        ensure!(
            fragment.elements.len() == entry.element_count,
            "fragment {} element count differs from its manifest",
            entry.file
        );
        plugin_format.get_or_insert(fragment.format);
        elements.extend(fragment.elements);
        for (id, value) in fragment.blobs {
            if let Some(previous) = blobs.insert(id.clone(), value.clone())
                && previous != value
            {
                bail!("conflicting values for blob {id:?} across pack fragments");
            }
        }
    }
    Ok(Plugin {
        format: plugin_format.unwrap_or_else(|| "tes5edit-rust-json/v2".into()),
        source_file: manifest.source_file,
        elements,
        blobs,
    })
}

pub fn inspect_json_input(path: impl AsRef<Path>) -> Result<JsonInputInfo> {
    let path = path.as_ref();
    if path.is_file() {
        return Ok(JsonInputInfo {
            is_pack: false,
            label_signature_count: 0,
        });
    }
    let manifest: PackManifest = serde_json::from_slice(&fs::read(path.join(MANIFEST_NAME))?)?;
    let label_signature_count = manifest
        .fragments
        .iter()
        .filter(|fragment| fragment.signature != "TES4")
        .map(|fragment| &fragment.signature)
        .collect::<BTreeSet<_>>()
        .len();
    Ok(JsonInputInfo {
        is_pack: true,
        label_signature_count,
    })
}

fn top_signature(element: &Element) -> String {
    match element {
        Element::Record(record) => record.signature.clone(),
        Element::Group(group) => group.label_signature.clone().unwrap_or_else(|| {
            group
                .label
                .to_le_bytes()
                .into_iter()
                .map(char::from)
                .collect()
        }),
    }
}

fn referenced_blobs(elements: &[Element]) -> BTreeSet<String> {
    fn visit(element: &Element, refs: &mut BTreeSet<String>) {
        match element {
            Element::Record(record) => {
                if let Some(id) = &record.original_compressed_ref {
                    refs.insert(id.clone());
                }
                for subrecord in &record.subrecords {
                    if let Some(id) = &subrecord.data_ref {
                        refs.insert(id.clone());
                    }
                }
            }
            Element::Group(group) => {
                for child in &group.elements {
                    visit(child, refs);
                }
            }
        }
    }
    let mut refs = BTreeSet::new();
    for element in elements {
        visit(element, &mut refs);
    }
    refs
}

fn safe_file_stem(signature: &str) -> String {
    signature
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn remove_previous_pack_files(pack_dir: &Path) -> Result<()> {
    // This directory is an application-owned `<plugin>.json-pack`. Clear its
    // generated JSON files before rewriting so signatures removed since the
    // previous export cannot remain as misleading stale fragments. Preserve
    // non-JSON files a user may have placed alongside the pack.
    for entry in fs::read_dir(pack_dir)? {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            fs::remove_file(&path)
                .with_context(|| format!("removing stale pack JSON {}", path.display()))?;
        }
    }
    Ok(())
}
