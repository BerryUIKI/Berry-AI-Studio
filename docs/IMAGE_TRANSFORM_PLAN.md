# Image transformation plan (issue #118)

Status: approved product direction; **planned, not implemented**. Target: after the Omera identity/migration stabilization and release-safety gates in [DELIVERY_ROADMAP.md](DELIVERY_ROADMAP.md). This plan translates the workflow suggested in [#118](https://github.com/BerryUIKI/Omera/issues/118) into Omera's library import, batch action and export surfaces. It does not add ComfyUI nodes or require ComfyUI at runtime.

## User outcomes and rollout order

1. **Export improvements (first).** Export selected images to JPEG, WebP or PNG with a working quality control where the codec supports it, optional resize, explicit metadata policy, collision handling, and accurate per-file results. Export never mutates library assets.
2. **Transform during managed import (second).** Offer compression/conversion while copying into a managed vault. Decode and validate each staged derivative before registering it. The external source remains intact by default. Extend the pipeline harvester only after its source-retention and cleanup behavior has been reconciled with this workflow.
3. **Transform existing managed assets (third).** Add a gallery batch action. Default to retaining the original; offer an explicit archive choice or move-to-system-Trash choice after the derivative and catalog update are verified. Never silently alter external linked folders.
4. **Advanced controls (later).** Target-size mode, percentage scaling, pixel-multiple alignment and reusable presets can follow measured demand. A target byte size is best effort and may require lowering quality or dimensions; do not promise visually lossless results.

This is a separate post-stabilization workstream, not part of the Berry-to-Omera migration or a release blocker. Export improvements can begin after the export/file-operation safety contracts are reviewed; import and post-import replacement require lead-owned persistence and filesystem contracts. Do not implement all phases in one PR.

## Current baseline and gaps

`ExportOptions` already exposes `original`/`webp`/`jpeg`/`png`, quality, maximum edge, privacy, sidecars, filename template, directory/ZIP, and showcase export. `ExportModal.vue` exposes the main controls, and `execute_batch_export` processes images in chunks of 16. Preserve these capabilities and their IPC compatibility.

Code review of the current `dev` baseline found the following preconditions for phase 1:

- `process_single_image` passes quality to the JPEG encoder, but the WebP branch calls `write_to(WebP)` without using `options.quality`. Do not present the current WebP slider as lossy quality control.
- `original` plus resize or privacy processing falls through to PNG encoding while the filename keeps the source extension. Output bytes, declared format and extension must agree. If the source codec cannot support the requested processing, reject it or require an explicit output format.
- Directory export uses `fs::write`, ZIP creation uses `File::create`, and the current write paths do not consistently propagate output/sidecar/ZIP entry errors into per-file success. Define a safe collision policy and count success only after every required write has completed. Do not silently overwrite an existing asset or ZIP.
- The raster re-encode path strips embedded metadata. The UI's existing privacy choices cannot imply preservation of EXIF, ICC or AI workflow data when that preservation is not implemented. Sidecar contents must follow the same effective privacy choice.

The existing source of truth is `crates/berry-domain/src/export.rs`, `crates/berry-scan/src/export.rs`, `src/components/ExportModal.vue`, and the `export_files_batch` IPC entry in [IPC_REFERENCE.md](IPC_REFERENCE.md). Revalidate these observations against the implementation baseline before coding.

## Workflow and safety decisions

| Context | Source policy | Output and catalog policy |
| --- | --- | --- |
| Export | Read-only source | Write to a separate destination; reject or explicitly resolve name collisions; never change library records |
| Managed import | External source retained by default | Stage and verify a derivative in managed storage, then register the final path and extracted metadata |
| Pipeline harvest | Respect its existing grace period and source cleanup semantics | Keep transformation and cleanup as separate, recoverable steps; no early source removal |
| Existing managed asset | Retain original by default; archive or system Trash only by explicit user choice | Stage derivative, verify, reconcile catalog identity/relations, then archive/trash source |
| Linked external folder | Never rewrite or remove the external file through this feature | Allow export or an explicit copy into managed storage instead |

The backend owns canonical paths and registered roots. Refuse source/output path aliasing, unsafe traversal, unsupported animated/multi-frame inputs, and format/alpha combinations that would lose content without an explicit decision. Do not silently flatten transparency to JPEG. Preserve orientation and color appearance or report unsupported metadata preservation before writing. Lossy re-encoding is never described as lossless.

Each transformation should use a bounded worker queue and a staged output in the destination filesystem. Validate that output exists, decodes, has the expected dimensions/container, and satisfies the chosen metadata policy. Publish with collision-safe atomic operations where supported; record partial failures rather than claiming the whole batch succeeded. On cancellation/crash, retain originals and provide a recoverable state; never fall back from failed system Trash to permanent delete. A database commit and filesystem rename cannot be one atomic transaction, so the lead must specify a durable reconciliation/retry journal before phase 3.

Before replacing an indexed path, account for file IDs, tags/albums/stacks, ratings/flags, source fingerprints, extracted generation metadata, sidecars, thumbnail/cache revisions, embeddings and search indexes. Decide whether the transformed file retains the original asset ID or becomes a related derivative, and how archives are represented, before schema work. Measure the migration/index cost; do not assume a new table is required. The archive location must be discoverable and restoreable, and cannot be an untracked hidden copy.

## UX acceptance

- Import dialog and gallery batch action share understandable choices: format, quality (only when supported), dimensions, metadata/workflow handling and original disposition. Export reuses the same encode semantics while remaining non-destructive.
- Preflight shows input count, estimated output size/savings **as an estimate**, unsupported files, disk-space needs, destination and collision policy. A representative before/after preview exposes visual loss, transparency changes and metadata effects.
- Progress and final results show succeeded, failed, skipped and canceled counts, per-file errors and where originals/derivatives ended up. Closing the UI must not orphan an active job.
- The default original disposition is **keep**. Archive and move-to-Trash require explicit selection; no permanent-delete option in this batch workflow. Deletion never occurs merely because an encode command returned success.
- All new controls and error text are localized in all supported UI locales and remain keyboard accessible.

## Proposed interface contract (not callable yet)

Keep `export_files_batch` and its existing DTO stable until a coordinated versioned change. The following names and fields are planning sketches, **not registered Tauri commands**. The lead must approve exact serialization, identity, authorization, progress and persistence semantics in [API_CONTRACTS.md](API_CONTRACTS.md) and [IPC_REFERENCE.md](IPC_REFERENCE.md) before implementation.

```text
TransformSpec {
  format: original | jpeg | webp | png,
  quality?: 1..100,              // only for a codec with an effective lossy quality control
  max_edge?: positive integer,
  metadata_policy: keep_supported | strip_ai | strip_all,
  collision_policy: skip | rename, // never implicit overwrite
}
ImportTransformRequest { managed_destination_id, source_selection, spec, source_disposition: keep }
LibraryTransformRequest { file_ids, spec, original_disposition: keep | archive | trash }
TransformJobReceipt {
  job_id, phase, total, succeeded, failed, skipped, canceled,
  items: [{ source_id_or_path, output_id_or_path?, status, error_code?, original_action? }]
}
```

The source selector must be a validated backend-managed selection, not an arbitrary frontend path for deletion. Any archive/Trash operation needs a reviewed preview or equivalent explicit authorization tied to the exact items. Job events must carry `job_id` and sequence/generation information so concurrent runs cannot mix progress. Responses need stable error codes and bounded, non-sensitive context; avoid parsing human-readable strings. Retain receipts long enough for restart recovery and support a status query independent of an open modal. Define whether a new request is idempotent and how retries recognize already-published outputs.

## Delivery packages and verification

| Package | Primary owner | Deliverable and acceptance gate |
| --- | --- | --- |
| [T0: export contract/correctness](https://github.com/BerryUIKI/Omera/issues/158) | Lead for filesystem/privacy contract; engineer for bounded implementation | Codec/extension agreement, effective quality, non-overwrite output, truthful metadata policy, correct ZIP/sidecar errors; regression fixtures for PNG/JPEG/WebP and interruption |
| T1: export experience | General UI engineer | Preview, estimate, collision selection, localized progress/results; manual directory/ZIP and keyboard matrix |
| [T2: managed import transform](https://github.com/BerryUIKI/Omera/issues/159) | Lead for catalog/filesystem contract; engineer for UI after approval | Verified staging, unchanged source by default, indexing of final image, retry/cancel and insufficient-space cases |
| [T3: existing-library batch](https://github.com/BerryUIKI/Omera/issues/160) | Lead for asset identity, journal, archive/Trash and schema decision; engineer for UI after approval | Crash/restart recovery, preserved relations, no stale thumbnails/embeddings, archive restore and partial-failure evidence |
| T4: advanced controls | General engineer after T0–T3 | Bounded target-size search and optional scaling/alignment with measured runtime/quality tradeoffs |

Use separate PRs targeting `dev`, with one bounded package per PR. Phase gates: identity/migration stability and safe file-operation contracts first; T0/T1 before T2; T2 before T3; T4 last. Required tests include codec signatures/extensions, alpha and orientation, metadata/sidecars, duplicate names in directory and ZIP, read-only/unavailable source, low disk space, cancellation at each publish stage, crash/restart and large batches under a measured memory limit. Test source preservation and actual restore from archive/Trash; a successful mock-only UI test does not qualify destructive behavior.

## Handoff prompt

```text
Work from the current approved dev commit in BerryUIKI/Omera and target dev with
a small PR. Read docs/IMAGE_TRANSFORM_PLAN.md, docs/DELIVERY_ROADMAP.md,
docs/API_CONTRACTS.md and docs/IPC_REFERENCE.md. Take only the explicitly
assigned T0/T1/T2/T3/T4 package. Revalidate the baseline defects and do not
invent a Tauri command, schema or deletion policy. Preserve existing export
IPC compatibility. The lead owns persistence, registered-root access, archive/
Trash authorization and crash recovery. For UI work, use approved typed mocks
until those contracts exist. Report exact tests, manual evidence, data-safety
cases, measured memory use where relevant, and remaining dependencies. Link the
implementation issue in the PR; do not close it until merged and validated.
```
