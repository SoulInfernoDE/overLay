# Mod-Richtlinie

> Deutsche Fassung. Englisches Original: [MOD_POLICY.md](MOD_POLICY.md)
> Gilt für jeden Mod im offiziellen Index (`overlay-mods-index`). Version 1 · 2026-10-14

overLay gibt es, weil Spieler Overlays ohne Werbung, Tracking und Ballast verdienen. Diese Richtlinie macht aus diesem Versprechen Regeln. Die Index-CI prüft, was sich automatisch prüfen lässt, den Rest prüfen die Maintainer.

## 1. Open Source

1. Der vollständige Quellcode des Mods liegt öffentlich in einem Git-Repository, das im Manifest verlinkt ist (`source`).
2. Der Mod steht unter einer [OSI-anerkannten](https://opensource.org/licenses) Lizenz, die im Manifest als SPDX-Kennung angegeben ist (`license`).
3. Jedes gelistete Release verweist auf einen Git-Tag und einen Commit. Die Index-CI baut den Mod aus diesem Commit, und das Ergebnis muss Byte für Byte dem veröffentlichten Artefakt entsprechen (**reproduzierbarer Build**).
4. Generierter oder minifizierter Code ist nur als Build-Ergebnis erlaubt, nie als einzige Form des Quellcodes.

## 2. Keine Telemetrie, keine Werbung

Folgendes ist **verboten**, im Mod selbst ebenso wie in seinen Abhängigkeiten:

- Analytics, Nutzungsstatistiken, Heatmaps, Sitzungsaufzeichnung
- Absturz- oder Fehlerberichte an Dritte
- Werbung jeder Art, Werbe-SDKs, Affiliate- oder Empfehlungs-Tracking
- Fingerprinting, Tracking-Pixel, eindeutige Geräte- oder Nutzer-IDs, die über das Netz gesendet werden
- Krypto-Mining oder jede andere Nutzung der Ressourcen des Nutzers für Dritte

## 3. Netzwerkzugriff

1. Jede Domain, die ein Mod kontaktiert, ist im Manifest deklariert (`network`), und ihr Zweck ist in der README des Mods beschrieben.
2. Der Host blockiert jede Anfrage an nicht deklarierte Domains. Wer versucht, das zu umgehen (z. B. über Weiterleitungen, Proxys oder DNS-Tricks), wird entfernt.
3. Anfragen enthalten keine personenbezogenen Daten über das hinaus, was die Funktion zwingend braucht.
4. **Kein nachgeladener Code.** Ein Mod darf kein JavaScript, WebAssembly oder anderen Code aus dem Netz laden und ausführen. Der gesamte Code steckt im geprüften Artefakt. Inhalte aus dem Netz sind nur Daten (JSON, Bilder usw.).

## 4. Nutzerdaten

1. Daten bleiben standardmäßig lokal (Mod-Speicher des Hosts).
2. Jeder Upload von Nutzer- oder Spieldaten ist **Opt-in**: standardmäßig aus, vor dem Einschalten verständlich erklärt, jederzeit abschaltbar und löschbar.
3. Solche Uploads dürfen nur an Dienste gehen, deren Server-Code quelloffen und selbst hostbar ist.

## 5. Updates

1. Mods werden nur über die Index-Pipeline aktualisiert (signierter Index, geprüfter Hash, gestaffelte Aktivierung).
2. Ein Mod darf sich nicht selbst aktualisieren, keine Plugins nachladen und seine eigenen Dateien nicht ersetzen.
3. Ein Release, das Berechtigungen oder Netzwerk-Domains hinzufügt, wird manuell geprüft und muss vor der Aktivierung vom Nutzer bestätigt werden.

## 6. Fair Play

1. Keine Code-Injektion in Spielclients und keine Änderung von Spieldateien. Ausgenommen ist, was der Game Provider des Hosts tut, um das offizielle Logging einzuschalten (z. B. Hearthstones `log.config`).
2. Keine Automatisierung von Spieleingaben (Bots, Makros, die für den Nutzer spielen).
3. Ein Mod zeigt nur Informationen, die der Spieler legitim wissen oder selbst mitverfolgen könnte. Er darf keine verborgenen Informationen aufdecken (z. B. die Hand des Gegners), die das Spiel dem Spieler nicht zeigt.
4. Mods halten die Nutzungsbedingungen des unterstützten Spiels ein. Im Zweifel vorher in einem Issue fragen.

## 7. Geistiges Eigentum

1. Kein Code, keine Assets und keine Texte aus Projekten, deren Lizenz das nicht erlaubt. Funktionen eines anderen Produkts nachzubauen ist nur als **Clean-Room**-Implementierung erlaubt: öffentliche Funktionsbeschreibungen und Screenshots ja, Original-Quellcode nie.
2. Spiel-Assets (Kartenbilder, Icons, Sounds) werden nicht mitgeliefert, wenn ihre Lizenz die Weitergabe nicht erlaubt. Sie werden zur Laufzeit aus der Quelle geladen, die der Publisher erlaubt.
3. Namen oder Logos anderer Produkte nicht so verwenden, dass eine Verbindung suggeriert wird.

## 8. Lokalisierung

1. Alle sichtbaren Texte stehen in Locale-Katalogen (`locales/<locale>/*.ftl`). Nichts ist fest im Code verdrahtet.
2. Englisch ist Basis- und Fallback-Sprache. Weitere Sprachen sind willkommen.

## 9. Prüfung, Entfernung und Einspruch

1. Jeder neue Mod und jedes Release wird über einen Pull Request im Index-Repository geprüft.
2. Maintainer können einen Mod jederzeit entfernen oder ein Release zurückziehen, wenn er gegen diese Richtlinie verstößt oder Nutzer gefährdet. Sicherheitsprobleme führen zum sofortigen Zurückziehen.
3. Autoren können über ein Issue im Index-Repository Einspruch einlegen.

## 10. Eigene Repositories

Diese Richtlinie gilt für den offiziellen Index. Nutzer können eigene Repositories auf eigenes Risiko hinzufügen. Der Host setzt aber auch für diese Mods die Sandbox, die Netzwerk-Allowlist und die Signaturprüfung durch und kennzeichnet Mods aus eigenen Repositories in der Oberfläche.
