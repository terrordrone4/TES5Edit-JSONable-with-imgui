<p align="center">
  <img src="docs/assets/skyrim-logo.png" alt="The Elder Scrolls V: Skyrim icon" width="128">
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="docs/assets/dear-imgui-logo.jpg" alt="Dear ImGui logo" width="360">
</p>

<h1 align="center">TES5Edit Rust JSON</h1>

<p align="center"><em>Made by GPT-5.6</em></p>

A Windows desktop utility that structurally converts Skyrim `.esp`, `.esm`, and `.esl` plugins to editable JSON and rebuilds plugins from that JSON.

## Source layout

```text
src/
  plugin.rs                    binary-format portal and data model
  plugin/
    read.rs                    public plugin-read API
    write.rs                   public plugin-write API
    io-utils.rs                shared plugin path/I/O validation
    value_codec/
      mod.rs                   typed binary/JSON codec
      fields.rs                record/subrecord field-kind routing
      display_names/           xEdit labels split by record family
  imgui.rs                     GUI portal (`run`)
  imgui/
    app.rs                     application screen and conversion actions
    utils.rs                   shared desktop/Explorer integration
    components/
      output_log.rs            newest-first scrollable log and Open folder items
  main.rs                      GUI/CLI entry point
```

## Format and fidelity

The converter follows xEdit's TES5 binary layer in `Core/wbImplementation.pas` and `Core/wbDefinitionsTES5.pas`:

- 24-byte main-record and `GRUP` headers
- recursive `GRUP` sizes (including the header)
- ordered subrecords with 4-byte signatures and 16-bit sizes
- `XXXX` extended-size markers for subrecords larger than 65,535 bytes
- record flag `0x00040000`, a 32-bit uncompressed-size prefix, and zlib data
- xEdit's four-byte signatures containing control bytes, including IMAD's `#00IAD` family

Version-2 JSON keeps record structure readable: each subrecord contains a small `data_ref`, while the Base64 payloads live in the final top-level `blobs` object. For fields backed by a verified SSEEdit definition, an editable `value` is also emitted (for example `EDID` as a `zstring`, FLST/FSTP references as hexadecimal `form_id` values, colors as RGBA channels, and selected numeric primitives). When `value` is present the writer rebuilds the bytes from it; remove `value` to edit the raw blob directly. `text_preview` is informational. Header values, ordering, group hierarchy, unknown fields, and compression state are preserved.

Schema-aware coverage includes the commonly edited fields of NPCs, leveled actors/items, magic effects, spells, outfits, armor, factions, and races. Major compound values include `NPC_.ACBS` Configuration, AI and player skills; leveled-list entries and extra data; the complete 152-byte `MGEF.DATA`; spell metadata and effect parameters; outfit item arrays; armor biped/data/rating fields; faction relations, flags, crime and vendor settings; and the complete 164-byte `RACE.DATA`, attacks, biped slots, tint references, and phoneme weights. Known fields also include xEdit's `display_name` beside their signature. Unknown bits are retained explicitly, and unsupported or context-dependent data remains in its lossless blob.

The `TES4` main header exposes named master/localized/light-plugin flags, header version, record count, next object ID, author, description, master dependencies, master metadata, overridden forms, and interior-cell count. Raw unknown flag bits remain explicit and lossless.

The reader remains compatible with version-1 JSON that stores `data_base64` inline.

The GUI can optionally export a JSON pack named `<plugin>.json-pack`. It contains `TES4.json` plus one JSON fragment for each top-level group signature (`KYWD.json`, `TXST.json`, and so on). Those fragments contain only blob IDs; the shared `__blobs__.json` file maps every ID to its Base64 value. `pack-manifest.json` preserves the original top-level order for exact reconstruction. Folder import automatically loads the manifest, shared blob dictionary, and fragments before writing the plugin.

`form_id` values are emitted as fixed-width hexadecimal strings (for example, `"0x0300AB61"`). The reader also accepts decimal `form_id` values from JSON created by older releases. The redundant `compressed` property is omitted; compression is derived from the record flags.

Numeric record/group header fields whose value is zero are omitted from JSON and restored as zero when imported. This includes zero-valued `flags`, `form_id`, `revision`, `version`, `unknown`, `label`, `group_type`, and `stamp` fields.

Compressed records retain an `original_compressed_ref` into the same blob table. If their subrecords are unchanged, the writer reuses those exact bytes for a byte-identical round trip. If a subrecord is edited, the writer detects the change and creates a fresh valid zlib stream.

This remains deliberately conservative rather than pretending to be xEdit's entire schema engine. The TES4 localization flag is honored: ordinary strings become editable text and localized strings become explicit string-table IDs. Resolving those IDs to translated text still requires external `.STRINGS` files. VMAD, conditions, models' opaque hash data, and a few sequence-dependent arrays remain blob-backed. That choice keeps unknown and unsupported records round-trip safe.

## Build and run

```powershell
cargo run --release
```

Run tests with `cargo test`. A successful no-edit round trip is expected to be byte-identical; semantic content is always retained.

The test suite recursively discovers every `.esp`, `.esm`, and `.esl` below `examples/in`, passes each through binary → JSON → binary, requires byte equality, and reparses the result.

The same conversion engine is available for scripting:

```powershell
tes5edit-rust-json to-json MyMod.esp MyMod.esp.json
tes5edit-rust-json to-json-pack MyMod.esp MyMod.esp.json-pack
tes5edit-rust-json from-json MyMod.esp.json Rebuilt.esp
```

## Artwork credits

- [Dear ImGui logo](https://www.dearimgui.com/) by Albane Kim.
- [Skyrim icon](https://commons.wikimedia.org/wiki/File:Skyrim_logo.png) by Bethesda Software, distributed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). The image is reproduced without modification.
