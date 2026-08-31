use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};

mod display_names;
mod fields;

use fields::{
    is_empty as is_empty_field, is_f32 as is_f32_field, is_form_id as is_form_id_field,
    is_form_id_array as is_form_id_array_field, is_i16 as is_i16_field,
    is_localized_string as is_localized_string_field, is_u16 as is_u16_field,
    is_u32 as is_u32_field, is_zstring as is_zstring_field,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcSkill {
    pub name: String,
    pub value: u8,
    pub offset: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceSkillBoost {
    pub actor_value: i8,
    pub boost: i8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MagicEffectSound {
    pub sound_type: u32,
    pub sound: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelTextureHash {
    pub file_hash: String,
    pub extension: String,
    pub folder_hash: String,
}

/// Editable payloads whose binary representation is unambiguous in TES5.
///
/// xEdit defines fields per record, not merely per subrecord signature.  This
/// enum is intentionally small: adding a guessed decoder is worse than leaving
/// an unknown field in its lossless blob.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubrecordValue {
    /// Authoritative bytes for a payload whose semantic schema is not yet
    /// implemented. This keeps clean JSON self-contained without a blob table.
    RawBytes {
        base64: String,
    },
    Zstring {
        text: String,
    },
    FixedString {
        text: String,
    },
    LocalizedStringId {
        id: String,
    },
    FormId {
        id: String,
    },
    FormIdArray {
        ids: Vec<String>,
    },
    InventoryItem {
        item: String,
        count: i32,
    },
    FactionMembership {
        faction: String,
        rank: i8,
        unused: String,
    },
    BipedBodyTemplate {
        slots: Vec<String>,
        slots_unknown_bits: String,
        armor_type: u32,
    },
    AttackData {
        damage_multiplier: f32,
        attack_chance: f32,
        attack_spell: String,
        flags: Vec<String>,
        flags_unknown_bits: String,
        attack_angle: f32,
        strike_angle: f32,
        stagger: f32,
        attack_type: String,
        knockdown: f32,
        recovery_time: f32,
        stamina_multiplier: f32,
    },
    U8 {
        value: u8,
    },
    I8 {
        value: i8,
    },
    U16 {
        value: u16,
    },
    I16 {
        value: i16,
    },
    Empty,
    Flags8 {
        set: Vec<String>,
        unknown_bits: String,
    },
    U32 {
        value: u32,
    },
    U64 {
        value: u64,
    },
    I32 {
        value: i32,
    },
    F32 {
        value: f32,
    },
    ColorRgba {
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    },
    ColorRgbFloat {
        red: f32,
        green: f32,
        blue: f32,
    },
    Flags32 {
        set: Vec<String>,
        unknown_bits: String,
    },
    FactionRelation {
        faction: String,
        modifier: i32,
        group_combat_reaction: u32,
    },
    CrimeValues {
        arrest: bool,
        attack_on_sight: bool,
        murder: u16,
        assault: u16,
        trespass: u16,
        pickpocket: u16,
        unknown: u16,
        steal_multiplier: f32,
        escape: u16,
        werewolf: u16,
    },
    VendorValues {
        start_hour: u16,
        end_hour: u16,
        radius: u16,
        unknown_1: String,
        only_buys_stolen_items: bool,
        not_sell_buy: bool,
        unknown_2: String,
    },
    Location {
        location_type: i32,
        location_value: String,
        radius: i32,
    },
    ObjectBounds {
        min_x: i16,
        min_y: i16,
        min_z: i16,
        max_x: i16,
        max_y: i16,
        max_z: i16,
    },
    PluginHeader {
        version: f32,
        number_of_records: u32,
        next_object_id: String,
    },
    ItemData {
        value: i32,
        weight: f32,
    },
    ArmorRating {
        value: f32,
    },
    NpcAiData {
        aggression: u8,
        confidence: u8,
        energy_level: u8,
        morality: u8,
        mood: u8,
        assistance: u8,
        aggro_radius_behavior: bool,
        unused: u8,
        warn: u32,
        warn_attack: u32,
        attack: u32,
    },
    NpcPlayerSkills {
        skills: Vec<NpcSkill>,
        health: u16,
        magicka: u16,
        stamina: u16,
        unused: String,
        far_away_model_distance: f32,
        geared_up_weapons: u8,
        trailing_unused: String,
    },
    LeveledListEntry {
        level: u16,
        reference: String,
        count: u16,
    },
    LeveledExtraData {
        owner: String,
        global_or_required_rank_raw: String,
        item_condition: f32,
    },
    SpellData {
        base_cost: u32,
        flags: Vec<String>,
        flags_unknown_bits: String,
        spell_type: u32,
        charge_time: f32,
        cast_type: u32,
        delivery: u32,
        cast_duration: f32,
        range: f32,
        half_cost_perk: String,
    },
    EffectParameters {
        magnitude: f32,
        area: u32,
        duration: u32,
    },
    MagicEffectData {
        flags: Vec<String>,
        flags_unknown_bits: String,
        base_cost: f32,
        associated_item: String,
        magic_skill: i32,
        resist_value: i32,
        counter_effect_count: u16,
        casting_light: String,
        taper_weight: f32,
        hit_shader: String,
        enchant_shader: String,
        minimum_skill_level: u32,
        spellmaking_area: u32,
        spellmaking_casting_time: f32,
        taper_curve: f32,
        taper_duration: f32,
        second_actor_value_weight: f32,
        archetype: u32,
        primary_actor_value: i32,
        projectile: String,
        explosion: String,
        casting_type: u32,
        delivery: u32,
        second_actor_value: i32,
        casting_art: String,
        hit_effect_art: String,
        impact_data: String,
        skill_usage_multiplier: f32,
        dual_casting_art: String,
        dual_casting_scale: f32,
        enchant_art: String,
        hit_visuals: String,
        enchant_visuals: String,
        equip_ability: String,
        image_space_modifier: String,
        perk_to_apply: String,
        casting_sound_level: u32,
        script_effect_ai_score: f32,
        script_effect_ai_delay_time: f32,
    },
    MagicEffectSounds {
        sounds: Vec<MagicEffectSound>,
    },
    FloatArray {
        values: Vec<f32>,
    },
    RaceData {
        skill_boosts: Vec<RaceSkillBoost>,
        unknown: String,
        male_height: f32,
        female_height: f32,
        male_weight: f32,
        female_weight: f32,
        flags: Vec<String>,
        flags_unknown_bits: String,
        starting_health: f32,
        starting_magicka: f32,
        starting_stamina: f32,
        base_carry_weight: f32,
        base_mass: f32,
        acceleration_rate: f32,
        deceleration_rate: f32,
        size: u32,
        head_biped_object: i32,
        hair_biped_object: i32,
        injured_health_pct: f32,
        shield_biped_object: i32,
        health_regen: f32,
        magicka_regen: f32,
        stamina_regen: f32,
        unarmed_damage: f32,
        unarmed_reach: f32,
        body_biped_object: i32,
        aim_angle_tolerance: f32,
        flight_radius: f32,
        angular_acceleration_rate: f32,
        angular_tolerance: f32,
        flags_2: Vec<String>,
        flags_2_unknown_bits: String,
        mount_offset_x: f32,
        mount_offset_y: f32,
        mount_offset_z: f32,
        dismount_offset_x: f32,
        dismount_offset_y: f32,
        dismount_offset_z: f32,
        mount_camera_offset_x: f32,
        mount_camera_offset_y: f32,
        mount_camera_offset_z: f32,
    },
    NpcConfiguration {
        flags: Vec<String>,
        flags_unknown_bits: String,
        magicka_offset: i16,
        stamina_offset: i16,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        level: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        level_multiplier: Option<f32>,
        calc_min_level: u16,
        calc_max_level: u16,
        speed_multiplier: u16,
        disposition_base_unused: i16,
        template_flags: Vec<String>,
        template_flags_unknown_bits: String,
        health_offset: i16,
        bleedout_override: u16,
    },
    RelationshipData {
        parent: String,
        child: String,
        rank: u16,
        unknown: u8,
        flags: u8,
        association_type: String,
    },
    SoundDescriptorValues {
        frequency_shift_percent: i8,
        frequency_variance_percent: i8,
        priority: u8,
        db_variance: u8,
        static_attenuation_db: f32,
    },
    SoundLoopValues {
        unknown_1: u8,
        looping: u8,
        unknown_2: u8,
        rumble_send_value: u8,
    },
    ArmorAddonData {
        male_priority: u8,
        female_priority: u8,
        male_weight_slider: u8,
        female_weight_slider: u8,
        unknown_1: String,
        detection_sound_value: u8,
        unknown_2: u8,
        weapon_adjust: f32,
    },
    IngestibleEffectData {
        value: i32,
        flags: u32,
        addiction: String,
        addiction_chance: f32,
        consume_sound: String,
    },
    DialogueData {
        do_all_before_repeating: bool,
        category: u8,
        subtype: u16,
    },
    PackageData {
        general_flags: u32,
        package_type: u8,
        interrupt_override: u8,
        preferred_speed: u8,
        unknown_1: u8,
        interrupt_flags: u16,
        unknown_2: String,
    },
    PackageSchedule {
        month: i8,
        day_of_week: i8,
        date: i8,
        hour: i8,
        minute: i8,
        unused: String,
        duration_minutes: u32,
    },
    PackageCounter {
        data_input_count: u32,
        package_template: String,
        version_counter: u32,
    },
    PackageTopicData {
        topic_type: u32,
        data: String,
    },
    ModelInformation {
        textures: Vec<ModelTextureHash>,
        addon_nodes: Vec<u32>,
    },
}

const NPC_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "female"),
    (1, "essential"),
    (2, "is_chargen_face_preset"),
    (3, "respawn"),
    (4, "auto_calc_stats"),
    (5, "unique"),
    (6, "does_not_affect_stealth_meter"),
    (7, "pc_level_mult"),
    (8, "use_template"),
    (9, "unknown_9"),
    (10, "unknown_10"),
    (11, "protected"),
    (12, "unknown_12"),
    (13, "unknown_13"),
    (14, "summonable"),
    (15, "unknown_15"),
    (16, "does_not_bleed"),
    (17, "unknown_17"),
    (18, "bleedout_override"),
    (19, "opposite_gender_anims"),
    (20, "simple_actor"),
    (21, "looped_script"),
    (22, "unknown_22"),
    (23, "unknown_23"),
    (24, "unknown_24"),
    (25, "unknown_25"),
    (26, "unknown_26"),
    (27, "unknown_27"),
    (28, "looped_audio"),
    (29, "is_ghost"),
    (30, "unknown_30"),
    (31, "invulnerable"),
];

const NPC_TEMPLATE_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "traits"),
    (1, "stats"),
    (2, "factions"),
    (3, "spell_list"),
    (4, "ai_data"),
    (5, "ai_packages"),
    (6, "model_animation"),
    (7, "base_data"),
    (8, "inventory"),
    (9, "script"),
    (10, "def_pack_list"),
    (11, "attack_data"),
    (12, "keywords"),
];

