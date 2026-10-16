# Host shell: window chrome, dock, tray and top-level sections.

-app-name = overLay

shell-window-minimize = Minimize
shell-window-close = Close
shell-dock-label = Main navigation

tray-open = Open { -app-name }
tray-quit = Quit
tray-tooltip-idle = { -app-name }
tray-tooltip-running = { -app-name } · { $game } is running

nav-library = Library
nav-store = Store
nav-settings = Settings

library-title = Library
library-games-title = Games
library-mods-title = Mods
library-empty = No mods installed yet.
library-empty-action = Browse the store

game-status-running =
    { $flavor ->
        [wine] Running (Wine/Proton)
       *[native] Running
    }
game-status-idle = Not running

store-title = Store
store-placeholder = The mod store arrives with the signed index in phase 4.

settings-title = Settings
settings-language = Language
settings-language-system = System language
settings-general = General
settings-autostart = Start { -app-name } when I log in
settings-start-minimized = Start in the system tray
settings-overlay = Overlay
settings-overlay-enabled = Show the overlay over running games
settings-overlay-demo = Show the demo widget
settings-hotkeys = Hotkeys
settings-hotkey-launcher = Show or hide the launcher
settings-hotkey-overlay = Show or hide the overlay
settings-hotkey-hint = Hotkeys can be changed in { $path } for now.
settings-save-failed = Settings could not be saved: { $reason }

status-version = { -app-name } { $version }

provider-restart-needed = { -app-name } enabled the game logs { $game } needs for live game events. Restart { $game } once to use them.
