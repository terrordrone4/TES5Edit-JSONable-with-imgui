pub(super) fn get(record: &str, signature: &str) -> Option<&'static str> {
    match (record, signature) {
        ("LVLN" | "LVLI", "LVLD") => Some("Chance None"),
        ("LVLN" | "LVLI", "LVLF") => Some("Flags"),
        ("LVLN" | "LVLI", "LVLG") => Some("Global"),
        ("LVLN" | "LVLI", "LLCT") => Some("Leveled List Entry Count"),
        ("LVLN" | "LVLI", "LVLO") => Some("Leveled List Entry"),
        ("LVLN" | "LVLI", "COED") => Some("Extra Data"),
        ("OTFT", "INAM") => Some("Items"),
        ("ARMO", "DATA") => Some("Data"),
        ("ARMO", "DNAM") => Some("Armor Rating"),
        ("ARMO", "RNAM") => Some("Race"),
        ("ARMO", "MODL") => Some("Armature"),
        ("ARMO", "BOD2") => Some("Biped Body Template"),
        ("ARMO", "FULL") => Some("Name"),
        ("ARMO", "DESC") => Some("Description"),
        ("ARMO", "MOD2") => Some("Male World Model"),
        ("ARMO", "MOD4") => Some("Female World Model"),
        ("ARMO", "YNAM") => Some("Pickup Sound"),
        ("ARMO", "ZNAM") => Some("Drop Sound"),
        _ => None,
    }
}
