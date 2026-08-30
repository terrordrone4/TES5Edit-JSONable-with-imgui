# Clean JSON export: no more visible blobs

## Goal

Add an editable JSON export mode that resembles xEdit's schema-driven editor: expose proven structured values while keeping raw binary storage out of the files users normally edit.

The `blob_*` identifiers are converter storage references, not Skyrim fields. They should disappear wherever the typed `value` is sufficient to reconstruct the exact subrecord payload. Unsupported or ambiguous payloads must remain losslessly preserved in a sidecar rather than being guessed or discarded.

## Proposed behavior

- For a subrecord with a proven, lossless `value` codec, omit `data_ref` from editable JSON and do not retain its raw payload blob solely for provenance.
- For unsupported, ambiguous, malformed, or non-JSON-safe subrecords, retain the raw bytes in a shared sidecar blob table.
- In JSON-pack output, keep opaque payloads in `__blobs__.json`; record-family fragments should contain references only when raw storage is genuinely required.
- Preserve `original_compressed_ref` when it is needed to reproduce the original compressed stream byte-for-byte. If exact compressed bytes are not requested in a future export profile, allow recompression from the reconstructed payload.
- Continue accepting existing `tes5edit-rust-json/v1` and v2 files containing `data_ref` and inline `data_base64`.
- Keep `value` authoritative during import. When `value` exists, encode it with the record/subrecord-specific codec; never silently fall back to stale raw bytes after an invalid edit.

## Safety requirements

- Do not infer layouts from a subrecord signature alone. Codec selection must retain record context and fixed-size/version branches.
- Only remove a typed field's raw blob after decode/encode equality is proven for representative data and real fixtures.
- Preserve unknown flag bits and reject overlap between named and unknown bits.
- Never discard unsupported data merely to produce blob-free output.
- No-edit fixture round trips must remain byte-identical in the lossless profile, including compressed records.

## Suggested implementation

1. Introduce an explicit export profile, for example `lossless` and `clean`, instead of changing v2 serialization implicitly.
2. Add a traversal that determines which blobs are required:
   - every subrecord without `value` requires its `data_ref`;
   - compressed records may require `original_compressed_ref` in the lossless profile;
   - typed subrecord blobs can be pruned in the clean profile.
3. Serialize clean single-file JSON with only the required blob map. Serialize JSON-pack fragments with opaque references and keep their data in `__blobs__.json`.
4. Make import validation produce precise errors for missing raw data, mismatched value variants, invalid Base64, and dangling blob references.
5. Consider deterministic content-addressed blob IDs later; it is separate from removing visible blob plumbing.

## Verification

- Unit-test required-blob collection and pruning.
- Test a typed subrecord with no `data_ref` imports correctly from `value` alone.
- Test an unsupported subrecord remains byte-identical through its sidecar blob.
- Test edited typed values take precedence and encode correctly.
- Test compressed unchanged records remain byte-identical in the lossless profile.
- Test legacy v1 and current v2 imports remain supported.
- Run `cargo fmt --check`, `cargo test`, and all real-plugin fixture round trips.

## Important limitation

A completely blob-free lossless export is only possible when every encountered payload has a proven inverse codec. Until then, clean export should hide and minimize blobs, not delete unknown bytes or pretend xEdit-style schema coverage exists where it does not.
