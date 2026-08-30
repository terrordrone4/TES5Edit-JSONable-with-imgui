# TES5Edit Rust JSON Agent Notes

## Project goal

This is a lossless, bidirectional Skyrim plugin (`.esm`, `.esp`, `.esl`) to JSON converter. Raw subrecord payloads remain in the blob table, while fields with a proven xEdit schema also receive an editable `value` object. JSON edits must encode back to the exact TES5 binary layout.

The current schema-resolution work lives in `src/plugin/value_codec/`. `mod.rs` owns codec behavior, `fields.rs` owns field-kind routing, and `display_names/` partitions xEdit labels by record family. Do not guess a codec from a four-byte subrecord signature alone: signatures such as `DATA` have different layouts in different record types. Every decoder and encoder must be selected by at least `(record signature, subrecord signature, payload size)` when size is fixed.

## Authoritative schema source

Use the sibling xEdit Pascal checkout at `<some-folder>\SSEdit`, especially:

- `Core/wbDefinitionsTES5.pas` for Skyrim record/subrecord declarations and xEdit display names.
- `Core/wbDefinitionsCommon.pas` for shared flag definitions, union deciders, value scaling, and callbacks.

Useful searches:

```powershell
rg -n "wbStruct\(ACBS|wbRecord\(NPC_" <some-folder>\SSEdit\Core\wbDefinitionsTES5.pas
rg -n "wbACBSLevelDecider|wbACBSLevelMultAfter|wbTemplateFlags" <some-folder>\SSEdit\Core -g '*.pas'
```

Read every referenced helper, not only the top-level `wbStruct`. Helpers may change field interpretation, clamp values, divide stored integers for display, or choose a union member based on another field.

## Resolution mechanics

1. Locate the record definition in `wbDefinitionsTES5.pas`.
2. Find the subrecord by signature and record context; preserve its xEdit display name in notes/tests when useful.
3. Expand referenced helpers (`wbStruct`, `wbUnion`, flag definitions, deciders, `SetAfterLoad`, value formatters) until the byte layout and display semantics are unambiguous.
4. Add a descriptive variant to `SubrecordValue`.
5. Decode only when record, signature, and payload length match the proven schema. Return `None` for malformed, ambiguous, or non-JSON-safe values so the blob stays authoritative.
6. Add the inverse encoder with strict validation. Named flags must retain unknown bits separately so newer or unusual plugins remain lossless.
7. Test decode/encode equality on representative bytes, semantic behavior such as union selection, JSON serialization, and a real fixture.
8. Run the whole fixture round-trip suite. Binary output must equal every original plugin byte-for-byte.

The writer uses `value` when present and otherwise uses `data_ref`. The original blob remains available after decoding, but an edited `value` intentionally takes precedence. Compressed records preserve their original compressed stream only when their reconstructed uncompressed payload is unchanged.

## Implemented requested schema pass

Source: `Core/wbDefinitionsTES5.pas`, inside `wbRecord(NPC_, 'Non-Player Character', ...)`.

`ACBS` is exactly 24 bytes:

| Offset | Size | xEdit name | Binary type |
|---:|---:|---|---|
| 0 | 4 | Flags | `u32` bit flags |
| 4 | 2 | Magicka Offset | `i16` |
| 6 | 2 | Stamina Offset | `i16` |
| 8 | 2 | Level / Level Mult | `u16`; union selected by Flags bit 7 |
| 10 | 2 | Calc min level | `u16` |
| 12 | 2 | Calc max level | `u16` |
| 14 | 2 | Speed Multiplier | `u16` |
| 16 | 2 | Disposition Base (unused) | `i16` |
| 18 | 2 | Template Flags | `u16` bit flags |
| 20 | 2 | Health Offset | `i16` |
| 22 | 2 | Bleedout Override | `u16` |

When actor Flags bit 7 (`PC Level Mult`) is clear, offset 8 is `level`. When set, xEdit displays the stored integer divided by 1000 as `level_multiplier`. Encoding multiplies the JSON value by 1000 and rounds. xEdit's after-load callback clamps stored multipliers to 100..10000; decide whether the Rust encoder should enforce that narrower range before considering this codec complete.

The implementation lives in `src/plugin/value_codec/mod.rs` as `SubrecordValue::NpcConfiguration`. Actor and template flags use readable snake-case names plus a hexadecimal `*_unknown_bits` field. The broader pass now also covers fixed-layout and primitive fields for `NPC_`, `LVLN`, `LVLI`, `MGEF`, `SPEL`, `OTFT`, `ARMO`, `FACT`, and `RACE`; consult the enum and its decode match for the exact current list.

Primary fixture requested by the user:

```text
examples/in/Fertility Mode.esm
```

Expected output: NPC_ records with `ACBS` subrecords should contain a structured `value` object with flags, offsets, level semantics, level bounds, speed, template flags, health offset, and bleedout override instead of being blob-only.

## Remaining expansion work

- Localized `FULL`, `DESC`, `DNAM`, and `SHRT` fields honor TES4 localization state and emit string-table IDs. Resolving IDs to translated text still needs external `.STRINGS` file support.
- VMAD, conditions, model texture hashes, destructible data, and sequence-dependent groups still need dedicated codecs.
- Some RACE head/morph/tint support is primitive-only; `MPAI`, `MPAV`, and opaque model info remain blobs.
- MGEF sound arrays are supported, but conditions and VMAD remain blobs.
- Decide whether ACBS multiplier editing should enforce xEdit's post-load 0.1..10.0 clamp rather than merely the binary `u16` range.
- Continue adding focused mutation tests for every newly editable variant, in addition to byte-identical fixture tests.

## Broader backlog

- Continue resolving high-value compound fields using the same Pascal-driven method; prioritize common fields in the included example plugins.
- Avoid a generic signature-only registry. A table keyed by record/subrecord is safer, with explicit size/version branches where xEdit defines variants.
- Introduce reusable typed helpers for 8/16/32-bit named flags if several schemas need them, while keeping JSON readable and unknown bits lossless.
- Add schema display names only from xEdit definitions, never invented labels.
- Document newly supported pairs in the README once the codec and fixture tests pass.

## TES4 plugin header

The main `TES4` record now exposes `flags_value.plugin_type` as `esp`, `esm`, `esl`, or `esl_master`; this is derived from the actual ESM/ESL bits, not the filename extension. Other named header flags and unknown bits remain lossless. `HEDR`, author, description, master filenames and metadata, overridden forms, and interior-cell count have typed codecs. Header flag logic lives in `src/plugin/record-flags.rs`; header field routing and display names live under `src/plugin/value_codec/`.

## Verification commands

```powershell
cargo fmt --check
cargo test
cargo run -- to-json "examples/in/Fertility Mode.esm" "target/fertility-mode.json"
rg -n -m 3 '"signature": "ACBS"|"type": "npc_configuration"' target/fertility-mode.json
cargo run -- from-json "target/fertility-mode.json" "target/Fertility Mode.roundtrip.esm"
```

Compare the round-trip bytes with PowerShell `Get-FileHash` or the existing Rust fixture test. Do not commit generated `target` output.

## Safety and compatibility invariants

- Preserve unknown subrecords as blobs; unsupported is safer than guessed decoding.
- Preserve unknown flag bits and reject overlap with named bits on encode.
- Reject mismatched `SubrecordValue` variants rather than silently emitting incorrect bytes.
- Keep legacy `tes5edit-rust-json/v1` and current v2 input compatibility unless a deliberate format migration is requested.
- Do not remove `data_ref` merely because a `value` exists; it supports provenance and lossless fallback.
