//! Self-contained offline HTML showcase generator.
//!
//! Generates a standalone, zero-dependency `index.html` file that showcases
//! exported images with an interactive responsive gallery, search & rating filters,
//! a full-screen lightbox with pan/zoom, and an AI generation prompt & parameter inspector.

use serde::{Deserialize, Serialize};

/// Lightweight metadata for an item exported into the HTML showcase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShowcaseItemMetadata {
    pub filename: String,
    pub width: u32,
    pub height: u32,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub model: Option<String>,
    pub sampler: Option<String>,
    pub seed: Option<String>,
    pub cfg_scale: Option<f64>,
    pub steps: Option<u32>,
    pub rating: Option<u8>,
}

/// Generates a completely self-contained offline `index.html` string.
pub fn generate_html_showcase(title: &str, items: &[ShowcaseItemMetadata]) -> String {
    let items_json = serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string());
    // Escape any potential closing script tag in the serialized JSON
    let safe_items_json = items_json.replace("</script>", "<\\/script>");

    let escaped_title = html_escape(title);
    let count = items.len();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{escaped_title} - Berry AI Studio Showcase</title>
  <style>
    :root {{
      --bg: #0f1117;
      --card-bg: #1a1d26;
      --card-border: #282c39;
      --card-hover: #262b38;
      --text: #f3f4f6;
      --text-muted: #9ca3af;
      --accent: #8b5cf6;
      --accent-hover: #7c3aed;
      --badge-bg: #2d3345;
      --badge-text: #c7d2fe;
      --star-color: #fbbf24;
    }}
    * {{
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }}
    body {{
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }}
    header {{
      background: rgba(15, 17, 23, 0.95);
      backdrop-filter: blur(12px);
      border-bottom: 1px solid var(--card-border);
      position: sticky;
      top: 0;
      z-index: 50;
      padding: 16px 24px;
    }}
    .header-content {{
      max-width: 1600px;
      margin: 0 auto;
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }}
    .title-group h1 {{
      font-size: 1.4rem;
      font-weight: 700;
      letter-spacing: -0.02em;
      color: #fff;
    }}
    .title-group p {{
      font-size: 0.85rem;
      color: var(--text-muted);
      margin-top: 2px;
    }}
    .controls {{
      display: flex;
      align-items: center;
      gap: 12px;
      flex-wrap: wrap;
    }}
    .search-box {{
      position: relative;
    }}
    .search-box input {{
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      color: var(--text);
      border-radius: 8px;
      padding: 8px 12px 8px 32px;
      font-size: 0.88rem;
      width: 240px;
      outline: none;
      transition: all 0.2s;
    }}
    .search-box input:focus {{
      border-color: var(--accent);
      box-shadow: 0 0 0 2px rgba(139, 92, 246, 0.2);
      width: 280px;
    }}
    .search-icon {{
      position: absolute;
      left: 10px;
      top: 50%;
      transform: translateY(-50%);
      font-size: 0.85rem;
      color: var(--text-muted);
      pointer-events: none;
    }}
    .filter-btn {{
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      color: var(--text-muted);
      padding: 7px 12px;
      border-radius: 8px;
      font-size: 0.85rem;
      cursor: pointer;
      transition: all 0.15s;
    }}
    .filter-btn:hover, .filter-btn.active {{
      background: var(--badge-bg);
      color: var(--text);
      border-color: var(--accent);
    }}
    main {{
      max-width: 1600px;
      margin: 0 auto;
      padding: 24px;
      flex: 1;
      width: 100%;
    }}
    .gallery-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
      gap: 20px;
    }}
    .card {{
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      cursor: pointer;
      transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.2s, border-color 0.2s;
    }}
    .card:hover {{
      transform: translateY(-4px);
      box-shadow: 0 12px 24px rgba(0, 0, 0, 0.4);
      border-color: var(--accent);
    }}
    .thumb-wrapper {{
      position: relative;
      width: 100%;
      padding-top: 100%;
      background: #141720;
      overflow: hidden;
    }}
    .thumb-wrapper img {{
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      object-fit: cover;
      transition: transform 0.3s ease;
    }}
    .card:hover .thumb-wrapper img {{
      transform: scale(1.04);
    }}
    .rating-badge {{
      position: absolute;
      top: 8px;
      right: 8px;
      background: rgba(15, 17, 23, 0.85);
      backdrop-filter: blur(4px);
      border-radius: 6px;
      padding: 3px 6px;
      font-size: 0.75rem;
      color: var(--star-color);
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 3px;
    }}
    .card-info {{
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      flex: 1;
    }}
    .card-prompt {{
      font-size: 0.8rem;
      color: #e2e8f0;
      line-height: 1.35;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      word-break: break-word;
    }}
    .card-meta {{
      margin-top: auto;
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 0.75rem;
      color: var(--text-muted);
    }}
    .model-badge {{
      background: var(--badge-bg);
      color: var(--badge-text);
      padding: 2px 6px;
      border-radius: 4px;
      max-width: 180px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }}
    /* Lightbox Modal */
    .lightbox {{
      position: fixed;
      inset: 0;
      background: rgba(10, 11, 15, 0.95);
      backdrop-filter: blur(16px);
      z-index: 100;
      display: none;
      flex-direction: column;
    }}
    .lightbox.active {{
      display: flex;
    }}
    .lightbox-header {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 12px 20px;
      border-bottom: 1px solid var(--card-border);
      background: rgba(15, 17, 23, 0.8);
    }}
    .lightbox-body {{
      flex: 1;
      display: flex;
      overflow: hidden;
      position: relative;
    }}
    .lightbox-media {{
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 24px;
      position: relative;
      overflow: hidden;
    }}
    .lightbox-img {{
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
      border-radius: 8px;
      box-shadow: 0 16px 36px rgba(0, 0, 0, 0.6);
      transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
      cursor: zoom-in;
    }}
    .lightbox-img.zoomed {{
      cursor: zoom-out;
      transform: scale(2);
    }}
    .lightbox-sidebar {{
      width: 380px;
      background: var(--card-bg);
      border-left: 1px solid var(--card-border);
      padding: 20px;
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      gap: 16px;
    }}
    .sidebar-section h3 {{
      font-size: 0.8rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--text-muted);
      margin-bottom: 8px;
    }}
    .prompt-box {{
      background: #12141a;
      border: 1px solid var(--card-border);
      border-radius: 8px;
      padding: 10px;
      font-size: 0.85rem;
      line-height: 1.45;
      color: #e2e8f0;
      white-space: pre-wrap;
      word-break: break-word;
      max-height: 200px;
      overflow-y: auto;
      position: relative;
    }}
    .copy-btn {{
      background: var(--accent);
      color: #fff;
      border: none;
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 0.8rem;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 4px;
      margin-top: 8px;
      transition: background 0.15s;
    }}
    .copy-btn:hover {{
      background: var(--accent-hover);
    }}
    .params-grid {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
    }}
    .param-chip {{
      background: #141720;
      border: 1px solid var(--card-border);
      border-radius: 6px;
      padding: 6px 10px;
      display: flex;
      flex-direction: column;
    }}
    .param-label {{
      font-size: 0.7rem;
      color: var(--text-muted);
    }}
    .param-val {{
      font-size: 0.82rem;
      color: #f1f5f9;
      font-weight: 500;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }}
    .nav-arrow {{
      position: absolute;
      top: 50%;
      transform: translateY(-50%);
      background: rgba(15, 17, 23, 0.7);
      backdrop-filter: blur(8px);
      border: 1px solid var(--card-border);
      color: #fff;
      width: 44px;
      height: 44px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      font-size: 1.2rem;
      transition: all 0.2s;
    }}
    .nav-arrow:hover {{
      background: var(--accent);
      border-color: var(--accent);
    }}
    .nav-prev {{ left: 16px; }}
    .nav-next {{ right: 16px; }}
    .close-btn {{
      background: transparent;
      border: none;
      color: var(--text-muted);
      font-size: 1.4rem;
      cursor: pointer;
      padding: 4px 8px;
      border-radius: 4px;
    }}
    .close-btn:hover {{
      color: #fff;
      background: rgba(255, 255, 255, 0.1);
    }}
    .empty-state {{
      text-align: center;
      padding: 64px 20px;
      color: var(--text-muted);
      font-size: 1rem;
    }}
    footer {{
      border-top: 1px solid var(--card-border);
      padding: 16px;
      text-align: center;
      font-size: 0.75rem;
      color: var(--text-muted);
    }}
    @media (max-width: 900px) {{
      .lightbox-body {{
        flex-direction: column;
      }}
      .lightbox-sidebar {{
        width: 100%;
        max-height: 40vh;
        border-left: none;
        border-top: 1px solid var(--card-border);
      }}
    }}
  </style>
