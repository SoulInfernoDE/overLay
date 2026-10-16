# Host shell: window chrome, dock, tray and top-level sections.

-app-name = overLay

shell-window-minimize = Minimieren
shell-window-close = Schließen
shell-dock-label = Hauptnavigation

tray-open = { -app-name } öffnen
tray-quit = Beenden
tray-tooltip-idle = { -app-name }
tray-tooltip-running = { -app-name } · { $game } läuft

nav-library = Bibliothek
nav-store = Store
nav-settings = Einstellungen

library-title = Bibliothek
library-games-title = Spiele
library-mods-title = Mods
library-empty = Noch keine Mods installiert.
library-empty-action = Store durchstöbern

game-status-running =
    { $flavor ->
        [wine] Läuft (Wine/Proton)
       *[native] Läuft
    }
game-status-idle = Läuft nicht

store-title = Store
store-placeholder = Der Mod-Store kommt mit dem signierten Index in Phase 4.

settings-title = Einstellungen
settings-language = Sprache
settings-language-system = Systemsprache
settings-general = Allgemein
settings-autostart = { -app-name } beim Anmelden starten
settings-start-minimized = Im Infobereich starten
settings-overlay = Overlay
settings-overlay-enabled = Overlay über laufenden Spielen anzeigen
settings-overlay-demo = Demo-Widget anzeigen
settings-hotkeys = Tastenkürzel
settings-hotkey-launcher = Launcher ein- oder ausblenden
settings-hotkey-overlay = Overlay ein- oder ausblenden
settings-hotkey-hint = Tastenkürzel lassen sich vorerst in { $path } ändern.
settings-save-failed = Die Einstellungen konnten nicht gespeichert werden: { $reason }

status-version = { -app-name } { $version }

provider-restart-needed = { -app-name } hat die Spiel-Logs eingeschaltet, die { $game } für Live-Spielereignisse braucht. Starte { $game } einmal neu, um sie zu nutzen.
