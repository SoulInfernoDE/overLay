# Spike results: mod sandbox

> English original. German version: [RESULTS.de.md](RESULTS.de.md)

Question: can a mod run as a sandboxed frame inside the shared overlay webview, with its files served by the host over a custom scheme and its network access limited by a CSP the host sets as a response header?

Setup: Tauri 2.12.3, WebKitGTK 2.52.6, Linux Mint 22.3. The host page creates `<iframe sandbox="allow-scripts" src="mod://localhost/hello/index.html">`; the `mod` protocol handler answers with `Content-Security-Policy` (`default-src 'none'`, scripts and own files from `mod://localhost`, `connect-src` limited to `https://api.hearthstonejson.com`, `report-uri` back to the host) and `Access-Control-Allow-Origin: *`. Run `cargo run --release`; the results are printed to stderr.

| Check | Result |
|---|---|
| The mod's script runs; its origin is opaque (`null`) | ✓ |
| Access to the host page's DOM | blocked (`SecurityError`) |
| `localStorage` | blocked (`SecurityError`); storage goes through the host API |
| `eval` | blocked by the CSP (`EvalError`) |
| `fetch` to a declared domain | allowed (HTTP 200) |
| `fetch` to an undeclared domain | blocked; the browser POSTs a CSP report to the host's protocol handler |
| The mod's own files through relative URLs | ✓ |
| `postMessage` in both directions; the host can tell which frame sent a message (`event.source`) | ✓ |

**Conclusion:** the design holds. Mods run as sandboxed frames without `allow-same-origin`, the host serves their files and sets the CSP, network access is enforced by the webview and every blocked request is reported to the host for logging. Mods talk to the host only through `postMessage`, and the host checks permissions per frame. Still to be verified on Windows (WebView2 maps the scheme to `http://mod.localhost`).
