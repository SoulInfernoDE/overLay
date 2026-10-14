# Bei overLay mitwirken

> Deutsche Fassung. Englisches Original: [CONTRIBUTING.md](CONTRIBUTING.md)

Danke für deine Hilfe! Wer mitmacht, erkennt den [Verhaltenskodex](CODE_OF_CONDUCT.de.md) an.

## Einrichtung

Siehe [README → Entwicklung](README.de.md#entwicklung). Führe vor einem Pull Request alle dort aufgeführten Prüfungen aus.

## Grundregeln

1. **Keine Telemetrie, keine Werbung.** Keine Analytics, Absturzberichte, Tracker oder Werbe-SDKs, auch nicht als Abhängigkeit. Siehe [Mod-Richtlinie](MOD_POLICY.de.md), die auch für den Host gilt.
2. **Clean Room.** Kopiere nie Code, Assets oder Texte aus Projekten, deren Lizenz das nicht erlaubt. Das gilt ausdrücklich für **Firestone** (keine Lizenz) und die geschlossenen Teile von **Overwolf** und **CurseForge**. Öffentliche Funktionslisten, Dokumentation und Screenshots sind als Referenz in Ordnung. Wenn du deren Quellcode gelesen hast, gib das im Pull Request an und arbeite nicht an der entsprechenden Funktion.
3. **Lizenzen.** Neue Abhängigkeiten müssen `cargo deny check` und `npm run licenses` bestehen. Ändere die Allowlists nicht, ohne es vorher in einem Issue zu besprechen.
4. **Neueste Versionen.** Nimm die neueste stabile Version einer Abhängigkeit, außer es gibt einen dokumentierten Grund dagegen (z. B. eine Grenze bei Peer-Dependencies). Dependabot hält die Versionen aktuell.

## Lokalisierung

- Keine fest verdrahteten sichtbaren Texte. Neue Texte kommen in `locales/en/*.ftl` **und** `locales/de/*.ftl`. Der Test `crates/overlay-i18n/tests/repo_catalogs.rs` schlägt fehl, wenn eine Übersetzung fehlt.
- Eigennamen (overLay, Flint) und feste Begriffe wie HTTPS oder SSH dürfen unverändert bleiben.

## Dokumentation

Dokumente für Nutzer (README, Fahrplan, Richtlinien, Änderungsprotokoll, Release-Notes) werden auf Englisch geschrieben, mit einer deutschen Fassung in einer `*.de.md`-Datei daneben. Aktualisiere beide im selben Pull Request. Code-Kommentare sind nur auf Englisch.

## Commits

- Schreibe die Commit-Nachricht auf Englisch, darunter einen Absatz, der mit `Deutsch:` beginnt und die deutsche Fassung enthält.
- Zeichne jeden Commit ab (`git commit -s`). Damit bestätigst du das [Developer Certificate of Origin](https://developercertificate.org/), was für ein Clean-Room-Projekt wichtig ist.

Beispiel:

```
Add Fluent loader with English fallback

Deutsch: Fluent-Loader mit englischem Fallback hinzugefügt

Signed-off-by: Jane Doe <jane@example.org>
```

## Pull Requests

- Halte Pull Requests klein und auf ein Thema beschränkt.
- Änderungen an der Oberfläche: Vorher-/Nachher-Screenshots auf Englisch und Deutsch anhängen.
- Fülle die Checkliste der Pull-Request-Vorlage aus.
