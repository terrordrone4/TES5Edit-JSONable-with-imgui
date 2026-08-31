use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptObject {
    pub form_id: String,
    #[serde(default = "default_alias", skip_serializing_if = "is_default_alias")]
    pub alias: i16,
    #[serde(default = "default_unused", skip_serializing_if = "is_default_unused")]
    pub unused: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    None,
    Object { object: ScriptObject },
    String { string: String },
    Integer { integer: i32 },
    Float { float: f32 },
    Boolean { boolean: bool },
    Objects { objects: Vec<ScriptObject> },
    Strings { strings: Vec<String> },
    Integers { integers: Vec<i32> },
    Floats { floats: Vec<f32> },
    Booleans { booleans: Vec<bool> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptProperty {
    pub property_name: String,
    pub property_type: String,
    #[serde(
        default = "default_property_flags",
        skip_serializing_if = "is_default_property_flags"
    )]
    pub flags: String,
    pub value: PropertyValue,
}

fn default_alias() -> i16 {
    -1
}

fn is_default_alias(value: &i16) -> bool {
    *value == default_alias()
}

fn default_unused() -> String {
    "0x0000".to_owned()
}

fn is_default_unused(value: &str) -> bool {
    value.is_empty() || value.eq_ignore_ascii_case("0x0000") || value == "0000"
}

fn default_property_flags() -> String {
    "edited".to_owned()
}

