# Ergebnisse des Spikes: Mod-Sandbox

> Deutsche Fassung. Englisches Original: [RESULTS.md](RESULTS.md)

Frage: Kann ein Mod als abgeschotteter Frame in der gemeinsamen Overlay-WebView laufen, wobei der Host seine Dateien über ein eigenes URL-Schema ausliefert und seinen Netzwerkzugriff über eine CSP im Antwort-Header begrenzt?

Aufbau: Tauri 2.12.3, WebKitGTK 2.52.6, Linux Mint 22.3. Die Host-Seite erzeugt `<iframe sandbox="allow-scripts" src="mod://localhost/hello/index.html">`; der Protokoll-Handler für `mod` antwortet mit `Content-Security-Policy` (`default-src 'none'`, Skripte und eigene Dateien von `mod://localhost`, `connect-src` nur `https://api.hearthstonejson.com`, `report-uri` zurück an den Host) und `Access-Control-Allow-Origin: *`. Start mit `cargo run --release`; die Ergebnisse erscheinen auf stderr.

| Prüfung | Ergebnis |
|---|---|
| Das Skript des Mods läuft; sein Ursprung ist undurchsichtig (`null`) | ✓ |
| Zugriff auf das DOM der Host-Seite | blockiert (`SecurityError`) |
| `localStorage` | blockiert (`SecurityError`); Speicher läuft über die Host-API |
| `eval` | durch die CSP blockiert (`EvalError`) |
| `fetch` an eine deklarierte Domain | erlaubt (HTTP 200) |
| `fetch` an eine nicht deklarierte Domain | blockiert; der Browser schickt per POST einen CSP-Bericht an den Protokoll-Handler des Hosts |
| Eigene Dateien des Mods über relative URLs | ✓ |
| `postMessage` in beide Richtungen; der Host erkennt, welcher Frame gesendet hat (`event.source`) | ✓ |

**Fazit:** Das Konzept trägt. Mods laufen als Sandbox-Frames ohne `allow-same-origin`, der Host liefert ihre Dateien aus und setzt die CSP, den Netzwerkzugriff erzwingt die WebView, und jede blockierte Anfrage wird dem Host zum Protokollieren gemeldet. Mods sprechen nur über `postMessage` mit dem Host, und der Host prüft die Berechtigungen pro Frame. Unter Windows noch zu prüfen (WebView2 bildet das Schema auf `http://mod.localhost` ab).
