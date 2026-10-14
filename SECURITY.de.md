# Sicherheitsrichtlinie

> Deutsche Fassung. Englisches Original: [SECURITY.md](SECURITY.md)

## Eine Schwachstelle melden

Bitte **kein** öffentliches Issue eröffnen. Melde Schwachstellen vertraulich über GitHub: Repository → *Security* → *Report a vulnerability*.

Wir wollen eine Meldung innerhalb von 7 Tagen bestätigen und mit dir einen Termin für die Veröffentlichung abstimmen.

## Geltungsbereich

Besonders relevant sind:

- Ausbruch aus der Mod-Sandbox (Zugriff auf Dateisystem, Prozesse oder Netzwerk über die erteilten Berechtigungen hinaus)
- Umgehung der Netzwerk-Allowlist
- die Update-Kette: Index-Signaturen, Artefakt-Hashes, Staging und Aktivierung
- der Speicherleser und andere Game Provider
- die CI des Mod-Index (z. B. Wege, ein nicht reproduzierbares Artefakt durchzubringen)

Schwachstellen in einem Mod gehören in dessen eigenes Repository. Ist ein Mod im offiziellen Index bösartig, melde ihn bitte zusätzlich hier, damit er zurückgezogen werden kann.

## Unterstützte Versionen

overLay ist Pre-Alpha. Bis 1.0 bekommt nur der neueste Commit auf `main` Korrekturen.