fn is_default_property_flags(value: &str) -> bool {
    value == default_property_flags()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    pub script_name: String,
    pub flags: String,
    pub properties: Vec<ScriptProperty>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestFragment {
    pub quest_stage: u32,
    pub log_entry: u32,
    pub unknown: u8,
    pub script_name: String,
    pub fragment_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestFragments {
    pub extra_bind_data_version: i8,
    pub file_name: String,
    pub fragments: Vec<QuestFragment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestAlias {
    pub object: ScriptObject,
    pub version: i16,
    pub object_format: i16,
    pub scripts: Vec<Script>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestVmad {
    pub version: i16,
    pub object_format: i16,
    pub scripts: Vec<Script>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quest_fragments: Option<QuestFragments>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<QuestAlias>,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .context("VMAD offset overflow")?;
        ensure!(
            end <= self.bytes.len(),
            "truncated VMAD at offset 0x{:X}",
            self.offset
        );
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn i8(&mut self) -> Result<i8> {
        Ok(self.u8()? as i8)
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32> {
        let value = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
        ensure!(value.is_finite(), "VMAD contains a non-finite float");
        Ok(value)
    }

    fn string(&mut self) -> Result<String> {
        let length = self.u16()? as usize;
        Ok(std::str::from_utf8(self.take(length)?)
            .context("VMAD string is not valid UTF-8")?
            .to_owned())
    }
}

pub fn decode_quest(bytes: &[u8]) -> Option<QuestVmad> {
    decode_quest_result(bytes).ok()
}

fn decode_quest_result(bytes: &[u8]) -> Result<QuestVmad> {
    let mut cursor = Cursor::new(bytes);
    let version = cursor.i16()?;
    let object_format = cursor.i16()?;
    ensure!(
        matches!(object_format, 1 | 2),
        "unsupported VMAD object format {object_format}"
    );
    let scripts = decode_scripts(&mut cursor, object_format)?;
    if cursor.remaining() == 0 {
        return Ok(QuestVmad {
            version,
            object_format,
            scripts,
            quest_fragments: None,
            aliases: Vec::new(),
        });
    }

    let extra_bind_data_version = cursor.i8()?;
    let fragment_count = cursor.u16()? as usize;
    let file_name = cursor.string()?;
    let mut fragments = Vec::with_capacity(fragment_count);
    for _ in 0..fragment_count {
        fragments.push(QuestFragment {
            quest_stage: cursor.u32()?,
            log_entry: cursor.u32()?,
            unknown: cursor.u8()?,
            script_name: cursor.string()?,
            fragment_name: cursor.string()?,
        });
    }
    let alias_count = cursor.u16()? as usize;
    let mut aliases = Vec::with_capacity(alias_count);
    for _ in 0..alias_count {
        let object = decode_object(&mut cursor, object_format)?;
        let alias_version = cursor.i16()?;
        let alias_object_format = cursor.i16()?;
        ensure!(
            matches!(alias_object_format, 1 | 2),
            "unsupported alias VMAD object format {alias_object_format}"
        );
        aliases.push(QuestAlias {
            object,
            version: alias_version,
            object_format: alias_object_format,
            scripts: decode_scripts(&mut cursor, alias_object_format)?,
        });
    }
    ensure!(
        cursor.remaining() == 0,
        "VMAD has {} trailing bytes",
        cursor.remaining()
    );
    Ok(QuestVmad {
        version,
        object_format,
        scripts,
        quest_fragments: Some(QuestFragments {
            extra_bind_data_version,
            file_name,
            fragments,
        }),
        aliases,
    })
}

fn decode_scripts(cursor: &mut Cursor<'_>, object_format: i16) -> Result<Vec<Script>> {
    let count = cursor.u16()? as usize;
    let mut scripts = Vec::with_capacity(count);
    for _ in 0..count {
        let script_name = cursor.string()?;
        let flags = script_flag(cursor.u8()?)?.to_owned();
        let property_count = cursor.u16()? as usize;
        let mut properties = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            let property_name = cursor.string()?;
            let property_type_raw = cursor.u8()?;
            let flags = property_flag(cursor.u8()?)?.to_owned();
            let value = decode_property_value(cursor, object_format, property_type_raw)?;
            properties.push(ScriptProperty {
                property_name,
                property_type: property_type_name(property_type_raw)?.to_owned(),
                flags,
                value,
            });
        }
        scripts.push(Script {
            script_name,
            flags,
            properties,
        });
    }
    Ok(scripts)
}

fn decode_property_value(
    cursor: &mut Cursor<'_>,
    object_format: i16,
    property_type: u8,
) -> Result<PropertyValue> {
    Ok(match property_type {
        0 => PropertyValue::None,
        1 => PropertyValue::Object {
            object: decode_object(cursor, object_format)?,
        },
        2 => PropertyValue::String {
            string: cursor.string()?,
        },
        3 => PropertyValue::Integer {
            integer: cursor.i32()?,
        },
        4 => PropertyValue::Float {
            float: cursor.f32()?,
        },
        5 => PropertyValue::Boolean {
            boolean: decode_bool(cursor.u8()?)?,
        },
        11 => PropertyValue::Objects {
            objects: decode_array(cursor, |cursor| decode_object(cursor, object_format))?,
        },
        12 => PropertyValue::Strings {
            strings: decode_array(cursor, |cursor| cursor.string())?,
        },
        13 => PropertyValue::Integers {
            integers: decode_array(cursor, |cursor| cursor.i32())?,
        },
        14 => PropertyValue::Floats {
            floats: decode_array(cursor, |cursor| cursor.f32())?,
        },
        15 => PropertyValue::Booleans {
            booleans: decode_array(cursor, |cursor| decode_bool(cursor.u8()?))?,
        },
        value => bail!("unsupported VMAD property type {value}"),
    })
}

fn decode_array<T>(
    cursor: &mut Cursor<'_>,
    mut decode: impl FnMut(&mut Cursor<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let count = cursor.u32()? as usize;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(decode(cursor)?);
    }
    Ok(values)
}

fn decode_object(cursor: &mut Cursor<'_>, object_format: i16) -> Result<ScriptObject> {
    let (form_id, alias, unused) = if object_format == 2 {
        let unused = hex_bytes(cursor.take(2)?);
        let alias = cursor.i16()?;
        let form_id = hex_u32(cursor.u32()?);
        (form_id, alias, unused)
    } else {
        let form_id = hex_u32(cursor.u32()?);
        let alias = cursor.i16()?;
        let unused = hex_bytes(cursor.take(2)?);
        (form_id, alias, unused)
    };
    Ok(ScriptObject {
        form_id,
        alias,
        unused,
    })
}

pub fn encode_quest(value: &QuestVmad) -> Result<Vec<u8>> {
    ensure!(
        matches!(value.object_format, 1 | 2),
        "unsupported VMAD object format {}",
        value.object_format
    );
    let mut out = Vec::new();
    out.extend_from_slice(&value.version.to_le_bytes());
    out.extend_from_slice(&value.object_format.to_le_bytes());
    encode_scripts(&mut out, &value.scripts, value.object_format)?;
    match &value.quest_fragments {
        None => ensure!(
            value.aliases.is_empty(),
            "QUST VMAD aliases require quest_fragments"
        ),
        Some(fragment_data) => {
            out.push(fragment_data.extra_bind_data_version as u8);
            push_u16_count(
                &mut out,
                fragment_data.fragments.len(),
                "VMAD quest fragments",
            )?;
            push_string(&mut out, &fragment_data.file_name)?;
            for fragment in &fragment_data.fragments {
                out.extend_from_slice(&fragment.quest_stage.to_le_bytes());
                out.extend_from_slice(&fragment.log_entry.to_le_bytes());
                out.push(fragment.unknown);
                push_string(&mut out, &fragment.script_name)?;
                push_string(&mut out, &fragment.fragment_name)?;
            }
            push_u16_count(&mut out, value.aliases.len(), "VMAD aliases")?;
            for alias in &value.aliases {
                encode_object(&mut out, &alias.object, value.object_format)?;
                ensure!(
                    matches!(alias.object_format, 1 | 2),
                    "unsupported alias object format {}",
                    alias.object_format
                );
                out.extend_from_slice(&alias.version.to_le_bytes());
                out.extend_from_slice(&alias.object_format.to_le_bytes());
                encode_scripts(&mut out, &alias.scripts, alias.object_format)?;
            }
        }
    }
    Ok(out)
}

fn encode_scripts(out: &mut Vec<u8>, scripts: &[Script], object_format: i16) -> Result<()> {
    push_u16_count(out, scripts.len(), "VMAD scripts")?;
    for script in scripts {
        push_string(out, &script.script_name)?;
        out.push(parse_script_flag(&script.flags)?);
        push_u16_count(out, script.properties.len(), "VMAD properties")?;
        for property in &script.properties {
            push_string(out, &property.property_name)?;
            let property_type = parse_property_type(&property.property_type)?;
            out.push(property_type);
            out.push(parse_property_flag(&property.flags)?);
            encode_property_value(out, object_format, property_type, &property.value)?;
        }
    }
    Ok(())
}

fn encode_property_value(
    out: &mut Vec<u8>,
    object_format: i16,
    property_type: u8,
    value: &PropertyValue,
) -> Result<()> {
    match (property_type, value) {
        (0, PropertyValue::None) => {}
        (1, PropertyValue::Object { object }) => encode_object(out, object, object_format)?,
        (2, PropertyValue::String { string }) => push_string(out, string)?,
        (3, PropertyValue::Integer { integer }) => out.extend_from_slice(&integer.to_le_bytes()),
        (4, PropertyValue::Float { float }) => {
            ensure!(float.is_finite(), "VMAD property float must be finite");
            out.extend_from_slice(&float.to_le_bytes());
        }
        (5, PropertyValue::Boolean { boolean }) => out.push(u8::from(*boolean)),
        (11, PropertyValue::Objects { objects }) => {
            push_u32_count(out, objects.len(), "VMAD object array")?;
            for object in objects {
                encode_object(out, object, object_format)?;
            }
        }
        (12, PropertyValue::Strings { strings }) => {
            push_u32_count(out, strings.len(), "VMAD string array")?;
            for string in strings {
                push_string(out, string)?;
            }
        }
        (13, PropertyValue::Integers { integers }) => {
            push_u32_count(out, integers.len(), "VMAD integer array")?;
            for integer in integers {
                out.extend_from_slice(&integer.to_le_bytes());
            }
        }
        (14, PropertyValue::Floats { floats }) => {
            push_u32_count(out, floats.len(), "VMAD float array")?;
            for float in floats {
                ensure!(float.is_finite(), "VMAD array float must be finite");
                out.extend_from_slice(&float.to_le_bytes());
            }
        }
        (15, PropertyValue::Booleans { booleans }) => {
            push_u32_count(out, booleans.len(), "VMAD boolean array")?;
            out.extend(booleans.iter().map(|value| u8::from(*value)));
        }
        _ => bail!(
            "VMAD property_type {} does not match its value object",
            property_type_name(property_type)?
        ),
    }
    Ok(())
}

fn encode_object(out: &mut Vec<u8>, object: &ScriptObject, object_format: i16) -> Result<()> {
    let unused = parse_hex_bytes(&object.unused, 2)?;
    let form_id = parse_hex_u32(&object.form_id)?;
    if object_format == 2 {
        out.extend_from_slice(&unused);
        out.extend_from_slice(&object.alias.to_le_bytes());
        out.extend_from_slice(&form_id.to_le_bytes());
    } else {
        out.extend_from_slice(&form_id.to_le_bytes());
        out.extend_from_slice(&object.alias.to_le_bytes());
        out.extend_from_slice(&unused);
    }
    Ok(())
}

fn script_flag(value: u8) -> Result<&'static str> {
    Ok(match value {
        0 => "local",
        1 => "inherited",
        2 => "removed",
        3 => "inherited_and_removed",
        _ => bail!("unsupported VMAD script flag {value}"),
    })
}

fn parse_script_flag(value: &str) -> Result<u8> {
    match value {
        "local" => Ok(0),
        "inherited" => Ok(1),
        "removed" => Ok(2),
        "inherited_and_removed" => Ok(3),
        _ => bail!("unknown VMAD script flag {value:?}"),
    }
}

fn property_flag(value: u8) -> Result<&'static str> {
    Ok(match value {
        0 => "none",
        1 => "edited",
        2 => "unknown_2",
        3 => "removed",
        _ => bail!("unsupported VMAD property flag {value}"),
    })
}

fn parse_property_flag(value: &str) -> Result<u8> {
    match value {
        "none" => Ok(0),
        "edited" => Ok(1),
        "unknown_2" => Ok(2),
        "removed" => Ok(3),
        _ => bail!("unknown VMAD property flag {value:?}"),
    }
}

fn property_type_name(value: u8) -> Result<&'static str> {
    Ok(match value {
        0 => "none",
        1 => "object",
        2 => "string",
        3 => "int32",
        4 => "float",
        5 => "bool",
        11 => "object_array",
        12 => "string_array",
        13 => "int32_array",
        14 => "float_array",
        15 => "bool_array",
        _ => bail!("unsupported VMAD property type {value}"),
    })
}

fn parse_property_type(value: &str) -> Result<u8> {
    match value {
        "none" => Ok(0),
        "object" => Ok(1),
        "string" => Ok(2),
        "int32" => Ok(3),
        "float" => Ok(4),
        "bool" => Ok(5),
        "object_array" => Ok(11),
        "string_array" => Ok(12),
        "int32_array" => Ok(13),
        "float_array" => Ok(14),
        "bool_array" => Ok(15),
        _ => bail!("unknown VMAD property type {value:?}"),
    }
}

fn decode_bool(value: u8) -> Result<bool> {
    ensure!(value <= 1, "VMAD boolean must be 0 or 1, got {value}");
    Ok(value != 0)
}

fn push_string(out: &mut Vec<u8>, value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    push_u16_count(out, bytes.len(), "VMAD string bytes")?;
    out.extend_from_slice(bytes);
    Ok(())
}

fn push_u16_count(out: &mut Vec<u8>, count: usize, context: &str) -> Result<()> {
    out.extend_from_slice(
        &u16::try_from(count)
            .with_context(|| format!("{context} exceed 65535"))?
            .to_le_bytes(),
    );
    Ok(())
}

fn push_u32_count(out: &mut Vec<u8>, count: usize, context: &str) -> Result<()> {
    out.extend_from_slice(
        &u32::try_from(count)
            .with_context(|| format!("{context} exceed u32"))?
            .to_le_bytes(),
    );
    Ok(())
}

fn hex_u32(value: u32) -> String {
    format!("0x{value:08X}")
}

fn parse_hex_u32(value: &str) -> Result<u32> {
    if value.is_empty() {
        return Ok(0);
    }
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"));
    match digits {
        Some(digits) => Ok(u32::from_str_radix(digits, 16)?),
        None => Ok(value.parse()?),
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut value = String::from("0x");
    for byte in bytes {
        value.push_str(&format!("{byte:02X}"));
    }
    value
}

fn parse_hex_bytes(value: &str, expected: usize) -> Result<Vec<u8>> {
    if value.is_empty() {
        return Ok(vec![0; expected]);
    }
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);
    ensure!(
        digits.len() == expected * 2,
        "expected {expected} hexadecimal bytes"
    );
    (0..expected)
        .map(|index| Ok(u8::from_str_radix(&digits[index * 2..index * 2 + 2], 16)?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(form_id: &str, alias: i16) -> ScriptObject {
        ScriptObject {
            form_id: form_id.to_owned(),
            alias,
            unused: "0x0000".to_owned(),
        }
    }

    #[test]
    fn quest_vmad_all_property_types_fragments_and_aliases_round_trip() {
        let properties = vec![
            ScriptProperty {
                property_name: "None".into(),
                property_type: "none".into(),
                flags: "none".into(),
                value: PropertyValue::None,
            },
            ScriptProperty {
                property_name: "Object".into(),
                property_type: "object".into(),
                flags: "edited".into(),
                value: PropertyValue::Object {
                    object: object("0x01001234", -1),
                },
            },
            ScriptProperty {
                property_name: "String".into(),
                property_type: "string".into(),
                flags: "removed".into(),
                value: PropertyValue::String {
                    string: "hello".into(),
                },
            },
            ScriptProperty {
                property_name: "Int".into(),
                property_type: "int32".into(),
                flags: "unknown_2".into(),
                value: PropertyValue::Integer { integer: -42 },
            },
            ScriptProperty {
                property_name: "Float".into(),
                property_type: "float".into(),
                flags: "none".into(),
                value: PropertyValue::Float { float: 1.25 },
            },
            ScriptProperty {
                property_name: "Bool".into(),
                property_type: "bool".into(),
                flags: "none".into(),
                value: PropertyValue::Boolean { boolean: true },
            },
            ScriptProperty {
                property_name: "Objects".into(),
                property_type: "object_array".into(),
                flags: "none".into(),
                value: PropertyValue::Objects {
                    objects: vec![object("0x02000001", 3)],
                },
            },
            ScriptProperty {
                property_name: "Strings".into(),
                property_type: "string_array".into(),
                flags: "none".into(),
                value: PropertyValue::Strings {
                    strings: vec!["a".into(), "b".into()],
                },
            },
            ScriptProperty {
                property_name: "Ints".into(),
                property_type: "int32_array".into(),
                flags: "none".into(),
                value: PropertyValue::Integers {
                    integers: vec![-1, 2],
                },
            },
            ScriptProperty {
                property_name: "Floats".into(),
                property_type: "float_array".into(),
                flags: "none".into(),
                value: PropertyValue::Floats {
                    floats: vec![0.5, 2.0],
                },
            },
            ScriptProperty {
                property_name: "Bools".into(),
                property_type: "bool_array".into(),
                flags: "none".into(),
                value: PropertyValue::Booleans {
                    booleans: vec![true, false],
                },
            },
        ];
        let value = QuestVmad {
            version: 5,
            object_format: 2,
            scripts: vec![Script {
                script_name: "QuestScript".into(),
                flags: "local".into(),
                properties,
            }],
            quest_fragments: Some(QuestFragments {
                extra_bind_data_version: 3,
                file_name: "QF_Test_01001234".into(),
                fragments: vec![QuestFragment {
                    quest_stage: 10,
                    log_entry: 2,
                    unknown: 1,
                    script_name: "QF_Test".into(),
                    fragment_name: "Fragment_0".into(),
                }],
            }),
            aliases: vec![QuestAlias {
                object: object("0x01001234", 7),
                version: 5,
                object_format: 1,
                scripts: vec![Script {
                    script_name: "AliasScript".into(),
                    flags: "inherited".into(),
                    properties: Vec::new(),
                }],
            }],
        };
        let bytes = encode_quest(&value).unwrap();
        assert_eq!(decode_quest_result(&bytes).unwrap(), value);

        let json = serde_json::to_value(&value).unwrap();
        let object = &json["scripts"][0]["properties"][1]["value"]["object"];
        assert_eq!(object, &serde_json::json!({ "form_id": "0x01001234" }));
        assert!(json["scripts"][0]["properties"][1].get("flags").is_none());
        assert_eq!(serde_json::from_value::<QuestVmad>(json).unwrap(), value);
    }

    #[test]
    fn quest_vmad_rejects_property_type_value_mismatch() {
        let value = QuestVmad {
            version: 5,
            object_format: 2,
            scripts: vec![Script {
                script_name: "BadScript".into(),
                flags: "local".into(),
                properties: vec![ScriptProperty {
                    property_name: "Bad".into(),
                    property_type: "int32".into(),
                    flags: "none".into(),
                    value: PropertyValue::String {
                        string: "wrong".into(),
                    },
                }],
            }],
            quest_fragments: None,
            aliases: Vec::new(),
        };
        assert!(
            encode_quest(&value)
                .unwrap_err()
                .to_string()
                .contains("does not match")
        );
    }
}
