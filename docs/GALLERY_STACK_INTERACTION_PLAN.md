# Gallery and Stack Interaction Optimization Plan

## Product Position

This plan translates the requested behaviors into a consistent desktop-gallery model.
Where a literal interpretation would create conflicting gestures or unnecessary work,
the implementation favors predictability, performance, and recoverability.

## Product Decisions

### Stack click versus double-click

A stack cover has two intentional gestures:

- **Single-click:** toggle that stack between collapsed and expanded.
- **Double-click:** open the cover image in the existing zoom/lightbox view.

Browsers emit click events before a double-click event, so toggling immediately would
collapse and re-expand the stack before zooming. The cover click will therefore use a
short deferred action. A second click within the double-click interval cancels the
pending toggle and activates zoom. The delay applies only to stack covers; ordinary
images retain immediate selection and double-click activation.

Expanded member images remain ordinary selectable images. Only the stack cover or the
count control collapses a stack. Making every member collapse the group would prevent
rating, comparing, or choosing a new cover.

### Expansion policy

- All stacks start collapsed when the application launches.
- By default, expanding one stack collapses the previously open stack.
- A new **Allow multiple stacks to be open simultaneously** preference disables that
  automatic collapse behavior.
- The preference is persisted in `config.json` and defaults to `false` for existing and
  new users.
- Changing folders, searches, or major gallery contexts collapses all stacks to avoid
  carrying hidden spatial state into a different result set.

### Stack cover presentation

The existing hero image remains the stack cover. A separate cover image is not needed:
it would add another asset lifecycle, cache entry, and decoding path without improving
the user's ability to identify the stack.

- Collapsed covers hide the source filename and display the localized generic label
  **Image Stack** with the member count.
- Expanded covers and members show their real filenames because they are once again
  individual assets.
- Cover selection remains available through the existing hero command.

### Waterfall view

Waterfall is a third gallery mode alongside Grid and Table. It is a virtualized masonry
layout, not an unbounded CSS-column list.

- Images use metadata dimensions to preserve their original aspect ratios.
- Thumbnails use `object-fit: contain`; no visual content is cropped.
- Items are assigned to the currently shortest column for balanced layout.
- Only cards intersecting the viewport plus overscan are mounted.
- Files without usable dimensions use a square fallback.
- Existing selection, stacking, badges, drag behavior, NSFW treatment, and thumbnail
  caching remain shared with Grid.
- Keyboard navigation follows logical result order; scrolling uses the computed masonry
  position of the selected item.

## Settings Information Architecture

The current Settings dialog is functional but visually dense and mixes unrelated
preferences. The redesign will use:

- clearer category labels: **General**, **Gallery**, **Stacks**, **Metadata**, and
  **Storage & About**;
- a short purpose statement at the top of each category;
- grouped setting cards with consistent label, description, and control alignment;
- explicit advanced labels for automatic stacking thresholds;
- immediate-action controls, such as warning reset and cache clearing, visually
  separated from saved preferences;
- a responsive layout that becomes a horizontal tab strip on narrow windows;
- a sticky footer with a clear primary Save action and non-destructive Cancel action;
- visible focus states and disabled-state explanations.

## Performance Constraints

- Do not mount every waterfall card for large libraries.
- Do not decode a second image for stack decoration or create a separate cover asset.
- Do not reload the active query when a stack expands or collapses.
- Do not clear resolved thumbnail URLs when switching between Grid and Waterfall.
- Recompute masonry geometry only when files, card width, gap, or container width
  changes—not on scroll.
- Scroll handling must only update the visible slice.

## Roadmap

### Phase 1 — Documentation and interaction contract

- [x] Define click/double-click arbitration.
- [x] Define single-open and multi-open expansion behavior.
- [x] Decide against a dedicated cover asset and document the performance rationale.
- [x] Define a virtualized waterfall algorithm and Settings information architecture.

### Phase 2 — Expansion state and gesture handling

