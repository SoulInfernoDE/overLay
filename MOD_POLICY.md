# Mod policy

> English original. German version: [MOD_POLICY.de.md](MOD_POLICY.de.md)
> Applies to every mod listed in the official index (`overlay-mods-index`). Version 1 · 2026-10-14

overLay exists because gamers deserve overlays without ads, tracking and bloat. This policy turns that promise into rules. The index CI checks what it can automatically, and maintainers review the rest.

## 1. Open source

1. The complete source code of the mod is publicly available in a git repository linked in the manifest (`source`).
2. The mod is under an [OSI-approved](https://opensource.org/licenses) license, declared as an SPDX identifier in the manifest (`license`).
3. Every listed release points to a git tag and commit. The index CI builds the mod from that commit, and the result must match the published artifact byte for byte (**reproducible build**).
4. Generated or minified code is only allowed as a build output, never as the only form of the source.

## 2. No telemetry, no ads

The following are **forbidden**, whether in the mod itself or in its dependencies:

- analytics, usage statistics, heatmaps, session recording
- crash or error reporting to third parties
- advertising of any kind, ad SDKs, affiliate or referral tracking
- fingerprinting, tracking pixels, unique device or user IDs sent over the network
- cryptocurrency mining or any other use of the user's resources for third parties

## 3. Network access

1. Every domain a mod contacts is declared in the manifest (`network`), and its purpose is documented in the mod's README.
2. The host blocks every request to undeclared domains. Trying to work around this (e.g. through redirects, proxies or DNS tricks) leads to removal.
3. Requests carry no personal data beyond what the feature strictly needs.
4. **No remote code.** A mod must not load and run JavaScript, WebAssembly or other code from the network. All code ships inside the reviewed artifact. Remote content is data only (JSON, images, etc.).

## 4. User data

1. Data stays local by default (mod storage provided by the host).
2. Any upload of user or gameplay data is **opt-in**: off by default, explained in plain language before it is enabled, and possible to switch off and delete again.
3. Such uploads may only go to services whose server code is open source and self-hostable.

## 5. Updates

1. Mods are updated only through the index pipeline (signed index, verified hash, staged activation).
2. A mod must not update itself, download plugins, or replace its own files.
3. A release that adds permissions or network domains is reviewed manually and must be approved by the user before activation.

## 6. Fair play

1. No code injection into game clients and no modification of game files, except what the host's game provider does to enable official logging (e.g. Hearthstone's `log.config`).
2. No automation of game input (bots, macros that play for the user).
3. A mod shows only information the player could legitimately know or track on their own. It must not reveal hidden information (e.g. the opponent's hand) that the game does not expose to the player.
4. Mods respect the terms of service of the supported game. If in doubt, ask in an issue before you build the feature.

## 7. Intellectual property

1. No code, assets or text copied from projects whose license does not allow it. Recreating another product's features is allowed only as a **clean-room** implementation: use public feature descriptions and screenshots, never the original source.
2. Game assets (card art, icons, sounds) are not bundled unless their license allows redistribution. Load them at runtime from the source the game publisher permits.
3. Do not use other products' names or logos in a way that suggests affiliation.

## 8. Localization

1. All user-visible text lives in locale catalogs (`locales/<locale>/*.ftl`). Nothing is hard-coded.
2. English is the base and fallback locale. Further languages are welcome.

## 9. Review, removal and appeal

1. Every new mod and every release is reviewed through a pull request in the index repository.
2. Maintainers can remove a mod or yank a release at any time if it breaks this policy or puts users at risk. Security issues lead to an immediate yank.
3. Authors can appeal a decision by opening an issue in the index repository.

## 10. Custom repositories

This policy applies to the official index. Users can add custom repositories at their own risk. The host still enforces the sandbox, the network allowlist and signature checks for every mod, wherever it comes from, and marks mods from custom repositories in the UI.
