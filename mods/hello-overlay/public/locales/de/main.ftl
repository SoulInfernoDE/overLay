mod-description = Beispiel-Mod: zeigt, wie overLay-Mods Übersetzungen, Speicher, Hinweise, Tastenkürzel, Einstellungen und die Netzwerk-Allowlist nutzen.

greeting-friendly = Hallo! overLay läuft über { $game }.
greeting-formal = overLay ist über { $game } aktiv.
waiting = Warte auf das Spiel …

panel-title = Hello Overlay
panel-size = Spielfenster: { $width } × { $height }
panel-count =
    { $count ->
        [one] Einmal geklickt
       *[other] { $count }-mal geklickt
    }
panel-click = Zählen
panel-notify = Hinweis
panel-network = Netzwerk testen
panel-compact-hint = { $hotkey }: kompakt / voll

network-allowed = HearthstoneJSON: erreichbar (HTTP { $status })
network-failed = HearthstoneJSON: nicht erreichbar ({ $error })
network-blocked = example.com: blockiert, wie es sein soll
network-unexpected = example.com: NICHT blockiert, bitte melden

notification = Hallo von Hello Overlay!

hotkey-compact = Panel zwischen kompakt und voll umschalten
setting-greeting = Begrüßung
setting-show-size = Größe des Spielfensters anzeigen

desktop-title = Hello Overlay im Launcher
desktop-hint = Diese Seite läuft im Launcher. Sie teilt ihren Speicher mit dem Overlay-Panel und spricht über den Nachrichtenbus mit ihm.
desktop-reset = Zähler zurücksetzen

panel-live = { $mode } · Zug { $turn } · { $hand } Karten auf der Hand
panel-no-game = Noch keine Partie.
panel-last-event = Letztes Ereignis: { $event }
