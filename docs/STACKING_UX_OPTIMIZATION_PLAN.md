# Image Stacking Performance and UX Optimization Plan

## Purpose

This document defines the implementation plan for improving manual image stacking,
range selection, and the collapsed stack presentation. It is the source of truth for
the work and will be updated after each implementation phase.

## Findings

### 1. Ctrl+G causes the gallery to flicker

`onStackSelected` sends the stack mutation to the backend and then calls the general
`loadFiles` path. That path enables the gallery-wide loading state, queries the full
active result set, queries stack summaries, filters collapsed members, and starts a
new thumbnail prefetch. `VirtualGrid` replaces the gallery with a loading placeholder
for the duration, so every visible card disappears even though only the selected
records changed.

The grid also clears its component-local thumbnail map whenever the `files` array is
replaced. This compounds the perceived delay by making stable, visible cards resolve
their thumbnails again.

### 2. Shift-click has no stable anchor after a normal click

The range-selection code derives its anchor from the last member of
`selectedFilePaths`. A normal click only updates `selectedFile`, so the multi-selection
set remains empty. Consequently, selecting Image A normally and Shift-clicking Image B
does not form a range on the first attempt.

### 3. A collapsed stack is not visually or behaviorally clear

The current presentation adds two offset shadows and an emoji count badge to an
otherwise normal card. It does not convincingly communicate a pile of images. The
badge can expand a stack, but clicking the stack card itself only selects it and
double-clicking opens the lightbox. This conflicts with the desired interaction that a
collapsed stack expands only when the user clicks it.

## Target Interaction Model

### Selection

- A plain click selects one image and establishes it as the range anchor.
- Shift-click selects the inclusive range from the anchor to the clicked image on the
  first attempt.
- Ctrl/Cmd-click toggles an image without discarding the existing selection and makes
  that image the new range anchor.
- A new plain click clears an existing multi-selection before selecting the new anchor.
- Shift-click preserves selections outside the new range, matching common desktop file
  managers.
- Selection state must use immutable `Set` replacement so Vue updates badges and card
  states deterministically.

### Stack mutation

- Ctrl/Cmd+G updates the current result set and stack summary locally after the backend
  succeeds.
- The gallery remains mounted, retains scroll position, and keeps already resolved
  thumbnails visible.
- The first selected item in visual order becomes the cover, matching backend ordering.
- A background/full reload is reserved for recovery when the optimistic local update
  cannot be applied safely.

### Stack card

- A collapsed stack is a single cover card with two visible, slightly rotated backing
  layers and a concise numeric count badge.
- Clicking anywhere on a collapsed stack expands it inline. It does not open the
  lightbox on that click.
- Expanded members render as ordinary cards with a restrained shared-stack indicator.
- Clicking the count badge has the same expand/collapse behavior and remains keyboard
  accessible.
- Double-click activation remains available for normal cards and expanded stack
  members.
- Motion respects `prefers-reduced-motion`.

## Performance Constraints

- Do not set the gallery-wide loading state for stack, unstack, hero-cover, or
  expand/collapse mutations.
- Do not re-query the complete active library after a successful manual stack action.
- Do not clear the thumbnail cache merely because the `files` array identity changes.
- Keep virtual-grid item keys stable across selection and stack-state changes.
- Avoid layout animation of the full grid; animate only stack-card decoration and
  interaction affordances.

## Implementation Roadmap

### Phase 1 — Documentation and baseline

- [x] Trace the keyboard shortcut, selection state, data reload, virtualization, and
  stack-card rendering paths.
- [x] Record root causes, target behavior, constraints, and acceptance criteria.
- [x] Capture the baseline with the production frontend build.

### Phase 2 — Selection logic refinement

- [x] Add an explicit selection anchor independent of the multi-selection set.
- [x] Make plain-click, modifier-click, and Shift-click transitions deterministic.
- [x] Apply the behavior consistently to grid and table views through the shared App
  handler.
- [x] Update shortcut/user documentation with the refined range behavior.

### Phase 3 — Non-blocking stack mutations

- [x] Return/use the stack identifier from the existing stack command.
- [x] Patch affected files and stack summaries locally after Ctrl/Cmd+G.
- [x] Preserve the current gallery, scroll position, and thumbnail cache.
- [x] Make expand/collapse a local projection instead of a loading-state reload.
- [x] Retain a safe reload fallback for unexpected or stale data.

