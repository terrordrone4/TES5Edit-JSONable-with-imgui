use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use serde::{Deserialize, Serialize};

#[path = "plugin/io-utils.rs"]
mod io_utils;
#[path = "plugin/json-pack.rs"]
mod json_pack;
#[path = "plugin/read.rs"]
mod read;
#[path = "plugin/record-flags.rs"]
mod record_flags;
#[path = "plugin/value_codec/mod.rs"]
mod value_codec;
#[path = "plugin/write.rs"]
mod write;

pub use json_pack::{
    JsonInputInfo, JsonPackWriteResult, inspect_json_input, read_json_input, write_json_pack,
};
pub use read::parse_file;
pub use value_codec::SubrecordValue;
pub use write::write_file;

const RECORD_HEADER_SIZE: usize = 24;
const COMPRESSED: u32 = 0x0004_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub format: String,
    pub source_file: Option<String>,
    pub elements: Vec<Element>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub blobs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Element {
    Record(Record),
    Group(Group),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub signature: String,
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub flags: u32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub flags_value: Option<RecordFlagsValue>,
    #[serde(
        default,
        with = "io_utils::hex_u32",
        skip_serializing_if = "io_utils::is_zero"
    )]
    pub form_id: u32,
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub revision: u32,
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub version: u16,
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub unknown: u16,
    #[serde(skip, default)]
    pub compressed: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub original_compressed_ref: Option<String>,
    #[serde(skip_serializing, default)]
    pub original_compressed_base64: Option<String>,
    pub subrecords: Vec<Subrecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordFlagsValue {
    /// ESM/ESL bit combination: esp, esm, esl, or esl_master.
    pub plugin_type: String,
    pub set: Vec<String>,
    pub unknown_bits: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subrecord {
    pub signature: String,
    /// xEdit's schema label for this record/subrecord pair when known.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub display_name: Option<String>,
    /// Schema-aware, editable representation of the payload when a lossless
    /// codec is known. Unknown payloads deliberately remain blob-only.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub value: Option<SubrecordValue>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data_ref: Option<String>,
    #[serde(skip_serializing, default)]
    pub data_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub text_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub label: u32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub label_signature: Option<String>,
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub group_type: i32,
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub stamp: u32,
    #[serde(default, skip_serializing_if = "io_utils::is_zero")]
    pub unknown: u32,
    pub elements: Vec<Element>,
}

pub(crate) fn parse_file_impl(path: impl AsRef<Path>) -> Result<Plugin> {
    let path = path.as_ref();
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let localized = bytes.len() >= 12 && le_u32(&bytes[8..12]) & 0x80 != 0;
    let mut blobs = BTreeMap::new();
    let mut next_blob = 1;
    let elements = parse_elements(&bytes, "plugin", localized, &mut blobs, &mut next_blob)?;
    ensure!(!elements.is_empty(), "plugin is empty");
    match &elements[0] {
        Element::Record(r) if r.signature == "TES4" => {}
        _ => bail!("a Skyrim plugin must begin with a TES4 record"),
    }
    Ok(Plugin {
        format: "tes5edit-rust-json/v2".into(),
        source_file: path.file_name().map(|s| s.to_string_lossy().into_owned()),
        elements,
        blobs,
    })
}

pub(crate) fn write_file_impl(plugin: &Plugin, path: impl AsRef<Path>) -> Result<()> {
    ensure!(
        matches!(
            plugin.format.as_str(),
            "tes5edit-rust-json/v1" | "tes5edit-rust-json/v2"
        ),
        "unsupported JSON format {}",
        plugin.format
    );
    ensure!(
        matches!(plugin.elements.first(), Some(Element::Record(r)) if r.signature == "TES4"),
        "plugin JSON must begin with a TES4 record"
    );
    let mut out = Vec::new();
    write_elements(&plugin.elements, &plugin.blobs, &mut out)?;
    fs::write(path.as_ref(), out).with_context(|| format!("writing {}", path.as_ref().display()))
}

fn parse_elements(
    mut input: &[u8],
    scope: &str,
    localized: bool,
    blobs: &mut BTreeMap<String, String>,
    next_blob: &mut u64,
) -> Result<Vec<Element>> {
    let mut result = Vec::new();
    while !input.is_empty() {
        ensure!(
            input.len() >= RECORD_HEADER_SIZE,
            "truncated header in {scope}: {} bytes remain",
            input.len()
        );
        let sig = signature(&input[0..4])?;
        if sig == "GRUP" {
            let size = le_u32(&input[4..8]) as usize;
            ensure!(
                size >= RECORD_HEADER_SIZE && size <= input.len(),
                "invalid GRUP size {size} in {scope}"
            );
            let label = le_u32(&input[8..12]);
            let group_type = i32::from_le_bytes(input[12..16].try_into().unwrap());
            let children = parse_elements(&input[24..size], "GRUP", localized, blobs, next_blob)?;
            result.push(Element::Group(Group {
                label,
                label_signature: (group_type == 0).then(|| display_signature(label.to_le_bytes())),
                group_type,
                stamp: le_u32(&input[16..20]),
                unknown: le_u32(&input[20..24]),
                elements: children,
            }));
            input = &input[size..];
        } else {
            let data_size = le_u32(&input[4..8]) as usize;
            let total = RECORD_HEADER_SIZE
                .checked_add(data_size)
                .context("record size overflow")?;
            ensure!(
                total <= input.len(),
                "truncated {sig} record: declares {data_size} data bytes"
            );
            let flags = le_u32(&input[8..12]);
            let compressed = flags & COMPRESSED != 0;
            let payload = if compressed {
                decompress_payload(&input[24..total], &sig)?
            } else {
                input[24..total].to_vec()
            };
            result.push(Element::Record(Record {
                signature: sig.clone(),
                flags,
                flags_value: (sig == "TES4").then(|| record_flags::decode(flags)),
                compressed,
                original_compressed_ref: compressed
                    .then(|| insert_blob(blobs, next_blob, &input[24..total])),
                original_compressed_base64: None,
                form_id: le_u32(&input[12..16]),
                revision: le_u32(&input[16..20]),
                version: le_u16(&input[20..22]),
                unknown: le_u16(&input[22..24]),
                subrecords: parse_subrecords(&payload, &sig, localized, blobs, next_blob)?,
            }));
            input = &input[total..];
        }
    }
    Ok(result)
}

fn parse_subrecords(
    mut data: &[u8],
    record: &str,
    localized: bool,
    blobs: &mut BTreeMap<String, String>,
    next_blob: &mut u64,
) -> Result<Vec<Subrecord>> {
    let mut out = Vec::new();
    let mut extended = None;
    let total_len = data.len();
    while !data.is_empty() {
        let offset = total_len - data.len();
        ensure!(
            data.len() >= 6,
            "truncated subrecord header in {record} at payload offset 0x{offset:X}"
        );
        let sig = signature(&data[..4]).with_context(|| {
            format!(
                "reading subrecord in {record} at payload offset 0x{offset:X}; next bytes {:02X?}",
                &data[..data.len().min(32)]
            )
        })?;
        let short_size = le_u16(&data[4..6]) as usize;
        data = &data[6..];
        let size = extended.take().unwrap_or(short_size);
        ensure!(
            size <= data.len(),
            "truncated {sig} subrecord in {record}: wants {size}, has {}",
            data.len()
        );
        let bytes = &data[..size];
        data = &data[size..];
        if sig == "XXXX" {
            ensure!(
                size == 4 && extended.is_none(),
                "invalid XXXX marker in {record}"
            );
            extended = Some(le_u32(bytes) as usize);
            continue;
        }
        out.push(Subrecord {
            value: value_codec::decode_with_localization(record, &sig, bytes, localized),
            display_name: value_codec::display_name(record, &sig).map(str::to_owned),
            signature: sig,
            data_ref: Some(insert_blob(blobs, next_blob, bytes)),
            data_base64: None,
            text_preview: text_preview(bytes),
        });
    }
    ensure!(extended.is_none(), "dangling XXXX marker in {record}");
    Ok(out)
}

fn insert_blob(blobs: &mut BTreeMap<String, String>, next_blob: &mut u64, bytes: &[u8]) -> String {
    let id = format!("blob_{:08}", *next_blob);
    *next_blob += 1;
    blobs.insert(id.clone(), BASE64.encode(bytes));
    id
}

fn resolve_blob<'a>(
    blobs: &'a BTreeMap<String, String>,
    data_ref: Option<&str>,
    legacy_inline: Option<&'a str>,
    context: &str,
) -> Result<&'a str> {
    if let Some(id) = data_ref {
        return blobs
            .get(id)
            .map(String::as_str)
            .with_context(|| format!("missing blob {id:?} referenced by {context}"));
    }
    legacy_inline.with_context(|| format!("{context} has neither data_ref nor inline data_base64"))
}