- [x] Add the backward-compatible multi-open preference.
- [x] Make stack cover single-click toggle in both directions.
- [x] Preserve cover double-click zoom by cancelling the deferred single-click action.
- [x] Collapse other stacks before expansion unless multi-open is enabled.
- [x] Explicitly reset expansion state for application launch and new gallery contexts.

### Phase 3 — Stack cover content refinement

- [x] Hide filenames on collapsed covers.
- [x] Add a localized generic stack label and member count.
- [x] Retain the hero image as the zero-cost cover source.

### Phase 4 — Virtualized waterfall layout

- [x] Add `masonry` to the persisted gallery view mode.
- [x] Compute shortest-column positions from original aspect ratios.
- [x] Virtualize cards by vertical intersection with overscan.
- [x] Preserve full images with non-cropping thumbnail presentation.
- [x] Integrate the view toggle, zoom control, selection, stack gestures, and keyboard
  scrolling.

### Phase 5 — Settings redesign

- [x] Reorganize settings into task-oriented categories and grouped sections.
- [x] Add Grid, Waterfall, and Table default-view choices.
- [x] Add the multi-open stack preference with clear default behavior.
- [x] Improve responsive layout, focus visibility, action hierarchy, and immediate-action
  feedback.

### Phase 6 — Verification and close-out

- [x] Run frontend build, Rust tests, formatting, and Clippy.
- [ ] Visually verify stack gestures, cover labels, all gallery modes, and Settings.
- [ ] Test narrow-window and reduced-motion behavior.
- [ ] Record outcomes and any deferred large-library profiling.

## Acceptance Criteria

1. One click on a collapsed cover expands it; one click on its expanded cover collapses
   it.
2. Double-clicking either a collapsed or expanded cover opens zoom without leaving the
   stack in the wrong state.
3. A fresh application session begins with no expanded stacks.
4. With the default preference, expanding Stack B collapses Stack A.
5. With multi-open enabled, Stack A remains expanded when Stack B opens.
6. Collapsed covers never expose source filenames and do not require an additional
   cover asset.
7. Waterfall cards display the full image at its original aspect ratio.
8. Waterfall mounts only the visible window plus overscan for large result sets.
9. Grid, Waterfall, and Table can each be saved as the default view.
10. Settings remains usable at narrow desktop widths and communicates which actions are
    immediate versus saved.

## Progress Log

- **2026-09-16 — Plan complete:** translated the feature requests into a non-conflicting
  gesture model, single-open default, zero-extra-asset cover strategy, virtualized
  waterfall design, and task-oriented Settings redesign.
- **2026-09-16 — Phase 2 implemented:** stack covers now arbitrate deferred single-click
  toggles against double-click zoom, new contexts explicitly reset expansion, and the
  persisted multi-open preference controls whether opening a stack collapses its peer.
- **2026-09-16 — Phase 3 implemented:** collapsed covers retain the cached hero
  thumbnail but replace filename/path presentation with a localized Image Stack label,
  member count, and neutral STACK type marker.
- **2026-09-16 — Phase 4 implemented:** Grid and Waterfall now share one virtualized
  card pipeline. Waterfall computes stable shortest-column geometry from metadata,
  mounts only viewport-adjacent cards, preserves original aspect ratios without
  cropping, and remains compatible with selection, stack gestures, zoom sizing,
  thumbnail prefetch, drag behavior, and keyboard scrolling.
- **2026-09-16 — Phase 5 implemented:** Settings now uses task-oriented General,
  Gallery, Stacks, Metadata, and Storage categories; concise purpose text; accessible
  tab semantics and focus states; switch controls; distinct immediate actions; a
  stronger Save hierarchy; and a narrow-window horizontal navigation layout. Default
  view choices include Grid, Waterfall, and Table, and the multi-open stack preference
  is persisted with the other stack controls.
- **2026-09-16 — Automated verification complete:** the production frontend build,
  137 Rust unit tests plus doc tests, Rust formatting check, and warning-denied Clippy
  check all pass. Per the user's preference, final visual and hands-on interaction
  acceptance remains for user review rather than an additional automated UI pass.
