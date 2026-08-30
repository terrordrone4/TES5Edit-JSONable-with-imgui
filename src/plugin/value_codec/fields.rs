pub(super) fn is_form_id(record: &str, signature: &str) -> bool {
    match record {
        "LVLN" | "LVLI" => signature == "LVLG",
        "ARMO" => matches!(
            signature,
            "EITM" | "YNAM" | "ZNAM" | "ETYP" | "BIDS" | "BAMT" | "RNAM" | "MODL" | "TNAM"
        ),
        "NPC_" => matches!(
            signature,
            "INAM"
                | "VTCK"
                | "TPLT"
                | "RNAM"
                | "SPLO"
                | "WNAM"
                | "ANAM"
                | "ATKR"
                | "SPOR"
                | "OCOR"
                | "GWOR"
                | "ECOR"
                | "PKID"
                | "CNAM"
                | "PNAM"
                | "HCLF"
                | "ZNAM"
                | "GNAM"
                | "CSCR"
                | "DOFT"
                | "SOFT"
                | "DPLT"
                | "CRIF"
                | "FTST"
        ),
        "MGEF" => matches!(signature, "MDOB" | "ESCE"),
        "SPEL" => matches!(signature, "MDOB" | "ETYP" | "EFID"),
        "RACE" => matches!(
            signature,
            "SPLO"
                | "WNAM"
                | "ATKR"
                | "MTYP"
                | "QNAM"
                | "UNES"
                | "WKMV"
                | "RNMV"
                | "SWMV"
                | "FLMV"
                | "SNMV"
                | "SPMV"
                | "RPRM"
                | "AHCM"
                | "FTSM"
                | "DFTM"
                | "RPRF"
                | "AHCF"
                | "FTSF"
                | "DFTF"
                | "NAM8"
                | "RNAM"
                | "GNAM"
                | "NAM4"
                | "NAM5"
                | "NAM7"
                | "ONAM"
                | "LNAM"
                | "TIND"
                | "TINC"
                | "HEAD"
        ),
        "FACT" => matches!(
            signature,
            "JAIL" | "WAIT" | "STOL" | "PLCN" | "CRGR" | "JOUT" | "VEND" | "VENC"
        ),
        _ => false,
    }
}

pub(super) fn is_form_id_array(record: &str, signature: &str) -> bool {
    matches!(
        (record, signature),
        ("TES4", "ONAM")
            | ("OTFT", "INAM")
            | ("NPC_" | "ARMO" | "MGEF" | "SPEL" | "RACE", "KWDA")
            | ("RACE", "VTCK" | "DNAM" | "HCLF" | "HNAM" | "ENAM")
    )
}

pub(super) fn is_zstring(record: &str, signature: &str) -> bool {
    matches!(
        (record, signature),
        ("TES4", "CNAM" | "SNAM" | "MAST")
            | (
                "ARMO",
                "MOD2" | "ICON" | "MICO" | "MOD4" | "ICO2" | "MIC2" | "BMCT"
            )
            | (
                "RACE",
                "ANAM" | "MTNM" | "MODL" | "NAME" | "PHTN" | "TINT" | "ATKE"
            )
    )
}

pub(super) fn is_localized_string(record: &str, signature: &str) -> bool {
    matches!(
        (record, signature),
        ("NPC_", "FULL" | "SHRT")
            | ("ARMO", "FULL" | "DESC")
            | ("MGEF", "FULL" | "DNAM")
            | ("SPEL", "FULL" | "DESC")
            | ("RACE", "FULL" | "DESC")
            | ("FACT", "FULL" | "MNAM" | "FNAM")
    )
}

pub(super) fn is_empty(record: &str, signature: &str) -> bool {
    matches!(
        (record, signature),
        ("NPC_", "DATA") | ("RACE", "MNAM" | "FNAM" | "NAM0" | "NAM1" | "NAM2" | "NAM3")
    )
}

pub(super) fn is_u32(record: &str, signature: &str) -> bool {
    matches!(
        (record, signature),
        ("TES4", "INCC")
            | ("ARMO", "KSIZ")
            | ("NPC_", "SPCT" | "PRKZ" | "COCT" | "KSIZ" | "NAM8" | "TINV")
            | ("MGEF", "KSIZ")
            | ("SPEL", "KSIZ")
            | ("RACE", "SPCT" | "KSIZ" | "VNAM" | "INDX")
    )
}

pub(super) fn is_f32(record: &str, signature: &str) -> bool {
    matches!(
        (record, signature),
        ("NPC_", "NAM6" | "NAM7") | ("RACE", "PNAM" | "UNAM" | "TINV")
    )
}

pub(super) fn is_u16(record: &str, signature: &str) -> bool {
    matches!(
        (record, signature),
        ("NPC_", "TINI") | ("RACE", "TINL" | "TINI" | "TINP" | "TIRS")
    )
}

pub(super) fn is_i16(record: &str, signature: &str) -> bool {
    matches!((record, signature), ("NPC_", "TIAS"))
}
