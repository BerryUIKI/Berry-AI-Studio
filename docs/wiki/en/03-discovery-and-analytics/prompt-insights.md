# Prompt Analytics & Insights

As your asset collection grows, understanding which prompt keywords, artists, and technical parameters produce your highest-rated artworks becomes essential. Berry AI Studio provides a dedicated analytics modal (`PromptStatsModal.vue`) that aggregates metadata across your entire library.

---

## 1. Opening Prompt Keyword Insights

To launch the analytics dashboard:
- Click **Tools > Prompt Keyword Insights** from the top menu bar, or
- Click the **Insights** button in the quick-tools footer of the left navigation sidebar.

---

## 2. Analytics Tabs & Visual Metrics

```
┌────────────────────────────────────────────────────────────────────────┐
│ Prompt Insights & Keyword Analytics                                [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [ Positive Tokens ]  [ Negative Tokens ]  [ Models ]  [ Samplers ]     │
├────────────────────────────────────────────────────────────────────────┤
│ Ranked by: (●) Frequency    ( ) Average Rating                         │
├────────────────────────────────────────────────────────────────────────┤
│ Rank │ Token / Keyword              │ Occurrences │ Avg Rating │ Action│
├──────┼──────────────────────────────┼─────────────┼────────────┼───────┤
│ #1   │ masterpiece                  │ 4,120       │ ★ 4.2      │ [🔍]  │
│ #2   │ cinematic lighting           │ 2,845       │ ★ 4.7      │ [🔍]  │
│ #3   │ 1girl                        │ 2,410       │ ★ 3.9      │ [🔍]  │
│ #4   │ cyberpunk city               │ 1,890       │ ★ 4.8      │ [🔍]  │
│ #5   │ volumetric fog               │ 1,230       │ ★ 4.5      │ [🔍]  │
│ #6   │ photorealistic               │ 1,115       │ ★ 3.2      │ [🔍]  │
└────────────────────────────────────────────────────────────────────────┘
```

The dashboard provides four distinct analytical views:

### 1. Positive Prompt Tokens
- Evaluates individual keyword chips across all positive generation prompts.
- Displays total occurrence counts alongside the **Average Star Rating** of images using that token.
- Highlights your "secret sauce" keywords—tokens that consistently correlate with 4★ and 5★ ratings.

### 2. Negative Prompt Tokens
- Analyzes which negative prompt phrases appear most frequently across your workflow.
- Useful for spotting bloated or redundant negative embeddings and keywords that do not improve output quality.

### 3. Top Checkpoint Models
- Ranks every model checkpoint in your library by total generations and user ratings.
- Helps identify which fine-tuned models deliver your best results.

### 4. Samplers & Schedulers
- Ranks sampling algorithms (e.g. `DPM++ 2M Karras`, `Euler a`, `UniPC`) by frequency and aesthetic rating.

---

## 3. Interactive Search Integration

Every keyword row in the analytics table includes an **Action (`🔍`)** button:
- Clicking the search icon immediately closes the analytics modal, pastes the selected token into the main gallery search bar (`prompt:"..."`), and filters your canvas to all artworks containing that keyword.
- This allows you to jump directly from high-level statistics to reviewing specific image sets.
