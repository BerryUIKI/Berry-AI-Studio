# Stack Collapse Regression Plan

## Report

After expanding and collapsing a stack, multiple members can remain visible as separate
collapsed covers. The screenshot shows several cards carrying the same stack count and
generic cover label instead of a single hero card.

## Root Cause Hypothesis

The UI currently treats either `stack_order === 0` or the summary's `hero_image_id` as
a hero. If legacy or inconsistent data contains more than one zero-order member, the
collapse filter keeps all of them and every retained member is rendered as a cover.

## Product Invariant

A collapsed stack must render exactly one card:

1. Prefer the authoritative `hero_image_id` from the stack summary.
2. Use `stack_order === 0` only when no summary hero is available.
3. Use the same hero predicate for initial loading, local collapse, and cover gesture
   detection so rendering and interaction cannot disagree.
4. Preserve the selected hero and remove hidden member selections during collapse.

## Implementation Roadmap

- [ ] Centralize hero detection in the gallery and card components.
- [ ] Make initial stack filtering prefer the summary hero exclusively.
- [ ] Make local collapse retain exactly the preferred hero.
- [ ] Ensure only the preferred hero responds as the stack cover.
- [ ] Add focused regression tests for duplicate zero-order data.
- [ ] Run frontend build and relevant quality gates.

## Acceptance Criteria

- Collapsing a stack with multiple `stack_order === 0` members leaves one card.
- The retained card matches `hero_image_id` when it is available.
- Repeated expand/collapse cycles remain stable.
- Double-click zoom and ordinary member selection remain unchanged.
