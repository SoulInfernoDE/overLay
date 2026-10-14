# Contributing to overLay

> English original. German version: [CONTRIBUTING.de.md](CONTRIBUTING.de.md)

Thanks for helping! By taking part you agree to the [code of conduct](CODE_OF_CONDUCT.md).

## Setup

See [README → Development](README.md#development). Before you open a pull request, run all checks listed there.

## Ground rules

1. **No telemetry, no ads.** Do not add analytics, crash reporters, trackers or ad SDKs, not even as a dependency. See the [mod policy](MOD_POLICY.md), which applies to the host as well.
2. **Clean room.** Never copy code, assets or text from projects whose license does not allow it. This explicitly includes **Firestone** (no license) and the closed-source parts of **Overwolf** and **CurseForge**. Public feature lists, documentation and screenshots are fine as references. If you have read their source code, say so in your pull request and do not work on the matching feature.
3. **Licenses.** New dependencies must pass `cargo deny check` and `npm run licenses`. Do not change the allowlists without discussing it in an issue first.
4. **Latest versions.** Use the latest stable version of a dependency unless there is a documented reason not to (e.g. a peer-dependency limit). Dependabot keeps them current.

## Localization

- No hard-coded user-visible text. Add messages to `locales/en/*.ftl` **and** `locales/de/*.ftl`. The test `crates/overlay-i18n/tests/repo_catalogs.rs` fails when a translation is missing.
- Proper names (overLay, Flint) and fixed terms such as HTTPS or SSH may stay as they are.

## Documentation

User-facing docs (README, roadmap, policies, changelog, release notes) are written in English with a German version in a `*.de.md` file next to them. Update both in the same pull request. Code comments are English only.

## Commits

- Write the commit message in English, followed by a paragraph starting with `Deutsch:` that contains the German version.
- Sign off every commit (`git commit -s`). The sign-off confirms the [Developer Certificate of Origin](https://developercertificate.org/), which matters for a clean-room project.

Example:

```
Add Fluent loader with English fallback

Deutsch: Fluent-Loader mit englischem Fallback hinzugefügt

Signed-off-by: Jane Doe <jane@example.org>
```

## Pull requests

- Keep pull requests small and focused on one topic.
- UI changes: add before/after screenshots, in English and German.
- Fill in the checklist of the pull request template.
