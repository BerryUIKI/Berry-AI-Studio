# Generierungs-Interoperabilität

Omera fungiert als aktiver Begleiter Ihrer kreativen Generierungswerkzeuge und bietet eine bidirektionale API-Kommunikation mit **ComfyUI** und **AUTOMATIC1111 / SD.Next**.

---

## 1. Konfiguration der Generierungsdienste

Konfigurieren Sie die Verbindungs-Endpunkte unter **Einstellungen > Metadaten** bzw. im Interop-Bereich:

- **ComfyUI Basis-URL**: Standard `http://127.0.0.1:8188`
- **SD WebUI Basis-URL**: Standard `http://127.0.0.1:7860`

Jedes Konfigurationsfeld verfügt über eine Schaltfläche **„Verbindung testen“**. Omera sendet einen schnellen Status-Ping (`/system_stats` für ComfyUI bzw. `/sdapi/v1/options` für WebUI) und zeigt ein Status-Badge für **Online (🟢)** oder **Offline (🔴)** an.

---

## 2. ComfyUI-Workflow-Übergabe (`send_to_comfyui`)

Beim Betrachten eines Kunstwerks oder Videos, das in ComfyUI generiert wurde:
1. Suchen Sie im rechten Eigenschafts-Inspektor die Karte **Generierungs-Interop**.
2. Klicken Sie auf **„An ComfyUI senden“**.
3. Das Backend von Omera verbindet sich mit dem HTTP-Endpunkt `/prompt` von ComfyUI und übermittelt den exakten Knoten-Graphen und die latenten Parameter des Bildes.
4. Ihre laufende ComfyUI-Instanz lädt den Workflow sofort und reiht ihn in die Ausführungs-Warteschlange ein — ohne dass Sie Dateien mühsam zwischen Fenstern per Drag-and-Drop verschieben müssen.

---

## 3. SD WebUI-Prompt-Übergabe (`send_to_webui`)

Für Bilder, die mit AUTOMATIC1111, Forge oder SD.Next generiert wurden:
1. Klicken Sie im Inspektor auf **„An SD WebUI senden“**.
2. Omera erstellt einen txt2img-Payload mit positivem Prompt, negativem Prompt, Schritten, Sampler, CFG-Skala, Seed und Dimensionen.
3. Der Payload wird an die API `/sdapi/v1/txt2img` der WebUI übergeben, wodurch die Eingabefelder vorbefüllt oder direkt ein neuer Generierungslauf angestoßen wird.

---

## 4. Live-Pipeline-Überwachung im geschlossenen Kreislauf

Wenn Sie die **Generierungs-Interoperabilität** mit einem **Pipeline-Ordner** (Modus C) kombinieren:
1. Sie übergeben einen Workflow direkt aus Omera heraus an ComfyUI oder WebUI.
2. Der Generator berechnet das Bild und schreibt es in sein Ausgabe-Verzeichnis.
3. Der Hintergrundwächter von Omera registriert die neue Datei, entprellt Schreibsperren (500 ms), extrahiert die Metadaten, rendert ein WebP-Vorschaubild und platziert das fertige Kunstwerk in Echtzeit ganz oben in Ihrer Galerie.
4. Dadurch entsteht ein nahtloser, unterbrechungsfreier Arbeitsablauf zwischen Generierung und Kuration.