const NPC_SKILL_NAMES: [&str; 18] = [
    "one_handed",
    "two_handed",
    "marksman",
    "block",
    "smithing",
    "heavy_armor",
    "light_armor",
    "pickpocket",
    "lockpicking",
    "sneak",
    "alchemy",
    "speechcraft",
    "alteration",
    "conjuration",
    "destruction",
    "illusion",
    "restoration",
    "enchanting",
];

const FACT_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "hidden_from_npc"),
    (1, "special_combat"),
    (6, "track_crime"),
    (7, "ignore_murder"),
    (8, "ignore_assault"),
    (9, "ignore_stealing"),
    (10, "ignore_trespass"),
    (11, "do_not_report_crimes_against_members"),
    (12, "crime_gold_use_defaults"),
    (13, "ignore_pickpocket"),
    (14, "vendor"),
    (15, "can_be_owner"),
    (16, "ignore_werewolf"),
];

const SPELL_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "manual_cost_calc"),
    (16, "unknown_16"),
    (17, "pc_start_spell"),
    (18, "unknown_18"),
    (19, "area_effect_ignores_los"),
    (20, "ignore_resistance"),
    (21, "disallow_absorb_reflect"),
    (22, "unknown_22"),
    (23, "no_dual_cast_modification"),
];

const MGEF_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "hostile"),
    (1, "recover"),
    (2, "detrimental"),
    (3, "snap_to_navmesh"),
    (4, "no_hit_event"),
    (5, "unknown_6"),
    (6, "unknown_7"),
    (7, "unknown_8"),
    (8, "dispel_with_keywords"),
    (9, "no_duration"),
    (10, "no_magnitude"),
    (11, "no_area"),
    (12, "fx_persist"),
    (13, "unknown_14"),
    (14, "gory_visuals"),
    (15, "hide_in_ui"),
    (16, "unknown_17"),
    (17, "no_recast"),
    (18, "unknown_19"),
    (19, "unknown_20"),
    (20, "unknown_21"),
    (21, "power_affects_magnitude"),
    (22, "power_affects_duration"),
    (23, "unknown_24"),
    (24, "unknown_25"),
    (25, "unknown_26"),
    (26, "painless"),
    (27, "no_hit_effect"),
    (28, "no_death_dispel"),
    (29, "unknown_30"),
    (30, "unknown_31"),
    (31, "unknown_32"),
];

const RACE_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "playable"),
    (1, "facegen_head"),
    (2, "child"),
    (3, "tilt_front_back"),
    (4, "tilt_left_right"),
    (5, "no_shadow"),
    (6, "swims"),
    (7, "flies"),
    (8, "walks"),
    (9, "immobile"),
    (10, "not_pushable"),
    (11, "no_combat_in_water"),
    (12, "no_rotating_to_head_track"),
    (13, "do_not_show_blood_spray"),
    (14, "do_not_show_blood_decal"),
    (15, "uses_head_track_anims"),
    (16, "spells_align_with_magic_node"),
    (17, "use_world_raycasts_for_foot_ik"),
    (18, "allow_ragdoll_collision"),
    (19, "regen_hp_in_combat"),
    (20, "cannot_open_doors"),
    (21, "allow_pc_dialogue"),
    (22, "no_knockdowns"),
    (23, "allow_pickpocket"),
    (24, "always_use_proxy_controller"),
    (25, "do_not_show_weapon_blood"),
    (26, "overlay_head_part_list"),
    (27, "override_head_part_list"),
    (28, "can_pickup_items"),
    (29, "allow_multiple_membrane_shaders"),
    (30, "can_dual_wield"),
    (31, "avoids_roads"),
];
const RACE_FLAG_2_NAMES: &[(u32, &str)] = &[
    (0, "use_advanced_avoidance"),
    (1, "non_hostile"),
    (2, "unknown_2"),
    (3, "unknown_3"),
    (4, "allow_mounted_combat"),
];
const BIPED_SLOT_NAMES: &[(u32, &str)] = &[
    (0, "30_head"),
    (1, "31_hair"),
    (2, "32_body"),
    (3, "33_hands"),
    (4, "34_forearms"),
    (5, "35_amulet"),
    (6, "36_ring"),
    (7, "37_feet"),
    (8, "38_calves"),
    (9, "39_shield"),
    (10, "40_tail"),
    (11, "41_long_hair"),
    (12, "42_circlet"),
    (13, "43_ears"),
    (20, "50_decapitate_head"),
    (21, "51_decapitate"),
    (31, "61_fx01"),
];
const ATTACK_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "ignore_weapon"),
    (1, "bash_attack"),
    (2, "power_attack"),
    (3, "left_attack"),
    (4, "rotating_attack"),
    (31, "override_data"),
];
const LVLN_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "calculate_from_all_levels_at_or_below_player"),
    (1, "calculate_for_each_item_in_count"),
];
const LVLI_FLAG_NAMES: &[(u32, &str)] = &[
    (0, "calculate_from_all_levels_at_or_below_player"),
    (1, "calculate_for_each_item_in_count"),
    (2, "use_all"),
    (3, "special_loot"),
];

pub fn display_name(record: &str, signature: &str) -> Option<&'static str> {
    display_names::get(record, signature)
}

#[cfg(test)]
pub fn decode(record: &str, signature: &str, bytes: &[u8]) -> Option<SubrecordValue> {
    decode_with_localization(record, signature, bytes, false)
}