</head>
<body>
  <header>
    <div class="header-content">
      <div class="title-group">
        <h1>{escaped_title}</h1>
        <p id="item-counter">{count} artworks</p>
      </div>
      <div class="controls">
        <div class="search-box">
          <span class="search-icon">🔍</span>
          <input type="text" id="search-input" placeholder="Filter by prompt, model...">
        </div>
        <button type="button" class="filter-btn active" data-filter="all">All</button>
        <button type="button" class="filter-btn" data-filter="5">★ 5 Stars</button>
        <button type="button" class="filter-btn" data-filter="4">★ 4+ Stars</button>
      </div>
    </div>
  </header>

  <main>
    <div class="gallery-grid" id="gallery"></div>
    <div class="empty-state" id="empty-state" style="display: none;">
      No matching artworks found.
    </div>
  </main>

  <footer>
    Generated by <strong>Berry AI Studio</strong> &bull; Showcase Album
  </footer>

  <!-- Lightbox Modal -->
  <div class="lightbox" id="lightbox">
    <div class="lightbox-header">
      <div id="lightbox-filename" style="font-size: 0.9rem; font-weight: 500;"></div>
      <button type="button" class="close-btn" id="lightbox-close">&times;</button>
    </div>
    <div class="lightbox-body">
      <div class="lightbox-media">
        <button type="button" class="nav-arrow nav-prev" id="nav-prev">&#10094;</button>
        <img class="lightbox-img" id="lightbox-img" alt="Artwork detail">
        <button type="button" class="nav-arrow nav-next" id="nav-next">&#10095;</button>
      </div>
      <div class="lightbox-sidebar">
        <div class="sidebar-section" id="section-prompt">
          <h3>Prompt</h3>
          <div class="prompt-box" id="lightbox-prompt"></div>
          <button type="button" class="copy-btn" id="copy-prompt-btn">📋 Copy Prompt</button>
        </div>
        <div class="sidebar-section" id="section-neg" style="display: none;">
          <h3>Negative Prompt</h3>
          <div class="prompt-box" id="lightbox-neg"></div>
        </div>
        <div class="sidebar-section">
          <h3>Generation Parameters</h3>
          <div class="params-grid">
            <div class="param-chip" id="chip-model">
              <span class="param-label">Model</span>
              <span class="param-val" id="val-model">-</span>
            </div>
            <div class="param-chip" id="chip-sampler">
              <span class="param-label">Sampler</span>
              <span class="param-val" id="val-sampler">-</span>
            </div>
            <div class="param-chip" id="chip-seed">
              <span class="param-label">Seed</span>
              <span class="param-val" id="val-seed">-</span>
            </div>
            <div class="param-chip" id="chip-steps">
              <span class="param-label">Steps / CFG</span>
              <span class="param-val" id="val-steps">-</span>
            </div>
            <div class="param-chip" id="chip-res">
              <span class="param-label">Dimensions</span>
              <span class="param-val" id="val-res">-</span>
            </div>
            <div class="param-chip" id="chip-rating">
              <span class="param-label">Rating</span>
              <span class="param-val" id="val-rating">-</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <script id="showcase-data" type="application/json">
{safe_items_json}
  </script>

  <script>
    (function() {{
      const rawData = document.getElementById('showcase-data').textContent;
      const allItems = JSON.parse(rawData);
      let filteredItems = allItems.slice();
      let currentIndex = 0;
      let activeFilter = 'all';
      let searchQuery = '';

      const gallery = document.getElementById('gallery');
      const emptyState = document.getElementById('empty-state');
      const itemCounter = document.getElementById('item-counter');
      const searchInput = document.getElementById('search-input');
      const filterBtns = document.querySelectorAll('.filter-btn');

      const lightbox = document.getElementById('lightbox');
      const lightboxImg = document.getElementById('lightbox-img');
      const lightboxFilename = document.getElementById('lightbox-filename');
      const lightboxPrompt = document.getElementById('lightbox-prompt');
      const lightboxNeg = document.getElementById('lightbox-neg');
      const sectionNeg = document.getElementById('section-neg');
      const valModel = document.getElementById('val-model');
      const valSampler = document.getElementById('val-sampler');
      const valSeed = document.getElementById('val-seed');
      const valSteps = document.getElementById('val-steps');
      const valRes = document.getElementById('val-res');
      const valRating = document.getElementById('val-rating');
      const copyBtn = document.getElementById('copy-prompt-btn');

      function renderGallery() {{
        gallery.innerHTML = '';
        if (filteredItems.length === 0) {{
          emptyState.style.display = 'block';
          itemCounter.textContent = '0 artworks';
          return;
        }}
        emptyState.style.display = 'none';
        itemCounter.textContent = `${{filteredItems.length}} / ${{allItems.length}} artworks`;

        filteredItems.forEach((item, idx) => {{
          const card = document.createElement('div');
          card.className = 'card';
          card.onclick = () => openLightbox(idx);

          const thumbWrapper = document.createElement('div');
          thumbWrapper.className = 'thumb-wrapper';

          const img = document.createElement('img');
          img.src = encodeURI(item.filename);
          img.loading = 'lazy';
          img.alt = item.prompt || item.filename;
          thumbWrapper.appendChild(img);

          if (item.rating && item.rating > 0) {{
            const badge = document.createElement('div');
            badge.className = 'rating-badge';
            badge.innerHTML = `★ ${{item.rating}}`;
            thumbWrapper.appendChild(badge);
          }}

          const info = document.createElement('div');
          info.className = 'card-info';

          const prompt = document.createElement('div');
          prompt.className = 'card-prompt';
          prompt.textContent = item.prompt || item.filename;
          info.appendChild(prompt);

          const meta = document.createElement('div');
          meta.className = 'card-meta';

          if (item.model) {{
            const modelBadge = document.createElement('span');
            modelBadge.className = 'model-badge';
            modelBadge.textContent = item.model;
            meta.appendChild(modelBadge);
          }} else {{
            const emptySpan = document.createElement('span');
            emptySpan.textContent = item.filename;
            meta.appendChild(emptySpan);
          }}

          if (item.width && item.height) {{
            const resSpan = document.createElement('span');
            resSpan.textContent = `${{item.width}}×${{item.height}}`;
            meta.appendChild(resSpan);
          }}

          info.appendChild(meta);
          card.appendChild(thumbWrapper);
          card.appendChild(info);
          gallery.appendChild(card);
        }});
      }}

      function applyFilters() {{
        filteredItems = allItems.filter(item => {{
          if (activeFilter === '5' && (!item.rating || item.rating < 5)) return false;
          if (activeFilter === '4' && (!item.rating || item.rating < 4)) return false;

          if (searchQuery) {{
            const q = searchQuery.toLowerCase();
            const promptMatch = item.prompt && item.prompt.toLowerCase().includes(q);
            const modelMatch = item.model && item.model.toLowerCase().includes(q);
            const filenameMatch = item.filename.toLowerCase().includes(q);
            const seedMatch = item.seed && item.seed.toLowerCase().includes(q);
            if (!promptMatch && !modelMatch && !filenameMatch && !seedMatch) return false;
          }}
          return true;
        }});
        renderGallery();
      }}

      searchInput.addEventListener('input', (e) => {{
        searchQuery = e.target.value.trim();
        applyFilters();
      }});

      filterBtns.forEach(btn => {{
        btn.addEventListener('click', () => {{
          filterBtns.forEach(b => b.classList.remove('active'));
          btn.classList.add('active');
          activeFilter = btn.dataset.filter;
          applyFilters();
        }});
      }});

      function openLightbox(idx) {{
        currentIndex = idx;
        const item = filteredItems[idx];
        if (!item) return;

        lightboxImg.src = encodeURI(item.filename);
        lightboxImg.classList.remove('zoomed');
        lightboxFilename.textContent = item.filename;

        lightboxPrompt.textContent = item.prompt || '(No prompt available)';
        if (item.negative_prompt) {{
          sectionNeg.style.display = 'block';
          lightboxNeg.textContent = item.negative_prompt;
        }} else {{
          sectionNeg.style.display = 'none';
        }}

        valModel.textContent = item.model || 'Unknown';
        valSampler.textContent = item.sampler || 'Default';
        valSeed.textContent = item.seed || 'N/A';
        valSteps.textContent = item.steps ? `${{item.steps}} / ${{item.cfg_scale || 'N/A'}}` : 'N/A';
        valRes.textContent = (item.width && item.height) ? `${{item.width}} × ${{item.height}}` : 'Original';
        valRating.textContent = item.rating ? `★ ${{item.rating}}` : 'Unrated';

        lightbox.classList.add('active');
        document.body.style.overflow = 'hidden';
      }}

      function closeLightbox() {{
        lightbox.classList.remove('active');
        lightboxImg.classList.remove('zoomed');
        document.body.style.overflow = '';
      }}

      document.getElementById('lightbox-close').onclick = closeLightbox;
      document.getElementById('nav-prev').onclick = () => {{
        if (filteredItems.length === 0) return;
        currentIndex = (currentIndex - 1 + filteredItems.length) % filteredItems.length;
        openLightbox(currentIndex);
      }};
      document.getElementById('nav-next').onclick = () => {{
        if (filteredItems.length === 0) return;
        currentIndex = (currentIndex + 1) % filteredItems.length;
        openLightbox(currentIndex);
      }};

      lightboxImg.onclick = () => {{
        lightboxImg.classList.toggle('zoomed');
      }};

      window.addEventListener('keydown', (e) => {{
        if (!lightbox.classList.contains('active')) return;
        if (e.key === 'Escape') closeLightbox();
        if (e.key === 'ArrowLeft') document.getElementById('nav-prev').click();
        if (e.key === 'ArrowRight') document.getElementById('nav-next').click();
      }});

      copyBtn.onclick = () => {{
        const item = filteredItems[currentIndex];
        if (!item || !item.prompt) return;
        navigator.clipboard.writeText(item.prompt).then(() => {{
          copyBtn.textContent = '✓ Copied!';
          setTimeout(() => {{
            copyBtn.textContent = '📋 Copy Prompt';
          }}, 1800);
        }}).catch(() => {{
          copyBtn.textContent = 'Error copying';
        }});
      }};

      renderGallery();
    }})();
  </script>
</body>
</html>
"#,
        escaped_title = escaped_title,
        count = count,
        safe_items_json = safe_items_json
    )
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_html_showcase_contains_data_and_structure() {
        let items = vec![ShowcaseItemMetadata {
            filename: "art_01.webp".to_string(),
            width: 1024,
            height: 1024,
            prompt: Some("masterpiece, cyberpunk city".to_string()),
            negative_prompt: Some("low quality, blurry".to_string()),
            model: Some("cyberpunk_v2.safetensors".to_string()),
            sampler: Some("Euler a".to_string()),
            seed: Some("12345678".to_string()),
            cfg_scale: Some(7.0),
            steps: Some(25),
            rating: Some(5),
        }];

        let html = generate_html_showcase("Test Cyberpunk Album", &items);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Cyberpunk Album"));
        assert!(html.contains("art_01.webp"));
        assert!(html.contains("masterpiece, cyberpunk city"));
        assert!(html.contains("cyberpunk_v2.safetensors"));
    }
}