fn write_elements(
    elements: &[Element],
    blobs: &BTreeMap<String, String>,
    out: &mut Vec<u8>,
) -> Result<()> {
    for element in elements {
        match element {
            Element::Group(g) => {
                let start = out.len();
                out.extend_from_slice(b"GRUP");
                out.extend_from_slice(&0u32.to_le_bytes());
                out.extend_from_slice(&g.label.to_le_bytes());
                out.extend_from_slice(&g.group_type.to_le_bytes());
                out.extend_from_slice(&g.stamp.to_le_bytes());
                out.extend_from_slice(&g.unknown.to_le_bytes());
                write_elements(&g.elements, blobs, out)?;
                let size = u32::try_from(out.len() - start).context("group exceeds 4 GiB")?;
                out[start + 4..start + 8].copy_from_slice(&size.to_le_bytes());
            }
            Element::Record(r) => write_record(r, blobs, out)?,
        }
    }
    Ok(())
}

fn write_record(r: &Record, blobs: &BTreeMap<String, String>, out: &mut Vec<u8>) -> Result<()> {
    let sig = signature_bytes(&r.signature)?;
    let mut plain = Vec::new();
    for s in &r.subrecords {
        let ssig = signature_bytes(&s.signature)?;
        let bytes = if let Some(value) = &s.value {
            value_codec::encode(&r.signature, &s.signature, value)?
        } else {
            let encoded = resolve_blob(
                blobs,
                s.data_ref.as_deref(),
                s.data_base64.as_deref(),
                &format!("{}.{}", r.signature, s.signature),
            )?;
            BASE64
                .decode(encoded)
                .with_context(|| format!("invalid base64 in {}.{}", r.signature, s.signature))?
        };
        if bytes.len() > u16::MAX as usize {
            plain.extend_from_slice(b"XXXX");
            plain.extend_from_slice(&4u16.to_le_bytes());
            plain.extend_from_slice(
                &u32::try_from(bytes.len())
                    .context("subrecord exceeds 4 GiB")?
                    .to_le_bytes(),
            );
            plain.extend_from_slice(&ssig);
            plain.extend_from_slice(&0u16.to_le_bytes());
        } else {
            plain.extend_from_slice(&ssig);
            plain.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        }
        plain.extend_from_slice(&bytes);
    }
    let compressed = r.compressed || r.flags & COMPRESSED != 0;
    let payload = if compressed {
        if r.original_compressed_ref.is_some() || r.original_compressed_base64.is_some() {
            let original_base64 = resolve_blob(
                blobs,
                r.original_compressed_ref.as_deref(),
                r.original_compressed_base64.as_deref(),
                &format!("{} original compressed payload", r.signature),
            )?;
            let original = BASE64.decode(original_base64).with_context(|| {
                format!("invalid original compressed base64 in {}", r.signature)
            })?;
            if decompress_payload(&original, &r.signature)? == plain {
                original
            } else {
                compress_payload(&plain)?
            }
        } else {
            compress_payload(&plain)?
        }
    } else {
        plain
    };
    out.extend_from_slice(&sig);
    out.extend_from_slice(
        &u32::try_from(payload.len())
            .context("record exceeds 4 GiB")?
            .to_le_bytes(),
    );
    let base_flags = if let Some(value) = &r.flags_value {
        ensure!(
            r.signature == "TES4",
            "flags_value is only supported for TES4"
        );
        record_flags::encode(value)?
    } else {
        r.flags
    };
    let flags = if compressed {
        base_flags | COMPRESSED
    } else {
        base_flags & !COMPRESSED
    };
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&r.form_id.to_le_bytes());
    out.extend_from_slice(&r.revision.to_le_bytes());
    out.extend_from_slice(&r.version.to_le_bytes());
    out.extend_from_slice(&r.unknown.to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(())
}