pub fn decode_with_localization(
    record: &str,
    signature: &str,
    bytes: &[u8],
    localized: bool,
) -> Option<SubrecordValue> {
    // wbEDID is wbStringKC(EDID, ...) and is shared by TES5 record definitions.
    if signature == "EDID" {
        return decode_zstring(bytes);
    }
    if is_zstring_field(record, signature) {
        return decode_zstring(bytes);
    }
    if (record, signature, bytes.len()) == ("RACE", "MTNM", 4) {
        return Some(SubrecordValue::FixedString {
            text: bytes.iter().copied().map(char::from).collect(),
        });
    }
    if is_localized_string_field(record, signature) {
        return if localized {
            (bytes.len() == 4).then(|| SubrecordValue::LocalizedStringId {
                id: hex_u32(read_u32(bytes, 0)),
            })
        } else {
            decode_zstring(bytes)
        };
    }
    if bytes.is_empty() && is_empty_field(record, signature) {
        return Some(SubrecordValue::Empty);
    }
    if (record, signature, bytes.len()) == ("FACT", "CRVA", 20)
        && !f32::from_le_bytes(bytes[12..16].try_into().ok()?).is_finite()
    {
        return None;
    }

    // These pairs are direct primitive definitions in wbDefinitionsTES5.pas.
    // Keeping record context in the table prevents collisions such as DATA.
    match (record, signature, bytes.len()) {
        ("TES4", "HEDR", 12) => Some(SubrecordValue::PluginHeader {
            version: finite_f32_at(bytes, 0)?,
            number_of_records: read_u32(bytes, 4),
            next_object_id: hex_u32(read_u32(bytes, 8)),
        }),
        ("TES4", "DATA", 8) => Some(SubrecordValue::U64 {
            value: u64::from_le_bytes(bytes.try_into().ok()?),
        }),
        (_, "OBND", 12) => Some(SubrecordValue::ObjectBounds {
            min_x: read_i16(bytes, 0),
            min_y: read_i16(bytes, 2),
            min_z: read_i16(bytes, 4),
            max_x: read_i16(bytes, 6),
            max_y: read_i16(bytes, 8),
            max_z: read_i16(bytes, 10),
        }),
        ("ARMO" | "ARMA" | "RACE", "BOD2", 8) => {
            let (slots, slots_unknown_bits) =
                decode_named_flags(read_u32(bytes, 0), BIPED_SLOT_NAMES);
            Some(SubrecordValue::BipedBodyTemplate {
                slots,
                slots_unknown_bits,
                armor_type: read_u32(bytes, 4),
            })
        }
        ("NPC_" | "RACE", "ATKD", 44) => {
            let float_offsets = [0, 4, 16, 20, 24, 32, 36, 40];
            if !float_offsets
                .iter()
                .all(|offset| finite_f32_at(bytes, *offset).is_some())
            {
                return None;
            }
            let (flags, flags_unknown_bits) =
                decode_named_flags(read_u32(bytes, 12), ATTACK_FLAG_NAMES);
            Some(SubrecordValue::AttackData {
                damage_multiplier: finite_f32_at(bytes, 0)?,
                attack_chance: finite_f32_at(bytes, 4)?,
                attack_spell: hex_u32(read_u32(bytes, 8)),
                flags,
                flags_unknown_bits,
                attack_angle: finite_f32_at(bytes, 16)?,
                strike_angle: finite_f32_at(bytes, 20)?,
                stagger: finite_f32_at(bytes, 24)?,
                attack_type: hex_u32(read_u32(bytes, 28)),
                knockdown: finite_f32_at(bytes, 32)?,
                recovery_time: finite_f32_at(bytes, 36)?,
                stamina_multiplier: finite_f32_at(bytes, 40)?,
            })
        }
        (record, signature, len) if len % 4 == 0 && is_form_id_array_field(record, signature) => {
            Some(SubrecordValue::FormIdArray {
                ids: bytes.chunks_exact(4).map(form_id_text).collect(),
            })
        }
        ("NPC_" | "COBJ", "CNTO", 8) => Some(SubrecordValue::InventoryItem {
            item: hex_u32(read_u32(bytes, 0)),
            count: read_i32(bytes, 4),
        }),
        ("NPC_", "SNAM", 8) => Some(SubrecordValue::FactionMembership {
            faction: hex_u32(read_u32(bytes, 0)),
            rank: bytes[4] as i8,
            unused: hex_bytes(&bytes[5..8]),
        }),
        ("ARMO", "DATA", 8) => Some(SubrecordValue::ItemData {
            value: read_i32(bytes, 0),
            weight: finite_f32_at(bytes, 4)?,
        }),
        ("ARMO", "DNAM", 4) => Some(SubrecordValue::ArmorRating {
            value: read_i32(bytes, 0) as f32 / 100.0,
        }),
        ("NPC_", "AIDT", 20) => Some(SubrecordValue::NpcAiData {
            aggression: bytes[0],
            confidence: bytes[1],
            energy_level: bytes[2],
            morality: bytes[3],
            mood: bytes[4],
            assistance: bytes[5],
            aggro_radius_behavior: bytes[6] != 0,
            unused: bytes[7],
            warn: read_u32(bytes, 8),
            warn_attack: read_u32(bytes, 12),
            attack: read_u32(bytes, 16),
        }),
        ("NPC_", "DNAM", 52) => {
            let skills = NPC_SKILL_NAMES
                .iter()
                .enumerate()
                .map(|(index, name)| NpcSkill {
                    name: (*name).to_owned(),
                    value: bytes[index],
                    offset: bytes[18 + index],
                })
                .collect();
            Some(SubrecordValue::NpcPlayerSkills {
                skills,
                health: read_u16(bytes, 36),
                magicka: read_u16(bytes, 38),
                stamina: read_u16(bytes, 40),
                unused: hex_bytes(&bytes[42..44]),
                far_away_model_distance: finite_f32_at(bytes, 44)?,
                geared_up_weapons: bytes[48],
                trailing_unused: hex_bytes(&bytes[49..52]),
            })
        }
        ("NPC_", "QNAM", 12) => Some(SubrecordValue::ColorRgbFloat {
            red: finite_f32_at(bytes, 0)?,
            green: finite_f32_at(bytes, 4)?,
            blue: finite_f32_at(bytes, 8)?,
        }),
        ("NPC_", "TINC", 4) => Some(SubrecordValue::ColorRgba {
            red: bytes[0],
            green: bytes[1],
            blue: bytes[2],
            alpha: bytes[3],
        }),
        (record, signature, 2) if is_u16_field(record, signature) => Some(SubrecordValue::U16 {
            value: read_u16(bytes, 0),
        }),
        (record, signature, 2) if is_i16_field(record, signature) => Some(SubrecordValue::I16 {
            value: read_i16(bytes, 0),
        }),
        ("LVLN" | "LVLI", "LVLD" | "LLCT", 1) => Some(SubrecordValue::U8 { value: bytes[0] }),
        ("LVLN", "LVLF", 1) => Some(decode_flags8(bytes[0], LVLN_FLAG_NAMES)),
        ("LVLI", "LVLF", 1) => Some(decode_flags8(bytes[0], LVLI_FLAG_NAMES)),
        ("LVLN" | "LVLI", "LVLO", 12) => Some(SubrecordValue::LeveledListEntry {
            level: read_u16(bytes, 0),
            reference: hex_u32(read_u32(bytes, 4)),
            count: read_u16(bytes, 8),
        }),
        ("LVLN" | "LVLI", "COED", 12) => Some(SubrecordValue::LeveledExtraData {
            owner: hex_u32(read_u32(bytes, 0)),
            global_or_required_rank_raw: hex_u32(read_u32(bytes, 4)),
            item_condition: finite_f32_at(bytes, 8)?,
        }),
        ("SPEL", "SPIT", 36) => {
            let (flags, flags_unknown_bits) =
                decode_named_flags(read_u32(bytes, 4), SPELL_FLAG_NAMES);
            Some(SubrecordValue::SpellData {
                base_cost: read_u32(bytes, 0),
                flags,
                flags_unknown_bits,
                spell_type: read_u32(bytes, 8),
                charge_time: finite_f32_at(bytes, 12)?,
                cast_type: read_u32(bytes, 16),
                delivery: read_u32(bytes, 20),
                cast_duration: finite_f32_at(bytes, 24)?,
                range: finite_f32_at(bytes, 28)?,
                half_cost_perk: hex_u32(read_u32(bytes, 32)),
            })
        }
        ("SPEL" | "ALCH", "EFIT", 12) => Some(SubrecordValue::EffectParameters {
            magnitude: finite_f32_at(bytes, 0)?,
            area: read_u32(bytes, 4),
            duration: read_u32(bytes, 8),
        }),
        ("MGEF", "DATA", 152) => {
            let (flags, flags_unknown_bits) =
                decode_named_flags(read_u32(bytes, 0), MGEF_FLAG_NAMES);
            let float_offsets = [4, 28, 48, 52, 56, 60, 104, 112, 144, 148];
            if !float_offsets
                .iter()
                .all(|offset| finite_f32_at(bytes, *offset).is_some())
            {
                return None;
            }
            Some(SubrecordValue::MagicEffectData {
                flags,
                flags_unknown_bits,
                base_cost: finite_f32_at(bytes, 4)?,
                associated_item: hex_u32(read_u32(bytes, 8)),
                magic_skill: read_i32(bytes, 12),
                resist_value: read_i32(bytes, 16),
                counter_effect_count: read_u16(bytes, 20),
                casting_light: hex_u32(read_u32(bytes, 24)),
                taper_weight: finite_f32_at(bytes, 28)?,
                hit_shader: hex_u32(read_u32(bytes, 32)),
                enchant_shader: hex_u32(read_u32(bytes, 36)),
                minimum_skill_level: read_u32(bytes, 40),
                spellmaking_area: read_u32(bytes, 44),
                spellmaking_casting_time: finite_f32_at(bytes, 48)?,
                taper_curve: finite_f32_at(bytes, 52)?,
                taper_duration: finite_f32_at(bytes, 56)?,
                second_actor_value_weight: finite_f32_at(bytes, 60)?,
                archetype: read_u32(bytes, 64),
                primary_actor_value: read_i32(bytes, 68),
                projectile: hex_u32(read_u32(bytes, 72)),
                explosion: hex_u32(read_u32(bytes, 76)),
                casting_type: read_u32(bytes, 80),
                delivery: read_u32(bytes, 84),
                second_actor_value: read_i32(bytes, 88),
                casting_art: hex_u32(read_u32(bytes, 92)),
                hit_effect_art: hex_u32(read_u32(bytes, 96)),
                impact_data: hex_u32(read_u32(bytes, 100)),
                skill_usage_multiplier: finite_f32_at(bytes, 104)?,
                dual_casting_art: hex_u32(read_u32(bytes, 108)),
                dual_casting_scale: finite_f32_at(bytes, 112)?,
                enchant_art: hex_u32(read_u32(bytes, 116)),
                hit_visuals: hex_u32(read_u32(bytes, 120)),
                enchant_visuals: hex_u32(read_u32(bytes, 124)),
                equip_ability: hex_u32(read_u32(bytes, 128)),
                image_space_modifier: hex_u32(read_u32(bytes, 132)),
                perk_to_apply: hex_u32(read_u32(bytes, 136)),
                casting_sound_level: read_u32(bytes, 140),
                script_effect_ai_score: finite_f32_at(bytes, 144)?,
                script_effect_ai_delay_time: finite_f32_at(bytes, 148)?,
            })
        }
        ("MGEF", "SNDD", len) if len % 8 == 0 => Some(SubrecordValue::MagicEffectSounds {
            sounds: bytes
                .chunks_exact(8)
                .map(|entry| MagicEffectSound {
                    sound_type: read_u32(entry, 0),
                    sound: hex_u32(read_u32(entry, 4)),
                })
                .collect(),
        }),
        ("RELA", "DATA", 16) => Some(SubrecordValue::RelationshipData {
            parent: hex_u32(read_u32(bytes, 0)),
            child: hex_u32(read_u32(bytes, 4)),
            rank: read_u16(bytes, 8),
            unknown: bytes[10],
            flags: bytes[11],
            association_type: hex_u32(read_u32(bytes, 12)),
        }),
        ("SNDR", "BNAM", 6) => Some(SubrecordValue::SoundDescriptorValues {
            frequency_shift_percent: bytes[0] as i8,
            frequency_variance_percent: bytes[1] as i8,
            priority: bytes[2],
            db_variance: bytes[3],
            static_attenuation_db: read_u16(bytes, 4) as f32 / 100.0,
        }),
        ("SNDR", "LNAM", 4) => Some(SubrecordValue::SoundLoopValues {
            unknown_1: bytes[0],
            looping: bytes[1],
            unknown_2: bytes[2],
            rumble_send_value: bytes[3],
        }),
        ("ARMA", "DNAM", 12) => Some(SubrecordValue::ArmorAddonData {
            male_priority: bytes[0],
            female_priority: bytes[1],
            male_weight_slider: bytes[2],
            female_weight_slider: bytes[3],
            unknown_1: hex_bytes(&bytes[4..6]),
            detection_sound_value: bytes[6],
            unknown_2: bytes[7],
            weapon_adjust: finite_f32_at(bytes, 8)?,
        }),
        ("ALCH", "ENIT", 20) => Some(SubrecordValue::IngestibleEffectData {
            value: read_i32(bytes, 0),
            flags: read_u32(bytes, 4),
            addiction: hex_u32(read_u32(bytes, 8)),
            addiction_chance: finite_f32_at(bytes, 12)?,
            consume_sound: hex_u32(read_u32(bytes, 16)),
        }),
        ("DIAL", "DATA", 4) => Some(SubrecordValue::DialogueData {
            do_all_before_repeating: bytes[0] != 0,
            category: bytes[1],
            subtype: read_u16(bytes, 2),
        }),
        ("PACK", "PKDT", 12) => Some(SubrecordValue::PackageData {
            general_flags: read_u32(bytes, 0),
            package_type: bytes[4],
            interrupt_override: bytes[5],
            preferred_speed: bytes[6],
            unknown_1: bytes[7],
            interrupt_flags: read_u16(bytes, 8),
            unknown_2: hex_bytes(&bytes[10..12]),
        }),
        ("PACK", "PSDT", 12) => Some(SubrecordValue::PackageSchedule {
            month: bytes[0] as i8,
            day_of_week: bytes[1] as i8,
            date: bytes[2] as i8,
            hour: bytes[3] as i8,
            minute: bytes[4] as i8,
            unused: hex_bytes(&bytes[5..8]),
            duration_minutes: read_u32(bytes, 8),
        }),
        ("PACK", "PKCU", 12) => Some(SubrecordValue::PackageCounter {
            data_input_count: read_u32(bytes, 0),
            package_template: hex_u32(read_u32(bytes, 4)),
            version_counter: read_u32(bytes, 8),
        }),
        ("PACK", "PDTO", 8) => Some(SubrecordValue::PackageTopicData {
            topic_type: read_u32(bytes, 0),
            data: hex_u32(read_u32(bytes, 4)),
        }),
        ("PACK", "PLDT", 12) => Some(SubrecordValue::Location {
            location_type: read_i32(bytes, 0),
            location_value: hex_u32(read_u32(bytes, 4)),
            radius: read_i32(bytes, 8),
        }),
        ("PACK", "UNAM", 1) => Some(SubrecordValue::I8 {
            value: bytes[0] as i8,
        }),
        ("PACK", "CNAM" | "XNAM", 1) => Some(SubrecordValue::U8 { value: bytes[0] }),
        ("ARMA", "MO2T" | "MO3T" | "MO4T" | "MO5T", len) | ("ALCH", "MODT", len)
            if len >= 12 && read_u32(bytes, 0) == 2 =>
        {
            let texture_count = read_u32(bytes, 4) as usize;
            let addon_count = read_u32(bytes, 8) as usize;
            let expected = 12usize
                .checked_add(texture_count.checked_mul(12)?)?
                .checked_add(addon_count.checked_mul(4)?)?;
            if expected != len {
                return None;
            }
            let texture_end = 12 + texture_count * 12;
            let textures = bytes[12..texture_end]
                .chunks_exact(12)
                .map(|entry| ModelTextureHash {
                    file_hash: hex_u32(read_u32(entry, 0)),
                    extension: entry[4..8]
                        .iter()
                        .copied()
                        .take_while(|byte| *byte != 0)
                        .map(char::from)
                        .collect(),
                    folder_hash: hex_u32(read_u32(entry, 8)),
                })
                .collect();
            let addon_nodes = bytes[texture_end..]
                .chunks_exact(4)
                .map(|entry| read_u32(entry, 0))
                .collect();
            Some(SubrecordValue::ModelInformation {
                textures,
                addon_nodes,
            })
        }
        ("RACE", "PHWT", 64)
            if bytes
                .chunks_exact(4)
                .all(|v| f32::from_le_bytes(v.try_into().unwrap()).is_finite()) =>
        {
            Some(SubrecordValue::FloatArray {
                values: bytes
                    .chunks_exact(4)
                    .map(|v| f32::from_le_bytes(v.try_into().unwrap()))
                    .collect(),
            })
        }
        ("RACE", "DATA", 164) => {
            let float_offsets = [
                16, 20, 24, 28, 36, 40, 44, 48, 52, 56, 60, 76, 84, 88, 92, 96, 100, 108, 112, 116,
                120, 128, 132, 136, 140, 144, 148, 152, 156, 160,
            ];
            if !float_offsets
                .iter()
                .all(|offset| finite_f32_at(bytes, *offset).is_some())
            {
                return None;
            }
            let (flags, flags_unknown_bits) =
                decode_named_flags(read_u32(bytes, 32), RACE_FLAG_NAMES);
            let (flags_2, flags_2_unknown_bits) =
                decode_named_flags(read_u32(bytes, 124), RACE_FLAG_2_NAMES);
            Some(SubrecordValue::RaceData {
                skill_boosts: bytes[..14]
                    .chunks_exact(2)
                    .map(|v| RaceSkillBoost {
                        actor_value: v[0] as i8,
                        boost: v[1] as i8,
                    })
                    .collect(),
                unknown: hex_bytes(&bytes[14..16]),
                male_height: finite_f32_at(bytes, 16)?,
                female_height: finite_f32_at(bytes, 20)?,
                male_weight: finite_f32_at(bytes, 24)?,
                female_weight: finite_f32_at(bytes, 28)?,
                flags,
                flags_unknown_bits,
                starting_health: finite_f32_at(bytes, 36)?,
                starting_magicka: finite_f32_at(bytes, 40)?,
                starting_stamina: finite_f32_at(bytes, 44)?,
                base_carry_weight: finite_f32_at(bytes, 48)?,
                base_mass: finite_f32_at(bytes, 52)?,
                acceleration_rate: finite_f32_at(bytes, 56)?,
                deceleration_rate: finite_f32_at(bytes, 60)?,
                size: read_u32(bytes, 64),
                head_biped_object: read_i32(bytes, 68),
                hair_biped_object: read_i32(bytes, 72),
                injured_health_pct: finite_f32_at(bytes, 76)?,
                shield_biped_object: read_i32(bytes, 80),
                health_regen: finite_f32_at(bytes, 84)?,
                magicka_regen: finite_f32_at(bytes, 88)?,
                stamina_regen: finite_f32_at(bytes, 92)?,
                unarmed_damage: finite_f32_at(bytes, 96)?,
                unarmed_reach: finite_f32_at(bytes, 100)?,
                body_biped_object: read_i32(bytes, 104),
                aim_angle_tolerance: finite_f32_at(bytes, 108)?,
                flight_radius: finite_f32_at(bytes, 112)?,
                angular_acceleration_rate: finite_f32_at(bytes, 116)?,
                angular_tolerance: finite_f32_at(bytes, 120)?,
                flags_2,
                flags_2_unknown_bits,
                mount_offset_x: finite_f32_at(bytes, 128)?,
                mount_offset_y: finite_f32_at(bytes, 132)?,
                mount_offset_z: finite_f32_at(bytes, 136)?,
                dismount_offset_x: finite_f32_at(bytes, 140)?,
                dismount_offset_y: finite_f32_at(bytes, 144)?,
                dismount_offset_z: finite_f32_at(bytes, 148)?,
                mount_camera_offset_x: finite_f32_at(bytes, 152)?,
                mount_camera_offset_y: finite_f32_at(bytes, 156)?,
                mount_camera_offset_z: finite_f32_at(bytes, 160)?,
            })
        }
        ("GLOB", "FLTV", 4) => finite_f32(bytes),
        ("GLOB", "FNAM", 1) => Some(SubrecordValue::U8 { value: bytes[0] }),
        ("FLST", "LNAM", 4) => Some(form_id(bytes)),
        ("KYWD" | "LCRT" | "AACT", "CNAM", 4) => Some(SubrecordValue::ColorRgba {
            red: bytes[0],
            green: bytes[1],
            blue: bytes[2],
            alpha: bytes[3],
        }),
        ("ADDN", "DATA", 4) => Some(SubrecordValue::U32 {
            value: u32::from_le_bytes(bytes.try_into().ok()?),
        }),
        ("FSTP", "DATA", 4) => Some(form_id(bytes)),
        ("ALCH", "DATA", 4) => finite_f32(bytes),
        ("FACT", "DATA", 4) => Some(decode_flags32(bytes, FACT_FLAG_NAMES)),
        ("FACT", "XNAM", 12) => Some(SubrecordValue::FactionRelation {
            faction: hex_u32(read_u32(bytes, 0)),
            modifier: read_i32(bytes, 4),
            group_combat_reaction: read_u32(bytes, 8),
        }),
        ("FACT", "CRVA", 20) => Some(SubrecordValue::CrimeValues {
            arrest: bytes[0] != 0,
            attack_on_sight: bytes[1] != 0,
            murder: read_u16(bytes, 2),
            assault: read_u16(bytes, 4),
            trespass: read_u16(bytes, 6),
            pickpocket: read_u16(bytes, 8),
            unknown: read_u16(bytes, 10),
            steal_multiplier: f32::from_le_bytes(bytes[12..16].try_into().unwrap()),
            escape: read_u16(bytes, 16),
            werewolf: read_u16(bytes, 18),
        }),
        ("FACT", "VENV", 12) => Some(SubrecordValue::VendorValues {
            start_hour: read_u16(bytes, 0),
            end_hour: read_u16(bytes, 2),
            radius: read_u16(bytes, 4),
            unknown_1: hex_bytes(&bytes[6..8]),
            only_buys_stolen_items: bytes[8] != 0,
            not_sell_buy: bytes[9] != 0,
            unknown_2: hex_bytes(&bytes[10..12]),
        }),
        ("FACT", "PLVD", 12) => Some(SubrecordValue::Location {
            location_type: read_i32(bytes, 0),
            location_value: hex_u32(read_u32(bytes, 4)),
            radius: read_i32(bytes, 8),
        }),
        ("NPC_", "ACBS", 24) => {
            let flag_bits = read_u32(bytes, 0);
            let (flags, flags_unknown_bits) = decode_named_flags(flag_bits, NPC_FLAG_NAMES);
            let template_bits = read_u16(bytes, 18) as u32;
            let (template_flags, template_flags_unknown_bits) =
                decode_named_flags(template_bits, NPC_TEMPLATE_FLAG_NAMES);
            let raw_level = read_u16(bytes, 8);
            Some(SubrecordValue::NpcConfiguration {
                flags,
                flags_unknown_bits,
                magicka_offset: read_i16(bytes, 4),
                stamina_offset: read_i16(bytes, 6),
                level: (flag_bits & 0x80 == 0).then_some(raw_level),
                level_multiplier: (flag_bits & 0x80 != 0).then_some(raw_level as f32 / 1000.0),
                calc_min_level: read_u16(bytes, 10),
                calc_max_level: read_u16(bytes, 12),
                speed_multiplier: read_u16(bytes, 14),
                disposition_base_unused: read_i16(bytes, 16),
                template_flags,
                template_flags_unknown_bits,
                health_offset: read_i16(bytes, 20),
                bleedout_override: read_u16(bytes, 22),
            })
        }
        ("FACT", "JAIL" | "WAIT" | "STOL" | "PLCN" | "CRGR" | "JOUT" | "VEND" | "VENC", 4) => {
            Some(form_id(bytes))
        }
        (record, signature, 4) if is_form_id_field(record, signature) => Some(form_id(bytes)),
        (record, signature, 4) if is_u32_field(record, signature) => Some(SubrecordValue::U32 {
            value: read_u32(bytes, 0),
        }),
        (record, signature, 4) if is_f32_field(record, signature) => finite_f32(bytes),
        _ => None,
    }
}

