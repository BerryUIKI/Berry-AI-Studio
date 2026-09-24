# Keyboard Shortcuts Cheatsheet

Omera is designed with a **keyboard-first workflow**. You can browse, rate, group, inspect, and organize massive libraries without ever touching the mouse.

---

## 1. Quick Look & Gallery Browsing

| Shortcut | Action | Scope | Description |
| :--- | :--- | :--- | :--- |
| `Space` or `Enter` | **Open Lightbox** | Gallery selection | Open high-resolution Quick Look modal for selected item. |
| `Esc` | **Dismiss / Clear** | Global | Close Lightbox or modal, cancel search, or clear card selection. |
| `←` / `→` | **Navigate Previous / Next** | Gallery & Lightbox | Move selection to adjacent item (or step 1 frame in video playback). |
| `↑` / `↓` | **Navigate Row Up / Down** | Grid View | Move selection up or down by one visual row. |
| `Ctrl + A` / `Cmd + A` | **Select All** | Gallery Canvas | Select all cards currently matching the active search/filter. |
| `Delete` or `Backspace` | **Move to Trash** | Gallery selection | Send selected item(s) to the OS Recycle Bin/Trash. |

---

## 2. Fast Curation & Rating

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `1` | **Rate 1 Star** | Assign 1-star rating to selected item(s). |
| `2` | **Rate 2 Stars** | Assign 2-star rating to selected item(s). |
| `3` | **Rate 3 Stars** | Assign 3-star rating to selected item(s). |
| `4` | **Rate 4 Stars** | Assign 4-star rating to selected item(s). |
| `5` | **Rate 5 Stars** | Assign 5-star rating to selected item(s). |
| `0` | **Clear Rating** | Remove star rating from selected item(s). |
| `F` | **Toggle Favorite** | Bookmark or unbookmark selected item(s). |

> [!TIP]
> **Blind Rating**: You can keep one hand on the arrow keys (`←`, `→`) to flip through images while pressing `1`–`5` with your other hand. Ratings are written immediately to SQLite without interrupting playback.

---

## 3. Stacking & Comparison

| Shortcut | Action | Scope | Description |
| :--- | :--- | :--- | :--- |
| `Ctrl + G` / `Cmd + G` | **Group into Stack** | Gallery selection | Group selected items into a poker-deck burst stack. |
| `Ctrl + Shift + G` / `Cmd + Shift + G` | **Ungroup Stack** | Gallery selection | Dissolve selected stack back into standalone images. |
| `Alt + S` / `Option + S` | **Set Cover Hero** | Stack selection | Designate active image as the primary cover card of its stack. |
| `C` | **Side-by-Side Compare** | Gallery selection | Open split-screen 1-to-1 comparison view with synced zoom/pan. |

---

## 4. Workspace Panels & Zoom

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `B` | **Toggle Sidebar** | Show or hide the left navigation panel (Library, Folders, Tags). |
| `I` | **Toggle Inspector** | Show or hide the right property inspector panel. |
| `/` or `Ctrl + F` / `Cmd + F` | **Focus Search** | Highlight the search bar and select all query text. |
| `Ctrl + =` / `Cmd + =` | **Zoom In** | Increase gallery card thumbnail size. |
| `Ctrl + -` / `Cmd + -` | **Zoom Out** | Decrease gallery card thumbnail size. |
| `Ctrl + 0` / `Cmd + 0` | **Reset Zoom** | Reset gallery card size to default standard width (256px). |
| `F11` | **Fullscreen** | Toggle borderless fullscreen window mode. |

---

## 5. Library Operations & Modals

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + O` / `Cmd + O` | **Add Folder** | Open the Add Folder Mode Wizard. |
| `Ctrl + E` / `Cmd + E` | **Batch Export** | Open the batch transcoding, privacy sanitization, and packaging modal. |
| `Ctrl + ,` / `Cmd + ,` | **Preferences** | Open the central application settings window. |
| `?` or `Shift + /` | **Shortcuts Guide** | Open the interactive keyboard shortcuts cheatsheet. |
| `Alt + F4` | **Exit App** | Cleanly exit Omera. |

---

## 6. Selection Modifiers

- **Plain Click**: Selects a single card and sets it as the **selection anchor**.
- **Shift + Click**: Extends selection from the anchor to the clicked card as an **inclusive contiguous range**.
- **Ctrl + Click** (or `Cmd + Click` on macOS): **Toggles** selection for an individual card without clearing other selected items, and updates the selection anchor.
- **Double Click on Stack Cover**: Instantly opens Lightbox zoom for the hero image (gesture arbitration avoids accidental expansion).
- **Single Click on Stack Count Badge**: Expands or collapses the burst stack inline.
