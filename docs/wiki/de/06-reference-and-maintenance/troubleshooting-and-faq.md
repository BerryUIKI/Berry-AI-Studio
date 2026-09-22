# Fehlerbehebung & Häufig gestellte Fragen (FAQ)

Dieser Leitfaden behandelt typische Fragen, Problemstellungen und Schritte zur Fehlerbehebung in Berry AI Studio.

---

## 1. Häufige Probleme & Lösungen

### Problem: Neu generierte Bilder meines KI-Generators erscheinen nicht in der Galerie
- **Ursache**: Der Ordner ist möglicherweise nicht als aktive Erfassungs-Pipeline eingerichtet, oder die Entprellzeit des Dateiwächters wartet noch auf das vollständige Schreiben der Datei.
- **Lösung**:
  1. Prüfen Sie, ob der Ordner in der linken Seitenleiste registriert ist.
  2. Stellen Sie bei Nutzung von AUTOMATIC1111 oder ComfyUI sicher, dass der Ordner im **Modus C: Pipeline** angelegt wurde, damit die Schreibsperren-Entprellung aktiv ist.
  3. Klicken Sie mit der rechten Maustaste auf den Ordner und wählen Sie **„Neue Bilder erfassen“** oder **„Ordner scannen“**, um eine sofortige Prüfung anzustoßen.

### Problem: Vorschaubilder laden verzögert oder zeigen Platzhalter-Kästen
- **Ursache**: Bei sehr großen initialen Importen rendern die Rayon-Hintergrund-Threads Vorschaubilder schrittweise im Hintergrund.
- **Lösung**:
  1. Klicken Sie in der unteren Statusleiste auf **⚡ Aktivität**, um den Fortschritt der Vorschaubild-Warteschlange zu prüfen.
  2. Stellen Sie unter **Einstellungen > Galerie** sicher, dass das **Thumbnail Cache Budget** auf mindestens `2048 MB` gesetzt ist.
  3. Vermeiden Sie ressourcenintensive 3D-Rendervorgänge oder Spiele während des ersten großen Datei-Imports.

### Problem: Die Suche liefert 0 Treffer, obwohl das Wort im Prompt enthalten ist
- **Ursache**: Möglicherweise befindet sich die Suche im **Semantischen Modus (`🧠`)** statt im **Syntax-Modus (`🔍`)**, oder ein aktiver Filter grenzt die Ergebnisse zu stark ein.
- **Lösung**:
  1. Prüfen Sie das Symbol in der Suchleiste: Klicken Sie auf das Gehirn-Symbol, um zurück zur **Syntaxsuche (`🔍`)** zu wechseln.
  2. Setzen Sie Phrasen mit Leerzeichen in Anführungszeichen: `prompt:"cyberpunk city"`.
  3. Öffnen Sie das **Filter-Bedienfeld (`☰`)** und klicken Sie auf **„Zurücksetzen“**, um sicherzustellen, dass keine Filterkriterien (z. B. eine 5-Sterne-Mindestbewertung) Treffer ausblenden.

### Problem: Bildstapel erscheinen getrennt oder doppelt
- **Ursache**: Stapel können fragmentiert werden, wenn enthaltene Bilder extern über den Betriebssystem-Dateimanager umbenannt oder verschoben wurden.
- **Lösung**: Markieren Sie die betroffenen Bildkarten in der Galerie und drücken Sie `Strg + G`, um sie sauber zu einem gemeinsamen Stapel zusammenzuführen.

### Problem: „An ComfyUI senden“ meldet Verbindungsfehler
- **Ursache**: ComfyUI läuft nicht lokal oder nutzt einen abweichenden Port.
- **Lösung**:
  1. Öffnen Sie **Einstellungen > Generierungs-Interop**.
  2. Vergleichen Sie die **ComfyUI Basis-URL** mit der Konsolenausgabe Ihres ComfyUI-Terminals (Standard: `http://127.0.0.1:8188`).
  3. Klicken Sie auf **„Verbindung testen“**, um zu überprüfen, ob der Port erreichbar ist.

---

## 2. Häufig gestellte Fragen (FAQ)

### Ist Berry AI Studio vollkommen kostenlos nutzbar?
Ja. Berry AI Studio ist freie Open-Source-Software unter der **AGPL-3.0**. Es gibt weder Abonnements noch Bezahlschranken oder gesperrte Premium-Funktionen.

### Eignet sich Berry AI Studio für Bibliotheken mit 100.000+ oder 500.000+ Bildern?
Ja. Berry wurde von Grund auf für gewaltige Bildbestände konzipiert:
- **Keyset-Cursor-Tiefe-Paginierung** (`search_files_cursor_page`) garantiert Abfragelatenzen im Sub-Millisekunden-Bereich unabhängig von der Bibliotheksgröße.
- **SQLite Write-Ahead Logging (WAL)** sorgt für extrem schnelle, blockierungsfreie parallele Lesezugriffe.
- **Dynamische DOM-Virtualisierung** rendert ausschließlich die Karten, die sich aktuell im sichtbaren Bereich des Bildschirms befinden.

### Lädt Berry AI Studio meine Prompts oder Bilder in die Cloud hoch?
Nein. Sämtliche Scans, Metadaten-Extraktionen, Datenbankzugriffe und KI-Inferenzen (CLIP und WD14) laufen zu 100% lokal auf Ihrem eigenen Rechner. Es werden keinerlei Telemetriedaten erfasst.

### Kann ich Bilder per Drag & Drop direkt in ComfyUI oder Discord ziehen?
Ja. Wenn Sie eine Bildkarte aus der Galerie in einen Webbrowser oder eine externe Desktop-Applikation ziehen, überträgt Berry Standard-Dateisystem-Payloads unter vollständiger Beibehaltung aller eingebetteten Metadatenblöcke.

### Was geschieht, wenn ich einen Ordner aus der linken Seitenleiste entferne?
Das Entfernen eines Ordners aus Berry AI Studio löscht lediglich das Verzeichnis und seine Indexeinträge aus der SQLite-Datenbank. **Ihre physischen Originaldateien auf dem Datenträger werden niemals berührt oder gelöscht.**

### Können mehrere Teammitglieder gemeinsam an derselben Bibliothek arbeiten?
Ja. Durch den Wechsel von SQLite zu einer gemeinsamen **MySQL 8.0+**- oder **PostgreSQL 14+**-Datenbank unter **Einstellungen > Team & Datenbank** können mehrere Workstations gleichzeitig auf ein geteiltes Netzwerk-Repository zugreifen — mit Echtzeit-Synchronisation und plattformübergreifendem Pfad-Mapping.