fn decode_zstring(bytes: &[u8]) -> Option<SubrecordValue> {
    let body = bytes.strip_suffix(&[0])?;
    // TES5 editor IDs are effectively byte strings. Map bytes one-to-one so
    // non-UTF-8 plugins remain round-trip safe without assuming a code page.
    Some(SubrecordValue::Zstring {
        text: body.iter().copied().map(char::from).collect(),
    })
}

fn form_id(bytes: &[u8]) -> SubrecordValue {
    SubrecordValue::FormId {
        id: form_id_text(bytes),
    }
}

fn form_id_text(bytes: &[u8]) -> String {
    format!("0x{:08X}", u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn finite_f32(bytes: &[u8]) -> Option<SubrecordValue> {
    let value = f32::from_le_bytes(bytes.try_into().ok()?);
    // JSON has no NaN or infinity. Preserve unusual bit patterns as blobs.
    value.is_finite().then_some(SubrecordValue::F32 { value })
}

fn finite_f32_at(bytes: &[u8], offset: usize) -> Option<f32> {
    let value = f32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
    value.is_finite().then_some(value)
}

pub fn encode(record: &str, signature: &str, value: &SubrecordValue) -> Result<Vec<u8>> {
    match value {
        SubrecordValue::RawBytes { base64 } => BASE64
            .decode(base64)
            .with_context(|| format!("invalid raw_bytes base64 in {record}.{signature}")),
        SubrecordValue::Zstring { text } => {
            ensure!(
                signature == "EDID"
                    || is_zstring_field(record, signature)
                    || is_localized_string_field(record, signature),
                "{record}.{signature} is not a supported zstring field"
            );
            let mut bytes = Vec::with_capacity(text.len() + 1);
            for c in text.chars() {
                let code = u32::from(c);
                ensure!(
                    code <= 0xff && code != 0,
                    "{record}.{signature} contains a character that cannot be represented as one plugin byte"
                );
                bytes.push(code as u8);
            }
            bytes.push(0);
            Ok(bytes)
        }
        SubrecordValue::FixedString { text } if (record, signature) == ("RACE", "MTNM") => {
            let bytes = plugin_bytes(text, record, signature)?;
            ensure!(
                bytes.len() == 4,
                "RACE.MTNM must contain exactly four bytes"
            );
            Ok(bytes)
        }
        SubrecordValue::LocalizedStringId { id }
            if is_localized_string_field(record, signature) =>
        {
            Ok(parse_hex_u32(id)?.to_le_bytes().to_vec())
        }
        SubrecordValue::Empty if is_empty_field(record, signature) => Ok(Vec::new()),
        SubrecordValue::FormId { id }
            if is_form_id_field(record, signature)
                || matches!((record, signature), ("FLST", "LNAM") | ("FSTP", "DATA")) =>
        {
            let value = parse_hex_u32(id)?;
            Ok(value.to_le_bytes().to_vec())
        }
        SubrecordValue::FormIdArray { ids } if is_form_id_array_field(record, signature) => {
            let mut out = Vec::with_capacity(ids.len() * 4);
            for id in ids {
                out.extend_from_slice(&parse_hex_u32(id)?.to_le_bytes());
            }
            Ok(out)
        }
        SubrecordValue::InventoryItem { item, count }
            if matches!(record, "NPC_" | "COBJ") && signature == "CNTO" =>
        {
            let mut out = parse_hex_u32(item)?.to_le_bytes().to_vec();
            out.extend_from_slice(&count.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::FactionMembership {
            faction,
            rank,
            unused,
        } if (record, signature) == ("NPC_", "SNAM") => {
            let mut out = parse_hex_u32(faction)?.to_le_bytes().to_vec();
            out.push(*rank as u8);
            out.extend_from_slice(&parse_hex_bytes(unused, 3)?);
            Ok(out)
        }
        SubrecordValue::BipedBodyTemplate {
            slots,
            slots_unknown_bits,
            armor_type,
        } if matches!(record, "ARMO" | "ARMA" | "RACE") && signature == "BOD2" => {
            let bits =
                encode_named_flags(slots, slots_unknown_bits, BIPED_SLOT_NAMES, "BOD2 slots")?;
            let mut out = bits.to_le_bytes().to_vec();
            out.extend_from_slice(&armor_type.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::AttackData {
            damage_multiplier,
            attack_chance,
            attack_spell,
            flags,
            flags_unknown_bits,
            attack_angle,
            strike_angle,
            stagger,
            attack_type,
            knockdown,
            recovery_time,
            stamina_multiplier,
        } if matches!(record, "NPC_" | "RACE") && signature == "ATKD" => {
            let floats = [
                damage_multiplier,
                attack_chance,
                attack_angle,
                strike_angle,
                stagger,
                knockdown,
                recovery_time,
                stamina_multiplier,
            ];
            ensure!(
                floats.iter().all(|v| v.is_finite()),
                "ATKD floats must be finite"
            );
            let bits =
                encode_named_flags(flags, flags_unknown_bits, ATTACK_FLAG_NAMES, "ATKD flags")?;
            let mut out = Vec::with_capacity(44);
            push_f32(&mut out, *damage_multiplier);
            push_f32(&mut out, *attack_chance);
            push_form(&mut out, attack_spell)?;
            push_u32(&mut out, bits);
            for value in [attack_angle, strike_angle, stagger] {
                push_f32(&mut out, *value);
            }
            push_form(&mut out, attack_type)?;
            for value in [knockdown, recovery_time, stamina_multiplier] {
                push_f32(&mut out, *value);
            }
            Ok(out)
        }
        SubrecordValue::U8 { value }
            if matches!((record, signature), ("LVLN" | "LVLI", "LVLD" | "LLCT")) =>
        {
            Ok(vec![*value])
        }
        SubrecordValue::U8 { value } if (record, signature) == ("GLOB", "FNAM") => Ok(vec![*value]),
        SubrecordValue::U8 { value }
            if matches!((record, signature), ("PACK", "CNAM" | "XNAM")) =>
        {
            Ok(vec![*value])
        }
        SubrecordValue::I8 { value } if (record, signature) == ("PACK", "UNAM") => {
            Ok(vec![*value as u8])
        }
        SubrecordValue::Flags8 { set, unknown_bits }
            if matches!((record, signature), ("LVLN" | "LVLI", "LVLF")) =>
        {
            let names = if record == "LVLN" {
                LVLN_FLAG_NAMES
            } else {
                LVLI_FLAG_NAMES
            };
            let bits =
                encode_named_flags(set, unknown_bits, names, &format!("{record}.LVLF flags"))?;
            ensure!(bits <= u8::MAX as u32, "{record}.LVLF exceeds eight bits");
            Ok(vec![bits as u8])
        }
        SubrecordValue::U16 { value } if is_u16_field(record, signature) => {
            Ok(value.to_le_bytes().to_vec())
        }
        SubrecordValue::I16 { value } if is_i16_field(record, signature) => {
            Ok(value.to_le_bytes().to_vec())
        }
        SubrecordValue::U32 { value } if (record, signature) == ("ADDN", "DATA") => {
            Ok(value.to_le_bytes().to_vec())
        }
        SubrecordValue::U32 { value } if is_u32_field(record, signature) => {
            Ok(value.to_le_bytes().to_vec())
        }
        SubrecordValue::U64 { value } if (record, signature) == ("TES4", "DATA") => {
            Ok(value.to_le_bytes().to_vec())
        }
        SubrecordValue::I32 { .. } => {
            bail!("{record}.{signature} has no supported signed-integer codec")
        }
        SubrecordValue::F32 { value }
            if matches!((record, signature), ("GLOB", "FLTV") | ("ALCH", "DATA"))
                || is_f32_field(record, signature) =>
        {
            Ok(value.to_le_bytes().to_vec())
        }
        SubrecordValue::ColorRgba {
            red,
            green,
            blue,
            alpha,
        } if matches!(
            (record, signature),
            ("KYWD" | "LCRT" | "AACT", "CNAM") | ("NPC_", "TINC")
        ) =>
        {
            Ok(vec![*red, *green, *blue, *alpha])
        }
        SubrecordValue::ColorRgbFloat { red, green, blue }
            if (record, signature) == ("NPC_", "QNAM") =>
        {
            ensure!(
                [red, green, blue].iter().all(|v| v.is_finite()),
                "NPC_.QNAM colors must be finite"
            );
            let mut out = Vec::with_capacity(12);
            for value in [red, green, blue] {
                push_f32(&mut out, *value);
            }
            Ok(out)
        }
        SubrecordValue::Flags32 { set, unknown_bits }
            if (record, signature) == ("FACT", "DATA") =>
        {
            let mut bits = parse_hex_u32(unknown_bits)?;
            let known_mask = FACT_FLAG_NAMES
                .iter()
                .fold(0u32, |mask, (bit, _)| mask | (1 << bit));
            ensure!(
                bits & known_mask == 0,
                "FACT.DATA unknown_bits overlaps named flags"
            );
            for name in set {
                let bit = FACT_FLAG_NAMES
                    .iter()
                    .find(|(_, candidate)| candidate == name)
                    .map(|(bit, _)| *bit)
                    .with_context(|| format!("unknown FACT.DATA flag {name:?}"))?;
                bits |= 1 << bit;
            }
            Ok(bits.to_le_bytes().to_vec())
        }
        SubrecordValue::FactionRelation {
            faction,
            modifier,
            group_combat_reaction,
        } if (record, signature) == ("FACT", "XNAM") => {
            let mut out = Vec::with_capacity(12);
            out.extend_from_slice(&parse_hex_u32(faction)?.to_le_bytes());
            out.extend_from_slice(&modifier.to_le_bytes());
            out.extend_from_slice(&group_combat_reaction.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::CrimeValues {
            arrest,
            attack_on_sight,
            murder,
            assault,
            trespass,
            pickpocket,
            unknown,
            steal_multiplier,
            escape,
            werewolf,
        } if (record, signature) == ("FACT", "CRVA") => {
            ensure!(
                steal_multiplier.is_finite(),
                "FACT.CRVA steal_multiplier must be finite"
            );
            let mut out = vec![u8::from(*arrest), u8::from(*attack_on_sight)];
            for value in [murder, assault, trespass, pickpocket, unknown] {
                out.extend_from_slice(&value.to_le_bytes());
            }
            out.extend_from_slice(&steal_multiplier.to_le_bytes());
            out.extend_from_slice(&escape.to_le_bytes());
            out.extend_from_slice(&werewolf.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::VendorValues {
            start_hour,
            end_hour,
            radius,
            unknown_1,
            only_buys_stolen_items,
            not_sell_buy,
            unknown_2,
        } if (record, signature) == ("FACT", "VENV") => {
            let mut out = Vec::with_capacity(12);
            for value in [start_hour, end_hour, radius] {
                out.extend_from_slice(&value.to_le_bytes());
            }
            out.extend_from_slice(&parse_hex_bytes(unknown_1, 2)?);
            out.push(u8::from(*only_buys_stolen_items));
            out.push(u8::from(*not_sell_buy));
            out.extend_from_slice(&parse_hex_bytes(unknown_2, 2)?);
            Ok(out)
        }
        SubrecordValue::Location {
            location_type,
            location_value,
            radius,
        } if matches!((record, signature), ("FACT", "PLVD") | ("PACK", "PLDT")) => {
            let mut out = Vec::with_capacity(12);
            out.extend_from_slice(&location_type.to_le_bytes());
            out.extend_from_slice(&parse_hex_u32(location_value)?.to_le_bytes());
            out.extend_from_slice(&radius.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::ObjectBounds {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        } if signature == "OBND" => {
            let mut out = Vec::with_capacity(12);
            for value in [min_x, min_y, min_z, max_x, max_y, max_z] {
                out.extend_from_slice(&value.to_le_bytes());
            }
            Ok(out)
        }
        SubrecordValue::PluginHeader {
            version,
            number_of_records,
            next_object_id,
        } if (record, signature) == ("TES4", "HEDR") => {
            ensure!(version.is_finite(), "TES4.HEDR version must be finite");
            let mut out = version.to_le_bytes().to_vec();
            out.extend_from_slice(&number_of_records.to_le_bytes());
            out.extend_from_slice(&parse_hex_u32(next_object_id)?.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::ItemData { value, weight } if (record, signature) == ("ARMO", "DATA") => {
            ensure!(weight.is_finite(), "ARMO.DATA weight must be finite");
            let mut out = value.to_le_bytes().to_vec();
            out.extend_from_slice(&weight.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::ArmorRating { value } if (record, signature) == ("ARMO", "DNAM") => {
            ensure!(value.is_finite(), "ARMO.DNAM armor rating must be finite");
            let scaled = (*value as f64 * 100.0).round();
            ensure!(
                (i32::MIN as f64..=i32::MAX as f64).contains(&scaled),
                "ARMO.DNAM armor rating is out of range"
            );
            Ok((scaled as i32).to_le_bytes().to_vec())
        }
        SubrecordValue::NpcAiData {
            aggression,
            confidence,
            energy_level,
            morality,
            mood,
            assistance,
            aggro_radius_behavior,
            unused,
            warn,
            warn_attack,
            attack,
        } if (record, signature) == ("NPC_", "AIDT") => {
            let mut out = vec![
                *aggression,
                *confidence,
                *energy_level,
                *morality,
                *mood,
                *assistance,
                u8::from(*aggro_radius_behavior),
                *unused,
            ];
            for value in [warn, warn_attack, attack] {
                out.extend_from_slice(&value.to_le_bytes());
            }
            Ok(out)
        }
        SubrecordValue::NpcPlayerSkills {
            skills,
            health,
            magicka,
            stamina,
            unused,
            far_away_model_distance,
            geared_up_weapons,
            trailing_unused,
        } if (record, signature) == ("NPC_", "DNAM") => {
            ensure!(
                skills.len() == NPC_SKILL_NAMES.len(),
                "NPC_.DNAM requires 18 skills"
            );
            ensure!(
                far_away_model_distance.is_finite(),
                "NPC_.DNAM far-away distance must be finite"
            );
            let mut out = Vec::with_capacity(52);
            for (expected, skill) in NPC_SKILL_NAMES.iter().zip(skills) {
                ensure!(
                    skill.name == *expected,
                    "NPC_.DNAM skill order/name mismatch"
                );
                out.push(skill.value);
            }
            for skill in skills {
                out.push(skill.offset);
            }
            out.extend_from_slice(&health.to_le_bytes());
            out.extend_from_slice(&magicka.to_le_bytes());
            out.extend_from_slice(&stamina.to_le_bytes());
            out.extend_from_slice(&parse_hex_bytes(unused, 2)?);
            out.extend_from_slice(&far_away_model_distance.to_le_bytes());
            out.push(*geared_up_weapons);
            out.extend_from_slice(&parse_hex_bytes(trailing_unused, 3)?);
            Ok(out)
        }
        SubrecordValue::LeveledListEntry {
            level,
            reference,
            count,
        } if matches!(record, "LVLN" | "LVLI") && signature == "LVLO" => {
            let mut out = Vec::with_capacity(12);
            out.extend_from_slice(&level.to_le_bytes());
            out.extend_from_slice(&[0, 0]);
            out.extend_from_slice(&parse_hex_u32(reference)?.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            out.extend_from_slice(&[0, 0]);
            Ok(out)
        }
        SubrecordValue::LeveledExtraData {
            owner,
            global_or_required_rank_raw,
            item_condition,
        } if matches!(record, "LVLN" | "LVLI") && signature == "COED" => {
            ensure!(
                item_condition.is_finite(),
                "{record}.COED condition must be finite"
            );
            let mut out = parse_hex_u32(owner)?.to_le_bytes().to_vec();
            out.extend_from_slice(&parse_hex_u32(global_or_required_rank_raw)?.to_le_bytes());
            out.extend_from_slice(&item_condition.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::SpellData {
            base_cost,
            flags,
            flags_unknown_bits,
            spell_type,
            charge_time,
            cast_type,
            delivery,
            cast_duration,
            range,
            half_cost_perk,
        } if (record, signature) == ("SPEL", "SPIT") => {
            ensure!(
                [charge_time, cast_duration, range]
                    .iter()
                    .all(|v| v.is_finite()),
                "SPEL.SPIT floats must be finite"
            );
            let flag_bits = encode_named_flags(
                flags,
                flags_unknown_bits,
                SPELL_FLAG_NAMES,
                "SPEL.SPIT flags",
            )?;
            let mut out = Vec::with_capacity(36);
            out.extend_from_slice(&base_cost.to_le_bytes());
            out.extend_from_slice(&flag_bits.to_le_bytes());
            out.extend_from_slice(&spell_type.to_le_bytes());
            out.extend_from_slice(&charge_time.to_le_bytes());
            out.extend_from_slice(&cast_type.to_le_bytes());
            out.extend_from_slice(&delivery.to_le_bytes());
            out.extend_from_slice(&cast_duration.to_le_bytes());
            out.extend_from_slice(&range.to_le_bytes());
            out.extend_from_slice(&parse_hex_u32(half_cost_perk)?.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::EffectParameters {
            magnitude,
            area,
            duration,
        } if matches!(record, "SPEL" | "ALCH") && signature == "EFIT" => {
            ensure!(
                magnitude.is_finite(),
                "{record}.EFIT magnitude must be finite"
            );
            let mut out = magnitude.to_le_bytes().to_vec();
            out.extend_from_slice(&area.to_le_bytes());
            out.extend_from_slice(&duration.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::MagicEffectData {
            flags,
            flags_unknown_bits,
            base_cost,
            associated_item,
            magic_skill,
            resist_value,
            counter_effect_count,
            casting_light,
            taper_weight,
            hit_shader,
            enchant_shader,
            minimum_skill_level,
            spellmaking_area,
            spellmaking_casting_time,
            taper_curve,
            taper_duration,
            second_actor_value_weight,
            archetype,
            primary_actor_value,
            projectile,
            explosion,
            casting_type,
            delivery,
            second_actor_value,
            casting_art,
            hit_effect_art,
            impact_data,
            skill_usage_multiplier,
            dual_casting_art,
            dual_casting_scale,
            enchant_art,
            hit_visuals,
            enchant_visuals,
            equip_ability,
            image_space_modifier,
            perk_to_apply,
            casting_sound_level,
            script_effect_ai_score,
            script_effect_ai_delay_time,
        } if (record, signature) == ("MGEF", "DATA") => {
            let floats = [
                *base_cost,
                *taper_weight,
                *spellmaking_casting_time,
                *taper_curve,
                *taper_duration,
                *second_actor_value_weight,
                *skill_usage_multiplier,
                *dual_casting_scale,
                *script_effect_ai_score,
                *script_effect_ai_delay_time,
            ];
            ensure!(
                floats.iter().all(|value| value.is_finite()),
                "MGEF.DATA floats must be finite"
            );
            let flag_bits = encode_named_flags(
                flags,
                flags_unknown_bits,
                MGEF_FLAG_NAMES,
                "MGEF.DATA flags",
            )?;
            let mut out = Vec::with_capacity(152);
            push_u32(&mut out, flag_bits);
            push_f32(&mut out, *base_cost);
            push_form(&mut out, associated_item)?;
            push_i32(&mut out, *magic_skill);
            push_i32(&mut out, *resist_value);
            out.extend_from_slice(&counter_effect_count.to_le_bytes());
            out.extend_from_slice(&[0, 0]);
            push_form(&mut out, casting_light)?;
            push_f32(&mut out, *taper_weight);
            push_form(&mut out, hit_shader)?;
            push_form(&mut out, enchant_shader)?;
            push_u32(&mut out, *minimum_skill_level);
            push_u32(&mut out, *spellmaking_area);
            for value in [
                spellmaking_casting_time,
                taper_curve,
                taper_duration,
                second_actor_value_weight,
            ] {
                push_f32(&mut out, *value);
            }
            push_u32(&mut out, *archetype);
            push_i32(&mut out, *primary_actor_value);
            push_form(&mut out, projectile)?;
            push_form(&mut out, explosion)?;
            push_u32(&mut out, *casting_type);
            push_u32(&mut out, *delivery);
            push_i32(&mut out, *second_actor_value);
            push_form(&mut out, casting_art)?;
            push_form(&mut out, hit_effect_art)?;
            push_form(&mut out, impact_data)?;
            push_f32(&mut out, *skill_usage_multiplier);
            push_form(&mut out, dual_casting_art)?;
            push_f32(&mut out, *dual_casting_scale);
            push_form(&mut out, enchant_art)?;
            for value in [
                hit_visuals,
                enchant_visuals,
                equip_ability,
                image_space_modifier,
                perk_to_apply,
            ] {
                push_form(&mut out, value)?;
            }
            push_u32(&mut out, *casting_sound_level);
            push_f32(&mut out, *script_effect_ai_score);
            push_f32(&mut out, *script_effect_ai_delay_time);
            ensure!(out.len() == 152, "internal MGEF.DATA size mismatch");
            Ok(out)
        }
        SubrecordValue::MagicEffectSounds { sounds } if (record, signature) == ("MGEF", "SNDD") => {
            let mut out = Vec::with_capacity(sounds.len() * 8);
            for sound in sounds {
                push_u32(&mut out, sound.sound_type);
                push_form(&mut out, &sound.sound)?;
            }
            Ok(out)
        }
        SubrecordValue::FloatArray { values } if (record, signature) == ("RACE", "PHWT") => {
            ensure!(
                values.len() == 16 && values.iter().all(|v| v.is_finite()),
                "RACE.PHWT requires 16 finite weights"
            );
            let mut out = Vec::with_capacity(64);
            for value in values {
                push_f32(&mut out, *value);
            }
            Ok(out)
        }
        SubrecordValue::RaceData {
            skill_boosts,
            unknown,
            male_height,
            female_height,
            male_weight,
            female_weight,
            flags,
            flags_unknown_bits,
            starting_health,
            starting_magicka,
            starting_stamina,
            base_carry_weight,
            base_mass,
            acceleration_rate,
            deceleration_rate,
            size,
            head_biped_object,
            hair_biped_object,
            injured_health_pct,
            shield_biped_object,
            health_regen,
            magicka_regen,
            stamina_regen,
            unarmed_damage,
            unarmed_reach,
            body_biped_object,
            aim_angle_tolerance,
            flight_radius,
            angular_acceleration_rate,
            angular_tolerance,
            flags_2,
            flags_2_unknown_bits,
            mount_offset_x,
            mount_offset_y,
            mount_offset_z,
            dismount_offset_x,
            dismount_offset_y,
            dismount_offset_z,
            mount_camera_offset_x,
            mount_camera_offset_y,
            mount_camera_offset_z,
        } if (record, signature) == ("RACE", "DATA") => {
            ensure!(
                skill_boosts.len() == 7,
                "RACE.DATA requires seven skill boosts"
            );
            let floats = [
                male_height,
                female_height,
                male_weight,
                female_weight,
                starting_health,
                starting_magicka,
                starting_stamina,
                base_carry_weight,
                base_mass,
                acceleration_rate,
                deceleration_rate,
                injured_health_pct,
                health_regen,
                magicka_regen,
                stamina_regen,
                unarmed_damage,
                unarmed_reach,
                aim_angle_tolerance,
                flight_radius,
                angular_acceleration_rate,
                angular_tolerance,
                mount_offset_x,
                mount_offset_y,
                mount_offset_z,
                dismount_offset_x,
                dismount_offset_y,
                dismount_offset_z,
                mount_camera_offset_x,
                mount_camera_offset_y,
                mount_camera_offset_z,
            ];
            ensure!(
                floats.iter().all(|v| v.is_finite()),
                "RACE.DATA floats must be finite"
            );
            let flag_bits = encode_named_flags(
                flags,
                flags_unknown_bits,
                RACE_FLAG_NAMES,
                "RACE.DATA flags",
            )?;
            let flag_2_bits = encode_named_flags(
                flags_2,
                flags_2_unknown_bits,
                RACE_FLAG_2_NAMES,
                "RACE.DATA flags_2",
            )?;
            let mut out = Vec::with_capacity(164);
            for skill in skill_boosts {
                out.push(skill.actor_value as u8);
                out.push(skill.boost as u8);
            }
            out.extend_from_slice(&parse_hex_bytes(unknown, 2)?);
            for value in [male_height, female_height, male_weight, female_weight] {
                push_f32(&mut out, *value);
            }
            push_u32(&mut out, flag_bits);
            for value in [
                starting_health,
                starting_magicka,
                starting_stamina,
                base_carry_weight,
                base_mass,
                acceleration_rate,
                deceleration_rate,
            ] {
                push_f32(&mut out, *value);
            }
            push_u32(&mut out, *size);
            push_i32(&mut out, *head_biped_object);
            push_i32(&mut out, *hair_biped_object);
            push_f32(&mut out, *injured_health_pct);
            push_i32(&mut out, *shield_biped_object);
            for value in [
                health_regen,
                magicka_regen,
                stamina_regen,
                unarmed_damage,
                unarmed_reach,
            ] {
                push_f32(&mut out, *value);
            }
            push_i32(&mut out, *body_biped_object);
            for value in [
                aim_angle_tolerance,
                flight_radius,
                angular_acceleration_rate,
                angular_tolerance,
            ] {
                push_f32(&mut out, *value);
            }
            push_u32(&mut out, flag_2_bits);
            for value in [
                mount_offset_x,
                mount_offset_y,
                mount_offset_z,
                dismount_offset_x,
                dismount_offset_y,
                dismount_offset_z,
                mount_camera_offset_x,
                mount_camera_offset_y,
                mount_camera_offset_z,
            ] {
                push_f32(&mut out, *value);
            }
            ensure!(out.len() == 164, "internal RACE.DATA size mismatch");
            Ok(out)
        }
        SubrecordValue::NpcConfiguration {
            flags,
            flags_unknown_bits,
            magicka_offset,
            stamina_offset,
            level,
            level_multiplier,
            calc_min_level,
            calc_max_level,
            speed_multiplier,
            disposition_base_unused,
            template_flags,
            template_flags_unknown_bits,
            health_offset,
            bleedout_override,
        } if (record, signature) == ("NPC_", "ACBS") => {
            let flag_bits =
                encode_named_flags(flags, flags_unknown_bits, NPC_FLAG_NAMES, "NPC_.ACBS flags")?;
            let uses_multiplier = flag_bits & 0x80 != 0;
            ensure!(
                uses_multiplier == level_multiplier.is_some() && uses_multiplier != level.is_some(),
                "NPC_.ACBS must contain level_multiplier exactly when pc_level_mult is set, otherwise level"
            );
            let raw_level = if let Some(multiplier) = level_multiplier {
                ensure!(
                    multiplier.is_finite() && (0.0..=65.535).contains(multiplier),
                    "NPC_.ACBS level_multiplier is out of range"
                );
                (multiplier * 1000.0).round() as u16
            } else {
                level.unwrap()
            };
            let template_bits = encode_named_flags(
                template_flags,
                template_flags_unknown_bits,
                NPC_TEMPLATE_FLAG_NAMES,
                "NPC_.ACBS template_flags",
            )?;
            ensure!(
                template_bits <= u16::MAX as u32,
                "NPC_.ACBS template flags exceed 16 bits"
            );
            let mut out = Vec::with_capacity(24);
            out.extend_from_slice(&flag_bits.to_le_bytes());
            out.extend_from_slice(&magicka_offset.to_le_bytes());
            out.extend_from_slice(&stamina_offset.to_le_bytes());
            out.extend_from_slice(&raw_level.to_le_bytes());
            out.extend_from_slice(&calc_min_level.to_le_bytes());
            out.extend_from_slice(&calc_max_level.to_le_bytes());
            out.extend_from_slice(&speed_multiplier.to_le_bytes());
            out.extend_from_slice(&disposition_base_unused.to_le_bytes());
            out.extend_from_slice(&(template_bits as u16).to_le_bytes());
            out.extend_from_slice(&health_offset.to_le_bytes());
            out.extend_from_slice(&bleedout_override.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::RelationshipData {
            parent,
            child,
            rank,
            unknown,
            flags,
            association_type,
        } if (record, signature) == ("RELA", "DATA") => {
            let mut out = parse_hex_u32(parent)?.to_le_bytes().to_vec();
            out.extend_from_slice(&parse_hex_u32(child)?.to_le_bytes());
            out.extend_from_slice(&rank.to_le_bytes());
            out.push(*unknown);
            out.push(*flags);
            out.extend_from_slice(&parse_hex_u32(association_type)?.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::SoundDescriptorValues {
            frequency_shift_percent,
            frequency_variance_percent,
            priority,
            db_variance,
            static_attenuation_db,
        } if (record, signature) == ("SNDR", "BNAM") => {
            ensure!(
                static_attenuation_db.is_finite(),
                "SNDR.BNAM attenuation must be finite"
            );
            let scaled = (*static_attenuation_db * 100.0).round();
            ensure!(
                (0.0..=u16::MAX as f32).contains(&scaled),
                "SNDR.BNAM attenuation is out of range"
            );
            let mut out = vec![
                *frequency_shift_percent as u8,
                *frequency_variance_percent as u8,
                *priority,
                *db_variance,
            ];
            out.extend_from_slice(&(scaled as u16).to_le_bytes());
            Ok(out)
        }
        SubrecordValue::SoundLoopValues {
            unknown_1,
            looping,
            unknown_2,
            rumble_send_value,
        } if (record, signature) == ("SNDR", "LNAM") => {
            Ok(vec![*unknown_1, *looping, *unknown_2, *rumble_send_value])
        }
        SubrecordValue::ArmorAddonData {
            male_priority,
            female_priority,
            male_weight_slider,
            female_weight_slider,
            unknown_1,
            detection_sound_value,
            unknown_2,
            weapon_adjust,
        } if (record, signature) == ("ARMA", "DNAM") => {
            ensure!(
                weapon_adjust.is_finite(),
                "ARMA.DNAM weapon_adjust must be finite"
            );
            let mut out = vec![
                *male_priority,
                *female_priority,
                *male_weight_slider,
                *female_weight_slider,
            ];
            out.extend_from_slice(&parse_hex_bytes(unknown_1, 2)?);
            out.extend_from_slice(&[*detection_sound_value, *unknown_2]);
            out.extend_from_slice(&weapon_adjust.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::IngestibleEffectData {
            value,
            flags,
            addiction,
            addiction_chance,
            consume_sound,
        } if (record, signature) == ("ALCH", "ENIT") => {
            ensure!(
                addiction_chance.is_finite(),
                "ALCH.ENIT addiction_chance must be finite"
            );
            let mut out = value.to_le_bytes().to_vec();
            out.extend_from_slice(&flags.to_le_bytes());
            out.extend_from_slice(&parse_hex_u32(addiction)?.to_le_bytes());
            out.extend_from_slice(&addiction_chance.to_le_bytes());
            out.extend_from_slice(&parse_hex_u32(consume_sound)?.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::DialogueData {
            do_all_before_repeating,
            category,
            subtype,
        } if (record, signature) == ("DIAL", "DATA") => {
            let mut out = vec![u8::from(*do_all_before_repeating), *category];
            out.extend_from_slice(&subtype.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::PackageData {
            general_flags,
            package_type,
            interrupt_override,
            preferred_speed,
            unknown_1,
            interrupt_flags,
            unknown_2,
        } if (record, signature) == ("PACK", "PKDT") => {
            let mut out = general_flags.to_le_bytes().to_vec();
            out.extend_from_slice(&[
                *package_type,
                *interrupt_override,
                *preferred_speed,
                *unknown_1,
            ]);
            out.extend_from_slice(&interrupt_flags.to_le_bytes());
            out.extend_from_slice(&parse_hex_bytes(unknown_2, 2)?);
            Ok(out)
        }
        SubrecordValue::PackageSchedule {
            month,
            day_of_week,
            date,
            hour,
            minute,
            unused,
            duration_minutes,
        } if (record, signature) == ("PACK", "PSDT") => {
            let mut out = vec![
                *month as u8,
                *day_of_week as u8,
                *date as u8,
                *hour as u8,
                *minute as u8,
            ];
            out.extend_from_slice(&parse_hex_bytes(unused, 3)?);
            out.extend_from_slice(&duration_minutes.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::PackageCounter {
            data_input_count,
            package_template,
            version_counter,
        } if (record, signature) == ("PACK", "PKCU") => {
            let mut out = data_input_count.to_le_bytes().to_vec();
            out.extend_from_slice(&parse_hex_u32(package_template)?.to_le_bytes());
            out.extend_from_slice(&version_counter.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::PackageTopicData { topic_type, data }
            if (record, signature) == ("PACK", "PDTO") =>
        {
            let mut out = topic_type.to_le_bytes().to_vec();
            out.extend_from_slice(&parse_hex_u32(data)?.to_le_bytes());
            Ok(out)
        }
        SubrecordValue::ModelInformation {
            textures,
            addon_nodes,
        } if matches!(
            (record, signature),
            ("ARMA", "MO2T" | "MO3T" | "MO4T" | "MO5T") | ("ALCH", "MODT")
        ) =>
        {
            let mut out = Vec::with_capacity(12 + textures.len() * 12 + addon_nodes.len() * 4);
            out.extend_from_slice(&2u32.to_le_bytes());
            out.extend_from_slice(
                &u32::try_from(textures.len())
                    .context("too many model textures")?
                    .to_le_bytes(),
            );
            out.extend_from_slice(
                &u32::try_from(addon_nodes.len())
                    .context("too many model addon nodes")?
                    .to_le_bytes(),
            );
            for texture in textures {
                out.extend_from_slice(&parse_hex_u32(&texture.file_hash)?.to_le_bytes());
                let extension = plugin_bytes(&texture.extension, record, signature)?;
                ensure!(
                    extension.len() <= 4,
                    "{record}.{signature} extension exceeds four bytes"
                );
                out.extend_from_slice(&extension);
                out.resize(out.len() + (4 - extension.len()), 0);
                out.extend_from_slice(&parse_hex_u32(&texture.folder_hash)?.to_le_bytes());
            }
            for addon_node in addon_nodes {
                out.extend_from_slice(&addon_node.to_le_bytes());
            }
            Ok(out)
        }
        _ => bail!("value type does not match the supported codec for {record}.{signature}"),
    }
}

fn parse_hex_u32(text: &str) -> Result<u32> {
    let digits = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X"));
    match digits {
        Some(digits) => Ok(u32::from_str_radix(digits, 16)?),
        None => Ok(text.parse()?),
    }
}

fn plugin_bytes(text: &str, record: &str, signature: &str) -> Result<Vec<u8>> {
    text.chars().map(|c| {
        let code = u32::from(c);
        ensure!(code <= 0xff && code != 0,
            "{record}.{signature} contains a character that cannot be represented as one plugin byte");
        Ok(code as u8)
    }).collect()
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn hex_u32(value: u32) -> String {
    format!("0x{value:08X}")
}
fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}
fn parse_hex_bytes(text: &str, expected: usize) -> Result<Vec<u8>> {
    let text = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    ensure!(
        text.len() == expected * 2,
        "expected {} hexadecimal bytes",
        expected
    );
    (0..expected)
        .map(|index| Ok(u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)?))
        .collect()
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn push_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn push_form(out: &mut Vec<u8>, value: &str) -> Result<()> {
    push_u32(out, parse_hex_u32(value)?);
    Ok(())
}
fn decode_flags32(bytes: &[u8], names: &[(u32, &str)]) -> SubrecordValue {
    let bits = read_u32(bytes, 0);
    let known_mask = names.iter().fold(0u32, |mask, (bit, _)| mask | (1 << bit));
    SubrecordValue::Flags32 {
        set: names
            .iter()
            .filter(|(bit, _)| bits & (1 << bit) != 0)
            .map(|(_, name)| (*name).to_owned())
            .collect(),
        unknown_bits: hex_u32(bits & !known_mask),
    }
}

fn decode_named_flags(bits: u32, names: &[(u32, &str)]) -> (Vec<String>, String) {
    let known_mask = names.iter().fold(0u32, |mask, (bit, _)| mask | (1 << bit));
    let set = names
        .iter()
        .filter(|(bit, _)| bits & (1 << bit) != 0)
        .map(|(_, name)| (*name).to_owned())
        .collect();
    (set, hex_u32(bits & !known_mask))
}

fn decode_flags8(bits: u8, names: &[(u32, &str)]) -> SubrecordValue {
    let (set, unknown_bits) = decode_named_flags(bits as u32, names);
    SubrecordValue::Flags8 { set, unknown_bits }
}

fn encode_named_flags(
    set: &[String],
    unknown_bits: &str,
    names: &[(u32, &str)],
    context: &str,
) -> Result<u32> {
    let known_mask = names.iter().fold(0u32, |mask, (bit, _)| mask | (1 << bit));
    let mut bits = parse_hex_u32(unknown_bits)?;
    ensure!(
        bits & known_mask == 0,
        "{context} unknown bits overlap named flags"
    );
    for name in set {
        let bit = names
            .iter()
            .find(|(_, candidate)| candidate == name)
            .map(|(bit, _)| *bit)
            .with_context(|| format!("unknown {context} flag {name:?}"))?;
        bits |= 1 << bit;
    }
    Ok(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_id_is_lossless_and_editable() {
        let raw = b"IronSword\0";
        let value = decode("WEAP", "EDID", raw).unwrap();
        assert_eq!(encode("WEAP", "EDID", &value).unwrap(), raw);
    }

    #[test]
    fn context_prevents_data_guessing() {
        assert!(decode("BOOK", "DATA", &[0, 0, 0, 0]).is_none());
    }

    #[test]
    fn non_json_floats_remain_raw_blobs() {
        assert!(decode("GLOB", "FLTV", &f32::NAN.to_le_bytes()).is_none());
    }

    fn assert_codec_round_trip(record: &str, signature: &str, raw: &[u8]) -> SubrecordValue {
        let value = decode(record, signature, raw)
            .unwrap_or_else(|| panic!("{record}.{signature} did not decode"));
        assert_eq!(encode(record, signature, &value).unwrap(), raw);
        value
    }

    #[test]
    fn npc_configuration_uses_pc_level_multiplier_flag() {
        let mut raw = vec![0u8; 24];
        raw[0..4].copy_from_slice(&0x0000_0081u32.to_le_bytes());
        raw[4..6].copy_from_slice(&(-25i16).to_le_bytes());
        raw[6..8].copy_from_slice(&10i16.to_le_bytes());
        raw[8..10].copy_from_slice(&1500u16.to_le_bytes());
        let value = assert_codec_round_trip("NPC_", "ACBS", &raw);
        assert!(matches!(value, SubrecordValue::NpcConfiguration {
            level: None, level_multiplier: Some(value), ..
        } if value == 1.5));
        assert_eq!(display_name("NPC_", "ACBS"), Some("Configuration"));
    }

    #[test]
    fn major_requested_structs_are_lossless() {
        assert_codec_round_trip("SPEL", "SPIT", &[0; 36]);
        assert_codec_round_trip("SPEL", "EFIT", &[0; 12]);
        assert_codec_round_trip("MGEF", "DATA", &[0; 152]);
        assert_codec_round_trip("RACE", "DATA", &[0; 164]);
        assert_codec_round_trip("NPC_", "AIDT", &[0; 20]);
        assert_codec_round_trip("NPC_", "DNAM", &[0; 52]);
        assert_codec_round_trip("ARMO", "BOD2", &[0; 8]);
    }

    #[test]
    fn leveled_lists_and_outfits_are_lossless() {
        let mut entry = vec![0u8; 12];
        entry[0..2].copy_from_slice(&12u16.to_le_bytes());
        entry[4..8].copy_from_slice(&0x0100_1234u32.to_le_bytes());
        entry[8..10].copy_from_slice(&3u16.to_le_bytes());
        assert_codec_round_trip("LVLI", "LVLO", &entry);
        assert_codec_round_trip("LVLN", "LVLF", &[3]);
        assert_codec_round_trip("LVLI", "COED", &[0; 12]);
        assert_codec_round_trip("OTFT", "INAM", &[1, 0, 0, 0, 2, 0, 0, 0]);
    }

    #[test]
    fn plugin_header_and_master_fields_are_lossless() {
        let mut hedr = 1.7f32.to_le_bytes().to_vec();
        hedr.extend_from_slice(&42u32.to_le_bytes());
        hedr.extend_from_slice(&0x800u32.to_le_bytes());
        assert_codec_round_trip("TES4", "HEDR", &hedr);
        assert_codec_round_trip("TES4", "CNAM", b"Author\0");
        assert_codec_round_trip("TES4", "MAST", b"Skyrim.esm\0");
        assert_codec_round_trip("TES4", "DATA", &123_456u64.to_le_bytes());
        assert_codec_round_trip("TES4", "ONAM", &[1, 0, 0, 0, 2, 0, 0, 0]);
    }
}
