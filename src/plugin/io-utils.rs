use std::path::Path;

use anyhow::{Result, ensure};

pub(super) fn is_zero<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    value == &T::default()
}

pub(super) mod hex_u32 {
    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{value:08X}"))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum HexOrDecimal {
            Hex(String),
            Decimal(u32),
        }

        match HexOrDecimal::deserialize(deserializer)? {
            HexOrDecimal::Decimal(value) => Ok(value),
            HexOrDecimal::Hex(value) => {
                let digits = value
                    .strip_prefix("0x")
                    .or_else(|| value.strip_prefix("0X"))
                    .ok_or_else(|| {
                        de::Error::custom("expected a hexadecimal string beginning with 0x")
                    })?;
                u32::from_str_radix(digits, 16).map_err(de::Error::custom)
            }
        }
    }
}

pub(super) fn ensure_supported_plugin_path(path: &Path) -> Result<()> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    ensure!(
        matches!(extension.as_str(), "esp" | "esm" | "esl"),
        "expected an .esp, .esm, or .esl path: {}",
        path.display()
    );
    Ok(())
}
