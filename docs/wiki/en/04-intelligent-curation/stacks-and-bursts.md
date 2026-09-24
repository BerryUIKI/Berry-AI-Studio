# Image Stacks, Bursts & Comparison

Generative AI creators frequently generate batches of 10 to 50 variations with identical or slightly modified prompts to find the single best composition. Without curation tools, this floods your library with near-duplicate drafts.

Omera solves this with **Intelligent Burst Stacking**, **Poker-Deck Cards**, **Authoritative Hero Selection**, and **Side-by-Side Comparison**.

---

## 1. How Stacking Works

An **Image Stack** is a collection of related image variations grouped together and represented on the gallery canvas by a single cover image called the **Hero**.

```mermaid
flowchart LR
    subgraph Generative Batch
        A[Variant 1 - Seed 101]
        B[Variant 2 - Seed 102 (5★ Hero)]
        C[Variant 3 - Seed 103]
        D[Variant 4 - Seed 104]
    end

    subgraph Stack Representation
        E["Poker-Deck Card [Badge: 4] (Hero: Variant 2)"]
    end

    A & B & C & D -->|Auto or Manual Grouping| E
```

### Collapsed vs. Expanded Stacks
- **Collapsed (Default)**: Renders as a single poker-deck card with subtle layered cards visible behind it and a numeric count badge (e.g. `[ 4 ]`).
- **Expanded**: Clicking the count badge expands all member variations inline into the gallery grid, allowing individual rating, inspection, or deletion.

---

## 2. Automated Burst Stacking (`auto_stack_images`)

Omera can automatically detect and group sequential generation bursts in the background:

### Clustering Criteria:
1. **Prompt Token Similarity**: Computes tokenized Jaccard similarity across positive prompts. You can configure the required threshold in **Settings > Stacks & Bursts** (default: `0.85` / 85% match).
2. **Time Window Proximity**: Generative bursts occur in quick succession. Omera clusters variations created within a configurable time window (default: `180 minutes`).
3. **Execution**: You can run auto-stacking on demand via **Tools > Organize Library by Prompt (All Folders or Current Folder)**, or let Omera cluster images automatically during ingestion.

---

## 3. Manual Stacking Operations

You can create, dissolve, and adjust stacks using keyboard shortcuts:

| Action | Shortcut | Description |
| :--- | :--- | :--- |
| **Group into Stack** | `Ctrl + G` / `Cmd + G` | Combines all selected standalone images or stacks into a single stack. |
| **Ungroup / Dissolve** | `Ctrl + Shift + G` / `Cmd + Shift + G` | Dissolves the selected stack back into individual standalone cards. |
| **Designate Hero Cover** | `Alt + S` / `Option + S` | Designates the active image as the primary cover card (`stack_order = 0`). |

### Stack Merge Safety & Flattening
In Omera, stacks **cannot be nested** (you cannot put a stack inside a stack). When you select multiple stacks and press `Ctrl + G`:
- Omera automatically flattens all source stacks into the target stack.
- A confirmation dialog (`StackMergeWarningModal.vue`) appears to prevent accidental grouping.
- You can check *"Do not show again"* to suppress future warnings (can be reset in **Settings > Stacks & Bursts > Reset Suppressed Warnings**).

---

## 4. Gesture Arbitration: Single vs Double Click

To ensure smooth gallery interaction without gesture conflicts:
- **Single Click on Count Badge**: Expands or collapses the stack inline.
- **Single Click on Card Body**: Selects the stack (deferred 240 ms timer).
- **Double Click on Card Body**: Cancels the expand timer immediately and opens the **Hero** image in fullscreen **Quick Look Lightbox**.

---

## 5. Side-by-Side Synchronized Compare (`CompareModal.vue`)

When deciding between subtle differences (e.g. eye rendering, hand anatomy, or lighting):
1. Select two images in the gallery.
2. Press `C` (or click **Compare**).
3. The **Side-by-Side Compare Modal** opens:
   - **Slot A (Left)** and **Slot B (Right)** display both images side-by-side.
   - **Synchronized Zoom & Pan**: Dragging or mouse-wheel zooming on either viewport moves both images in lockstep, making 100% pixel comparisons effortless.
   - **HUD Metadata Comparison**: Displays differences in Seed, Model, Steps, and CFG scale.
   - **"★ Hero Cover" Button**: Click to designate the winning image as the cover of its burst stack.

---

## 6. Batch Cull Drafts Tool (`CullDraftsModal.vue`)

Once you've picked your winning Hero image and rated 3★+ favorites:
1. Select the stack and click **"Cull Drafts"** on the floating Batch Action Bar.
2. The Cull Drafts modal displays all non-hero candidate files rated below your threshold.
3. Click **"Trash Drafts"** to send unwanted variations to your OS Recycle Bin/Trash in a single click, recovering disk space while preserving your top-tier artworks.
