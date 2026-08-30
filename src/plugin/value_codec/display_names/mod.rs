mod actors;
mod header;
mod items;
mod magic;
mod world;

pub fn get(record: &str, signature: &str) -> Option<&'static str> {
    if signature == "EDID" {
        return Some("Editor ID");
    }
    header::get(record, signature)
        .or_else(|| actors::get(record, signature))
        .or_else(|| items::get(record, signature))
        .or_else(|| magic::get(record, signature))
        .or_else(|| world::get(record, signature))
        .or_else(|| match (record, signature) {
            ("NPC_" | "ARMO" | "MGEF" | "SPEL" | "RACE", "KSIZ") => Some("Keyword Count"),
            ("NPC_" | "ARMO" | "MGEF" | "SPEL" | "RACE", "KWDA") => Some("Keywords"),
            (_, "OBND") => Some("Object Bounds"),
            _ => None,
        })
}
