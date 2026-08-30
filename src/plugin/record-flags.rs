use anyhow::{Context, Result, bail, ensure};

use super::RecordFlagsValue;

const TES4_FLAG_NAMES: &[(u32, &str)] = &[
    (1, "altered"),
    (2, "checked"),
    (3, "active"),
    (4, "optimized"),
    (5, "temporary_id_owner"),
    (7, "localized"),
    (8, "precalculated_data_only"),
    (20, "update"),
];

pub(crate) fn decode(bits: u32) -> RecordFlagsValue {
    RecordFlagsValue {
        plugin_type: match (bits & 1 != 0, bits & (1 << 9) != 0) {
            (false, false) => "esp",
            (true, false) => "esm",
            (false, true) => "esl",
            (true, true) => "esl_master",
        }
        .to_owned(),
        set: TES4_FLAG_NAMES
            .iter()
            .filter(|(bit, _)| bits & (1 << bit) != 0)
            .map(|(_, name)| (*name).to_owned())
            .collect(),
        unknown_bits: format!("0x{:08X}", bits & !known_mask()),
    }
}

pub(crate) fn encode(value: &RecordFlagsValue) -> Result<u32> {
    let digits = value
        .unknown_bits
        .strip_prefix("0x")
        .or_else(|| value.unknown_bits.strip_prefix("0X"));
    let mut bits = match digits {
        Some(digits) => u32::from_str_radix(digits, 16)?,
        None => value.unknown_bits.parse()?,
    };
    ensure!(
        bits & known_mask() == 0,
        "TES4 unknown_bits overlaps named flags"
    );
    for name in &value.set {
        let bit = TES4_FLAG_NAMES
            .iter()
            .find(|(_, candidate)| candidate == name)
            .map(|(bit, _)| *bit)
            .with_context(|| format!("unknown TES4 flag {name:?}"))?;
        bits |= 1 << bit;
    }
    bits |= match value.plugin_type.as_str() {
        "esp" => 0,
        "esm" => 1,
        "esl" => 1 << 9,
        "esl_master" => 1 | (1 << 9),
        other => bail!("unknown TES4 plugin_type {other:?}"),
    };
    Ok(bits)
}

fn known_mask() -> u32 {
    TES4_FLAG_NAMES
        .iter()
        .fold(1 | (1 << 9), |mask, (bit, _)| mask | (1 << bit))
}
