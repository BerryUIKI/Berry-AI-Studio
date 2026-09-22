# Search Syntax & Visual Filtering

Berry AI Studio features a dual-mode search system: **Structured Syntax Search** for precise technical filtering, and **AI Semantic Search** for natural-language conceptual queries.

---

## 1. Syntax Search & Query Language

In the top search bar (`/` or `Ctrl + F`), you can type free-form keywords or compose structured key-value queries:

### Basic Free-Text Search
- Typing words without prefixes matches filenames, relative paths, positive prompts, negative prompts, and model names:
  ```
  cyberpunk neon rain
  ```
- Use double quotes for exact phrase matching:
  ```
  "cyberpunk street" "rainy reflections"
  ```

---

## 2. Key-Value Syntax Reference

Berry parses search tokens into structured `SearchCriteria` in native Rust, querying SQLite indexes with sub-millisecond execution:

| Token Key | Syntax Example | Description |
| :--- | :--- | :--- |
| `prompt:` | `prompt:"masterpiece, 1girl"` | Match words in positive generation prompt. |
| `neg:` | `neg:"bad hands, blurry"` | Match words in negative prompt. |
| `model:` | `model:"animagine_xl"` | Filter by model checkpoint name. |
| `hash:` | `hash:31e35c80` | Match short hash or full SHA256 model hash. |
| `sampler:` | `sampler:"Euler a"` | Filter by sampler algorithm. |
| `steps:` | `steps:30`, `steps:20..40`, `steps:>=25` | Match exact step counts or numeric ranges. |
| `cfg:` | `cfg:7`, `cfg:>=7.5`, `cfg:5..10` | Match CFG guidance scale or ranges. |
| `seed:` | `seed:12345678` | Search for a specific generation seed. |
| `rating:` | `rating:5`, `rating:>=4`, `rating:1..3` | Filter by user star rating. |
| `aesthetic:`| `aesthetic:>=7.0` | Filter by aesthetic prediction score. |
| `fav:` | `fav:true`, `fav:false` | Filter by bookmarked favorite status. |
| `is:` | `is:nsfw`, `is:sfw` | Filter by sensitive content flag. |
| `type:` | `type:image`, `type:video` | Filter by media container type. |
| `duration:` | `duration:>=5`, `duration:5..30` | Filter video duration in seconds. |
| `fps:` | `fps:>=30`, `fps:24..60` | Filter video framerate. |

### Numeric Range Syntax
- **Between Range (`min..max`)**: `steps:20..35` (between 20 and 35 steps inclusive).
- **Greater Than or Equal (`>=`)**: `cfg:>=7.0` (guidance scale 7.0 or higher).
- **Less Than or Equal (`<=`)**: `rating:<=2` (2 stars or lower).
- **Exact Match**: `rating:5` (exactly 5 stars).

---

## 3. The Visual Filter Drawer (`FilterDrawer.vue`)

If you prefer a graphical interface over typing query syntax, click the **Filter Drawer button (`☰ Filter`)** on the right side of the search bar to slide out the visual filter panel:

```
┌────────────────────────────────────────────────────────┐
│ Filter Library                         [Reset All] [✕] │
├────────────────────────────────────────────────────────┤
│ Checkpoint Model                                       │
│ [ All Models ▾                                       ] │
│                                                        │
│ Sampler Algorithm                                      │
│ [ All Samplers ▾                                     ] │
│                                                        │
│ Star Rating                                            │
│ [ ★★★★★ (5 Stars Only) ▾                             ] │
│                                                        │
│ Step Count Range                                       │
│ Min: [ 20 ] ─────────────●──────────── Max: [ 50 ]     │
│                                                        │
│ CFG Guidance Scale                                     │
│ Min: [ 5.0 ] ────────────●──────────── Max: [ 12.0 ]   │
│                                                        │
│ Media Type                                             │
│ (●) All Media    ( ) Images Only    ( ) Videos Only   │
│                                                        │
│ Content Flags                                          │
│ [✓] Favorites Only    [ ] NSFW Sensitive Only          │
└────────────────────────────────────────────────────────┘
```

The filter drawer features an active filter badge counter so you can see at a glance how many criteria are currently narrowing your results.

---

## 4. Gallery Sorting Options (`SortBar.vue`)

To the right of the gallery canvas header, you can sort your active results:

### Sort Fields:
- **Date Modified (`modified_at`)**: Chronological ordering based on file timestamp.
- **File Name / Path (`path`)**: Alphabetical ordering by filesystem path.
- **File Size (`size_bytes`)**: Order by storage footprint.
- **Star Rating (`rating`)**: Order by curated star scores.
- **Aesthetic Score (`aesthetic_score`)**: Order by neural aesthetic prediction score.

### Sort Direction:
- Click the direction button to toggle between **Descending (`↓`)** (highest / newest first) and **Ascending (`↑`)** (lowest / oldest first).