### Phase 4 — Stack-card redesign

- [x] Replace the emoji-led decoration with layered image-card surfaces.
- [x] Make a collapsed stack card expand on click.
- [x] Keep collapse, compare, selection, and image activation controls unambiguous.
- [x] Verify light/dark theme styles, narrow cards, keyboard focus, and reduced motion
  in the implementation.

### Phase 5 — Verification and documentation close-out

- [x] Run the TypeScript check and production build.
- [x] Confirm that Rust tests are not required because no backend code changed.
- [x] Verify collapsed/expanded stack rendering and click behavior in a browser QA
  harness using the production component.
- [x] Record implementation outcomes and deferred full-library performance profiling in
  this document.

## Acceptance Criteria

1. With two or more images selected, Ctrl/Cmd+G never replaces the gallery with an
   empty/loading view.
2. Stable visible cards do not lose their resolved thumbnails during stack creation.
3. Plain-click Image A followed by Shift-click Image B selects the inclusive range in
   one attempt in both grid and table views.
4. Ctrl/Cmd-click toggling and Ctrl/Cmd+A continue to work.
5. A collapsed stack is visibly distinguishable from an ordinary image without relying
   only on color or emoji.
6. One click on a collapsed stack expands its members; ordinary image cards still
   require double-click to activate the lightbox.
7. Expanding/collapsing a stack does not show the gallery loading placeholder.
8. The production frontend build passes with no TypeScript errors.

## Progress Log

- **2026-09-16 — Audit complete:** confirmed full-reload flicker, missing selection
  anchor, mutable selection sets, thumbnail-cache invalidation on array replacement,
  and ambiguous collapsed-stack interaction. Created the phased remediation plan.
- **2026-09-16 — Baseline verified:** `pnpm build` passes before implementation.
- **2026-09-16 — Phase 2 implemented:** selection now has an explicit anchor; the
  first Shift-click selects the full inclusive range, modifier-click uses immutable
  state updates, and grid/table behavior is shared.
- **2026-09-16 — Phase 3 implemented:** manual stack creation patches the active
  gallery in place; expand/collapse fetches only stack members and never enters the
  gallery loading state; unstack and hero changes also use targeted updates; resolved
  thumbnail URLs survive `files` array replacement.
- **2026-09-16 — Phase 4 implemented:** collapsed stacks now use two card-shaped
  backing layers and a compact CSS stack/count badge. A single unmodified click expands
  the stack, double-click activation is suppressed during expansion, expanded members
  use restrained styling, and motion can be disabled through the OS preference.
- **2026-09-16 — Visual QA complete:** rendered the real `VirtualGrid` component with
  a four-image sample stack. Verified layered collapsed styling, single-click expansion,
  cover-only collapse control, accessible expanded state, and collapse without a
  loading placeholder. Removed the temporary harness after verification.

## Outcome and Follow-up

All three reported issues are addressed in the frontend. The stack mutation path now
scales with the number of selected/stacked members instead of reloading the entire
active query, apart from an explicit recovery fallback when returned data is
inconsistent. A future profiling pass with a large real library should capture
interaction latency at several collection sizes; that measurement is useful release
evidence but is not required for the logic correction.

## Stack Merge Safety Extension

### Problem Statement

The storage model assigns a single `stack_id` directly to each image, so it does not
support nested stack entities. However, invoking Ctrl/Cmd+G with selected stack covers
currently treats those covers as ordinary images. This can detach a cover from its
existing members and leave behind a partial stack, which behaves like an accidental
and ambiguous nested-stack operation from the user's perspective.

### Merge Invariant

- A stack can contain images only; it can never contain another stack.
- Every selected item with a `stack_id` represents its complete stack, whether the
  selected item is the collapsed cover or an expanded member.
- The first selected stack in current visual order is the target stack.
- The target stack keeps its identifier, current cover, and existing member order.
- Other selected stacks are flattened into their image members and dissolved as part
  of the same database transaction.
- Selected standalone images are appended after the target stack members.
- Duplicate images and repeated selections from the same expanded stack are removed.
- Selecting only members of one existing stack is a no-op.

### Warning Interaction

