pub(super) fn get(record: &str, signature: &str) -> Option<&'static str> {
    match (record, signature) {
        ("TES4", "HEDR") => Some("Header"),
        ("TES4", "CNAM") => Some("Author"),
        ("TES4", "SNAM") => Some("Description"),
        ("TES4", "MAST") => Some("Master File"),
        ("TES4", "DATA") => Some("Master Data"),
        ("TES4", "ONAM") => Some("Overridden Forms"),
        ("TES4", "OFST" | "DELE") => Some("Unknown"),
        ("TES4", "SCRN") => Some("Screenshot"),
        ("TES4", "INTV") => Some("Unknown"),
        ("TES4", "INCC") => Some("Interior Cell Count"),
        _ => None,
    }
}
