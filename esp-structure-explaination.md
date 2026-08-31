# ESP JSON structure guide for LLMs

This file describes how to create Skyrim Special Edition plugin JSON for `tes5edit-rust-json`, then compile it into an `.esp`, `.esm`, or `.esl`. The current output format is `tes5edit-rust-json/v3`.

## Recommended workflow

For a nontrivial mod, start from a small valid plugin containing the desired record families:

```powershell
tes5edit-rust-json to-json "Seed.esp" "Seed.esp.json"
```

Edit the JSON while retaining its TES4 header, group hierarchy, record order, and any `raw_bytes` values you do not understand. Then compile it:

```powershell
tes5edit-rust-json from-json "Seed.esp.json" "Generated.esp"
```

The CLI also accepts a JSON-pack directory:

```powershell
tes5edit-rust-json to-json-pack "Seed.esp" "Seed.esp.json-pack"
tes5edit-rust-json from-json "Seed.esp.json-pack" "Generated.esp"
```

Always load the generated plugin in SSEEdit and check it for errors before distributing or launching the game.

## Top-level document

```json
{
  "format": "tes5edit-rust-json/v3",
  "source_file": "Generated.esp",
  "elements": [],
  "blobs": {}
}
```

- `format` must be `tes5edit-rust-json/v3` for new files.
- `source_file` is informational and may be `null` or omitted.
- `elements` is ordered and must begin with the `TES4` record.
- Clean JSON normally omits `blobs`. It is used only for preserved original compressed streams or old v2 input.

## Records and groups

A record has `kind: "record"`, a four-byte `signature`, a hexadecimal `form_id`, and ordered `subrecords`:

```json
{
  "kind": "record",
  "signature": "KYWD",
  "form_id": "0x01000800",
  "subrecords": [
    {
      "signature": "EDID",
      "value": { "type": "zstring", "text": "MyGeneratedKeyword" }
    },
    {
      "signature": "CNAM",
      "value": { "type": "color_rgba", "red": 255, "alpha": 255 }
    }
  ]
}
```

A top-level record family normally belongs inside a type-0 `GRUP`. `label` is the four signature bytes interpreted as a little-endian `u32`. For `KYWD`, the value is `0x4457594B`, or `1146571083`:

```json
{
  "kind": "group",
  "label": 1146571083,
  "label_signature": "KYWD",
  "elements": [
    { "kind": "record", "signature": "KYWD", "form_id": "0x01000800", "subrecords": [] }
  ]
}
```

Do not infer a field layout from its subrecord signature alone. `DATA`, `DNAM`, and similar signatures have different binary layouts in different record families. Copy the structure produced by this converter or use only a value type already demonstrated for the same record/subrecord pair.

## TES4 header

Every plugin must begin with one `TES4` record. A minimal header is:

```json
{
  "kind": "record",
  "signature": "TES4",
  "subrecords": [
    {
      "signature": "HEDR",
      "value": {
        "type": "plugin_header",
        "version": 1.7,
        "next_object_id": "0x00000800"
      }
    }
  ]
}
```

Use TES4 record flags to select plugin type. The raw `flags` field remains authoritative for record flags; exported headers also include a readable `flags_value`. Common combinations are:

- ESP: neither ESM nor ESL bit set.
- ESM: ESM bit `0x00000001` set.
- ESL-flagged ESP: ESL bit `0x00000200` set.
- ESL master: both bits set.

When creating a plugin from scratch, prefer copying a TES4 header exported from an existing empty plugin of the intended type. Masters require ordered `MAST` and `DATA` subrecords.

## FormIDs

FormIDs are fixed-width hexadecimal strings such as `"0x01000800"`. A zero/null reference is `"0x00000000"` and may be omitted when default trimming is enabled.

- The high byte is the file/load-order portion used by the source plugin representation.
- The low three bytes identify an object in a full plugin.
- Do not reuse a FormID within the same plugin.
- Keep `HEDR.next_object_id` above locally allocated object IDs.
- References to masters must use the correct master index implied by the TES4 `MAST` order.
- Compact ESL FormID allocation has additional range and load-order rules. Use SSEEdit to compact and flag a plugin rather than guessing.

## Subrecord values

Each subrecord has a four-byte `signature` and one authoritative `value`:

```json
{
  "signature": "EDID",
  "display_name": "Editor ID",
  "value": { "type": "zstring", "text": "ExampleEditorID" }
}
```

Frequently useful value shapes include:

```json
{ "type": "form_id", "id": "0x01000800" }
{ "type": "form_id_array", "ids": ["0x01000800", "0x01000801"] }
{ "type": "u8", "value": 1 }
{ "type": "u16", "value": 10 }
{ "type": "u32", "value": 100 }
{ "type": "i32", "value": -1 }
{ "type": "f32", "value": 1.5 }
{ "type": "zstring", "text": "EditorText" }
{ "type": "localized_string_id", "id": "0x00000123" }
{ "type": "empty" }
```

The accepted type is contextual. For example, `FLST.LNAM` accepts `form_id`, while `GLOB.FLTV` accepts `f32`. A type that is valid elsewhere will be rejected when it does not match the record and subrecord.

Unknown data is explicit and inline:

```json
{
  "signature": "VMAD",
  "value": { "type": "raw_bytes", "base64": "..." }
}
```

Never invent `raw_bytes`. Preserve it from an exported plugin unless you have implemented the exact binary schema independently.

## Default-value trimming

Trimmed JSON may omit typed members whose value is binary-default-like:

- integer or floating-point zero;
- `false`;
- an empty string or list;
- a zero/null FormID;
- a hexadecimal string containing only zero bits;
- the entire `value` property for a schema-defined empty subrecord.

The importer restores these omitted members to zero, false, or empty. Do not omit required non-default values. The `type` discriminator itself is never optional, except when a schema-defined `empty` value is represented by the absence of `value`.

## Compression and exact bytes

Record compression is derived from record flag `0x00040000`. Clean JSON contains values for the uncompressed subrecords and the writer creates a new valid zlib stream. `original_compressed_ref` and its blob are optional preservation data used only for byte-identical no-edit reconstruction; LLM-generated mods do not need them.

## Minimal from-scratch plugin

This creates a structurally minimal ESP containing only its TES4 header:

```json
{
  "format": "tes5edit-rust-json/v3",
  "source_file": "Generated.esp",
  "elements": [
    {
      "kind": "record",
      "signature": "TES4",
      "subrecords": [
        {
          "signature": "HEDR",
          "value": {
            "type": "plugin_header",
            "version": 1.7,
            "next_object_id": "0x00000800"
          }
        }
      ]
    }
  ]
}
```

Save it as `Generated.esp.json`, then run:

```powershell
tes5edit-rust-json from-json "Generated.esp.json" "Generated.esp"
```

For real content, export a seed plugin and add records inside its existing family groups. This avoids accidental mistakes in header flags, master indexing, group labels, and record ordering.

## Failure behavior

The compiler intentionally rejects unsafe JSON, including:

- a document not beginning with `TES4`;
- an unknown format version;
- a typed value used with the wrong record/subrecord pair;
- invalid Base64 or hexadecimal values;
- invalid fixed payload sizes;
- non-finite JSON floats;
- overlapping named and unknown flag bits;
- missing raw data for a payload with no typed value.

Treat an error as a schema problem to correct in JSON. Do not work around it by changing signatures or inserting guessed bytes.
