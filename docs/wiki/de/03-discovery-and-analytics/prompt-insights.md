# Prompt-Analysen & Einblicke

Mit wachsender Bildersammlung wird es entscheidend zu verstehen, welche Prompt-Schlüsselwörter, Künstler und technischen Parameter zu Ihren am besten bewerteten Kunstwerken führen. Omera bietet ein dediziertes Analyse-Dashboard (`PromptStatsModal.vue`), das Metadaten über Ihre gesamte Bibliothek hinweg aggregiert.

---

## 1. Prompt-Einblicke öffnen

So öffnen Sie das Analyse-Dashboard:
- Wählen Sie in der oberen Menüleiste **Werkzeuge > Prompt-Statistiken...**, oder
- Klicken Sie in der linken Seitenleiste in der Fußzeile auf **Einblicke**.

---

## 2. Analyse-Kategorien & Visuelle Metriken

```
┌────────────────────────────────────────────────────────────────────────┐
│ Prompt- & Metadaten-Einblicke                                      [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [ Häufigste positive Wörter ] [ Negative Wörter ] [ Modelle ] [ Sampler]│
├────────────────────────────────────────────────────────────────────────┤
│ Sortieren nach: (●) Vorkommen    ( ) Durchschnittsbewertung            │
├────────────────────────────────────────────────────────────────────────┤
│ Rang │ Token / Schlüsselwort        │ Vorkommen   │ Durchschnitt │Aktion│
├──────┼──────────────────────────────┼─────────────┼──────────────┼──────┤
│ #1   │ masterpiece                  │ 4.120       │ ★ 4.2        │ [🔍] │
│ #2   │ cinematic lighting           │ 2.845       │ ★ 4.7        │ [🔍] │
│ #3   │ 1girl                        │ 2.410       │ ★ 3.9        │ [🔍] │
│ #4   │ cyberpunk city               │ 1.890       │ ★ 4.8        │ [🔍] │
│ #5   │ volumetric fog               │ 1.230       │ ★ 4.5        │ [🔍] │
│ #6   │ photorealistic               │ 1.115       │ ★ 3.2        │ [🔍] │
└────────────────────────────────────────────────────────────────────────┘
```

Das Dashboard bietet vier spezialisierte Analyseansichten:

### 1. Häufigste positive Wörter (Positive Tokens)
- Analysiert einzelne Schlüsselwort-Chips über alle positiven Generierungs-Prompts hinweg.
- Zeigt die Gesamtzahl der Vorkommen neben der **Durchschnittsbewertung** der entsprechenden Bilder an.
- Hebt Ihre erfolgreichsten Schlüsselwörter hervor — Begriffe, die überdurchschnittlich oft mit 4★- und 5★-Bewertungen korrelieren.

### 2. Häufigste negative Wörter (Negative Tokens)
- Analysiert, welche negativen Begriffe in Ihrem Workflow am häufigsten auftreten.
- Nützlich zum Erkennen redundanter oder überladener negativer Embeddings und Phrasen, die die Bildqualität nicht messbar verbessern.

### 3. Meistgenutzte Checkpoint-Modelle
- Listet jedes Modell in Ihrer Bibliothek nach Gesamtzahl der Generationen und Benutzerbewertungen auf.
- Hilft bei der Beurteilung, welche Finetunes und Checkpoints konstant die überzeugendsten Resultate liefern.

### 4. Meistgenutzte Sampler & Scheduler
- Ordnet Sampling-Algorithmen (z. B. `DPM++ 2M Karras`, `Euler a`, `UniPC`) nach Häufigkeit und durchschnittlicher Bewertung.

---

## 3. Interaktive Suchintegration

Jede Zeile in der Analysetabelle enthält eine **Aktionsschaltfläche (`🔍`)**:
- Ein Klick auf das Lupensymbol schließt das Analysefenster sofort, übernimmt das gewählte Token in das Hauptsuchfeld der Galerie (`prompt:"..."`) und filtert Ihren Arbeitsbereich auf alle Kunstwerke, die dieses Schlüsselwort enthalten.
- So wechseln Sie nahtlos von übergreifenden Statistiken zur direkten visuellen Begutachtung konkreter Bildserien.
