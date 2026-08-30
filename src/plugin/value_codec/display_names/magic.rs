pub(super) fn get(record: &str, signature: &str) -> Option<&'static str> {
    match (record, signature) {
        ("MGEF", "DATA") => Some("Data"),
        ("MGEF", "ESCE") => Some("Counter Effect"),
        ("MGEF", "FULL") => Some("Name"),
        ("MGEF", "DNAM") => Some("Magic Item Description"),
        ("MGEF", "MDOB") => Some("Menu Display Object"),
        ("MGEF", "SNDD") => Some("Sounds"),
        ("SPEL", "SPIT") => Some("Data"),
        ("SPEL", "EFID") => Some("Base Effect"),
        ("SPEL", "EFIT") => Some("Effect Parameters"),
        ("SPEL", "FULL") => Some("Name"),
        ("SPEL", "DESC") => Some("Description"),
        ("SPEL", "MDOB") => Some("Menu Display Object"),
        ("SPEL", "ETYP") => Some("Equipment Type"),
        _ => None,
    }
}
