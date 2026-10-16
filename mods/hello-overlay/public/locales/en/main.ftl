mod-description = Example mod: shows how overLay mods use translations, storage, notifications, hotkeys, settings and the network allowlist.

greeting-friendly = Hi! overLay is running over { $game }.
greeting-formal = overLay is active over { $game }.
waiting = Waiting for the game …

panel-title = Hello Overlay
panel-size = Game window: { $width } × { $height }
panel-count =
    { $count ->
        [one] Clicked once
       *[other] Clicked { $count } times
    }
panel-click = Count
panel-notify = Notify
panel-network = Test network
panel-compact-hint = { $hotkey }: compact / full

network-allowed = HearthstoneJSON: reachable (HTTP { $status })
network-failed = HearthstoneJSON: not reachable ({ $error })
network-blocked = example.com: blocked, as it should be
network-unexpected = example.com: NOT blocked, please report this

notification = Hello from Hello Overlay!

hotkey-compact = Switch the panel between compact and full
setting-greeting = Greeting
setting-show-size = Show the game window size

desktop-title = Hello Overlay in the launcher
desktop-hint = This page runs in the launcher. It shares its storage with the overlay panel and talks to it over the message bus.
desktop-reset = Reset the counter

panel-live = { $mode } · turn { $turn } · { $hand } cards in hand
panel-no-game = No game running yet.
panel-last-event = Last event: { $event }