- If Ctrl/Cmd+G would combine multiple stacks or add standalone images to a stack,
  show a dedicated warning dialog before mutation.
- Explain which stack remains the target and that all source stacks will be flattened
  into it.
- Provide **Cancel** and **Merge into Stack** actions.
- Include a **Do not show this warning again** checkbox.
- Persist suppression only when the user confirms the merge; cancelling never changes
  preferences.
- Suppression uses a stable warning identifier in the application configuration rather
  than a one-off boolean, allowing future suppressible warnings to share the system.
- Settings provides **Reset Suppressed Warnings**, which clears every stored warning
  identifier and reports the result without changing unrelated preferences.

### Extension Roadmap

#### Phase 6 — Documentation and merge-domain contract

- [x] Document flat merge semantics, target selection, ordering, deduplication, and
  warning behavior before implementation.
- [x] Add a transactional storage operation that flattens source stacks into a target
  stack without leaving partial source stacks.
- [x] Add storage tests for stack+image, stack+stack, duplicate-member, and target-cover
  preservation cases.

#### Phase 7 — Warning preference infrastructure

- [x] Add backward-compatible `suppressed_warnings` configuration storage.
- [x] Add reusable helpers for checking, suppressing, and resetting named warnings.
- [x] Add the Settings reset control with success/error feedback.

#### Phase 8 — Merge warning and Ctrl/Cmd+G integration

- [x] Resolve the selected visual items into a deterministic flat merge request.
- [x] Show an accessible warning dialog only for stack-affecting merges.
- [x] Apply the successful merge locally without a gallery-wide reload.
- [x] Update stack summaries, expanded state, selection, and thumbnail-preserving view
  state consistently.

#### Phase 9 — Verification and close-out

- [x] Run frontend type-check/build and relevant Rust tests.
- [x] Verify dialog keyboard behavior, suppression, reset, and merge scenarios.
- [x] Update this document with final outcomes and any deferred profiling work.

### Extension Acceptance Criteria

1. Ctrl/Cmd+G with standalone images retains the existing direct-stack behavior and
   does not show the merge warning.
2. A selected stack plus standalone images shows the warning unless it is suppressed.
3. Two or more selected stacks show the warning unless it is suppressed.
4. Cancelling leaves database, gallery, selection, and warning preferences unchanged.
5. Confirming produces exactly one flat stack containing every unique member.
6. The first selected stack keeps its stack ID and cover image.
7. No source stack remains partially populated after a successful merge.
8. Choosing not to show the warning again persists across application restarts.
9. Resetting suppressed warnings in Settings causes the merge warning to appear again.
10. Stack merging never enters the gallery-wide loading state or clears stable
    thumbnails.

### Extension Progress Log

- **2026-09-16 — Extension audit complete:** confirmed the existing command can
  reassign a selected cover without including its hidden members. Defined a
  transactionally flat merge contract and a reusable warning-suppression model.
- **2026-09-16 — Phase 6 implemented:** added a transactional flat-merge storage
  operation and Tauri command. Tests cover stack-to-stack merging, standalone appends,
  duplicate inputs, target hero preservation, source dissolution, and stale stacked-ID
  rejection with full rollback.
- **2026-09-16 — Phase 7 implemented:** added a backward-compatible named warning
  suppression list, typed persistence helpers, and a Settings control that resets all
  suppressed warnings without changing other preferences.
- **2026-09-16 — Phase 8 implemented:** Ctrl/Cmd+G now separates ordinary grouping
  from stack-affecting merges, uses the first selected stack in visual order as the
  target, presents a localized accessible warning, optionally persists suppression on
  confirmation, and reconciles the merged cover locally without a full gallery reload.
- **2026-09-16 — Phase 9 verified:** production rendering confirmed safe Cancel focus,
  Escape cancellation, dynamic merge counts, suppression selection, and confirmation.
  Automated coverage confirms transactional flattening/rollback and legacy config
  compatibility; frontend and Rust verification commands pass.

### Extension Outcome

Stack nesting is now prevented by construction. Stack-affecting Ctrl/Cmd+G operations
are explicit flat merges, execute transactionally, preserve the target stack identity
and cover, and retain the non-blocking gallery behavior. Warning suppression is named,
persistent, localized, and reversible from Settings. Future suppressible warnings can
reuse the same configuration mechanism without adding one-off preference fields.