fn compress_payload(plain: &[u8]) -> Result<Vec<u8>> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(
        &u32::try_from(plain.len())
            .context("record exceeds 4 GiB")?
            .to_le_bytes(),
    );
    let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
    z.write_all(plain)?;
    encoded.extend_from_slice(&z.finish()?);
    Ok(encoded)
}

fn decompress_payload(data: &[u8], sig: &str) -> Result<Vec<u8>> {
    ensure!(
        data.len() >= 4,
        "compressed {sig} record lacks its uncompressed-size prefix"
    );
    let expected = le_u32(&data[..4]) as usize;
    let mut decoded = Vec::with_capacity(expected);
    ZlibDecoder::new(&data[4..])
        .read_to_end(&mut decoded)
        .with_context(|| format!("decompressing {sig}"))?;
    ensure!(
        decoded.len() == expected,
        "{sig} expands to {} bytes, expected {expected}",
        decoded.len()
    );
    Ok(decoded)
}

fn signature(bytes: &[u8]) -> Result<String> {
    let a: [u8; 4] = bytes.try_into().context("signature is not four bytes")?;
    // xEdit's TwbSignature is exactly four arbitrary bytes, not a FourCC that
    // is guaranteed to be printable. TES5 IMAD records, for example, use
    // #$00'IAD' through #$54'IAD'. Map bytes one-to-one to Unicode code points
    // so serde_json writes controls as \u00XX and the representation remains
    // both valid JSON and lossless.
    Ok(a.into_iter().map(char::from).collect())
}
fn signature_bytes(s: &str) -> Result<[u8; 4]> {
    let chars: Vec<_> = s.chars().collect();
    ensure!(
        chars.len() == 4 && chars.iter().all(|c| u32::from(*c) <= 0xff),
        "signature must represent exactly four raw bytes: {s:?}"
    );
    Ok(chars
        .into_iter()
        .map(|c| u32::from(c) as u8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap())
}
fn display_signature(bytes: [u8; 4]) -> String {
    bytes.into_iter().map(char::from).collect()
}
fn le_u16(b: &[u8]) -> u16 {
    u16::from_le_bytes(b.try_into().unwrap())
}
fn le_u32(b: &[u8]) -> u32 {
    u32::from_le_bytes(b.try_into().unwrap())
}
fn text_preview(bytes: &[u8]) -> Option<String> {
    let body = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    if !body.is_empty()
        && body
            .iter()
            .all(|b| b.is_ascii_graphic() || b.is_ascii_whitespace())
    {
        Some(String::from_utf8_lossy(body).into_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(compressed: bool, large: bool) -> Plugin {
        let data = if large {
            vec![0x5a; 70_000]
        } else {
            b"hello\0".to_vec()
        };
        Plugin {
            format: "tes5edit-rust-json/v2".into(),
            source_file: None,
            elements: vec![
                Element::Record(Record {
                    signature: "TES4".into(),
                    flags: 0,
                    flags_value: None,
                    form_id: 0,
                    revision: 1,
                    version: 44,
                    unknown: 0,
                    compressed: false,
                    original_compressed_ref: None,
                    original_compressed_base64: None,
                    subrecords: vec![],
                }),
                Element::Group(Group {
                    label: u32::from_le_bytes(*b"STAT"),
                    label_signature: Some("STAT".into()),
                    group_type: 0,
                    stamp: 7,
                    unknown: 0,
                    elements: vec![Element::Record(Record {
                        signature: "STAT".into(),
                        flags: if compressed { COMPRESSED } else { 0 },
                        flags_value: None,
                        form_id: 0x123,
                        revision: 2,
                        version: 44,
                        unknown: 0,
                        compressed,
                        original_compressed_ref: None,
                        original_compressed_base64: None,
                        subrecords: vec![Subrecord {
                            signature: "EDID".into(),
                            display_name: None,
                            value: None,
                            data_ref: None,
                            data_base64: Some(BASE64.encode(data)),
                            text_preview: None,
                        }],
                    })],
                }),
            ],
            blobs: BTreeMap::new(),
        }
    }
    #[test]
    fn round_trip_plain_compressed_and_xxxx() {
        for p in [
            sample(false, false),
            sample(true, false),
            sample(false, true),
        ] {
            let mut bytes = vec![];
            write_elements(&p.elements, &p.blobs, &mut bytes).unwrap();
            let mut blobs = BTreeMap::new();
            let mut next_blob = 1;
            let parsed = parse_elements(&bytes, "test", false, &mut blobs, &mut next_blob).unwrap();
            let mut again = vec![];
            write_elements(&parsed, &blobs, &mut again).unwrap();
            let mut reparsed_blobs = BTreeMap::new();
            let mut reparsed_next = 1;
            assert_eq!(
                parse_elements(
                    &again,
                    "again",
                    false,
                    &mut reparsed_blobs,
                    &mut reparsed_next
                )
                .unwrap()
                .len(),
                2
            );
            assert_eq!(bytes, again);
        }
    }

    #[test]
    fn supports_xedit_control_byte_signatures() {
        for raw in [[0x00, b'I', b'A', b'D'], [0x40, b'I', b'A', b'D']] {
            let encoded = signature(&raw).unwrap();
            assert_eq!(signature_bytes(&encoded).unwrap(), raw);
            let json = serde_json::to_string(&encoded).unwrap();
            assert_eq!(serde_json::from_str::<String>(&json).unwrap(), encoded);
        }
    }

    #[test]
    fn form_ids_serialize_as_hex_and_accept_legacy_decimal() {
        let record = sample(false, false);
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"form_id\":\"0x00000123\""));
        assert!(!json.contains("\"form_id\":\"0x00000000\""));
        assert!(!json.contains("\"compressed\""));

        let legacy = json.replace("\"form_id\":\"0x00000123\"", "\"form_id\":291");
        let decoded: Plugin = serde_json::from_str(&legacy).unwrap();
        assert!(matches!(decoded.elements.first(), Some(Element::Record(r)) if r.form_id == 0));
        assert!(matches!(
            decoded.elements.get(1),
            Some(Element::Group(g))
                if matches!(g.elements.first(), Some(Element::Record(r)) if r.form_id == 0x123)
        ));
    }

    #[test]
    fn zero_numeric_header_fields_are_omitted_and_defaulted() {
        let plugin = sample(false, false);
        let json = serde_json::to_string(&plugin).unwrap();
        assert!(!json.contains("\"flags\":0"));
        assert!(!json.contains("\"revision\":0"));
        assert!(!json.contains("\"unknown\":0"));
        assert!(!json.contains("\"group_type\":0"));

        let decoded: Plugin = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(decoded.elements.first(), Some(Element::Record(r)) if r.flags == 0 && r.form_id == 0 && r.unknown == 0)
        );
        assert!(
            matches!(decoded.elements.get(1), Some(Element::Group(g)) if g.group_type == 0 && g.unknown == 0)
        );
    }

    #[test]
    fn tes4_flags_are_named_and_lossless() {
        let raw = (1 << 0) | (1 << 7) | (1 << 9) | (1 << 31);
        let value = record_flags::decode(raw);
        assert_eq!(value.plugin_type, "esl_master");
        assert_eq!(value.set, ["localized"]);
        assert_eq!(value.unknown_bits, "0x80000000");
        assert_eq!(record_flags::encode(&value).unwrap(), raw);
    }

    #[test]
    fn round_trip_every_example_skyrim_plugin() {
        fn collect_plugins(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect_plugins(&path, files);
                } else if path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                    matches!(e.to_ascii_lowercase().as_str(), "esp" | "esm" | "esl")
                }) {
                    files.push(path);
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/in");
        assert!(
            root.is_dir(),
            "fixture directory is missing: {}",
            root.display()
        );
        let mut files = Vec::new();
        collect_plugins(&root, &mut files);
        files.sort();
        assert!(
            !files.is_empty(),
            "no plugin fixtures found below {}",
            root.display()
        );

        for (fixture_index, path) in files.into_iter().enumerate() {
            let original = fs::read(&path).unwrap();
            let plugin = parse_file(&path)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e:#}", path.display()));
            let json = serde_json::to_vec(&plugin).unwrap();
            let from_json: Plugin = serde_json::from_slice(&json).unwrap();
            let mut rebuilt = Vec::new();
            write_elements(&from_json.elements, &from_json.blobs, &mut rebuilt)
                .unwrap_or_else(|e| panic!("failed to rebuild {}: {e:#}", path.display()));
            if rebuilt != original {
                let offset = rebuilt
                    .iter()
                    .zip(&original)
                    .position(|(a, b)| a != b)
                    .unwrap_or(rebuilt.len().min(original.len()));
                panic!(
                    "round trip changed {} at file offset 0x{offset:X} (rebuilt length {}, original length {})",
                    path.display(),
                    rebuilt.len(),
                    original.len()
                );
            }
            let mut rebuilt_blobs = BTreeMap::new();
            let mut rebuilt_next = 1;
            parse_elements(
                &rebuilt,
                "rebuilt fixture",
                false,
                &mut rebuilt_blobs,
                &mut rebuilt_next,
            )
            .unwrap_or_else(|e| panic!("rebuilt {} is invalid: {e:#}", path.display()));

            let pack_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/json-pack-tests")
                .join(format!("fixture-{fixture_index}"));
            let pack_result = write_json_pack(&plugin, &pack_dir)
                .unwrap_or_else(|e| panic!("failed to pack {}: {e:#}", path.display()));
            let actual_json_count = fs::read_dir(&pack_dir)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
                .count();
            assert_eq!(pack_result.json_file_count, actual_json_count);
            assert!(pack_dir.join("__blobs__.json").is_file());
            let tes4_fragment = fs::read_to_string(pack_dir.join("TES4.json")).unwrap();
            assert!(!tes4_fragment.contains("\"blobs\""));
            assert!(tes4_fragment.contains("\"data_ref\""));
            let pack_info = inspect_json_input(&pack_dir)
                .unwrap_or_else(|e| panic!("failed to inspect pack for {}: {e:#}", path.display()));
            assert!(pack_info.is_pack);
            let from_pack = read_json_input(&pack_dir)
                .unwrap_or_else(|e| panic!("failed to reload pack for {}: {e:#}", path.display()));
            let mut rebuilt_from_pack = Vec::new();
            write_elements(
                &from_pack.elements,
                &from_pack.blobs,
                &mut rebuilt_from_pack,
            )
            .unwrap_or_else(|e| panic!("failed to rebuild pack for {}: {e:#}", path.display()));
            assert_eq!(
                rebuilt_from_pack,
                original,
                "JSON pack round trip changed {}",
                path.display()
            );
        }
    }
}
