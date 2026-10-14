//! Serves mod files to mod frames over the `mod` scheme (`mod://localhost/<id>/<file>`;
//! WebView2 on Windows maps it to `http://mod.localhost/<id>/<file>`).
//!
//! Each response carries the mod's Content-Security-Policy: scripts and files only from
//! the mod itself, network access only to the https hosts its manifest declares. The
//! webview enforces it and reports every blocked request to `/__report/<id>`, which is
//! passed on to the core for logging. See `spikes/mod-sandbox/RESULTS.md`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use overlay_ipc::{ModList, Request};
use overlay_mods::resolve_file;
use tauri::http::{Request as HttpRequest, Response};
use tauri::{AppHandle, Manager, Runtime};

use super::CoreClient;

pub const SCHEME: &str = "mod";

/// Origin of mod frames as the webview sees it.
pub fn base_url() -> &'static str {
    if cfg!(windows) {
        "http://mod.localhost"
    } else {
        "mod://localhost"
    }
}

/// The page needs the origin of mod frames to build their URLs.
#[tauri::command]
pub(crate) fn mod_base_url() -> &'static str {
    base_url()
}

struct ModEntry {
    root: PathBuf,
    network: Vec<String>,
}

/// Mod folders and network hosts, kept in sync with the core's mod list.
#[derive(Default)]
pub struct ModMap(RwLock<HashMap<String, ModEntry>>);

impl ModMap {
    pub fn update(&self, list: &ModList) {
        let entries = list
            .mods
            .iter()
            .filter(|m| m.enabled)
            .map(|m| {
                let entry = ModEntry {
                    root: PathBuf::from(&m.path),
                    network: m.manifest.network.clone(),
                };
                (m.manifest.id.clone(), entry)
            })
            .collect();
        *self.0.write().expect("mod map lock") = entries;
    }

    fn lookup(&self, id: &str) -> Option<(PathBuf, Vec<String>)> {
        self.0
            .read()
            .expect("mod map lock")
            .get(id)
            .map(|e| (e.root.clone(), e.network.clone()))
    }
}

pub fn register<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.register_asynchronous_uri_scheme_protocol(SCHEME, |ctx, request, responder| {
        let app = ctx.app_handle().clone();
        // File reads and core requests must not block the webview's thread.
        thread::spawn(move || responder.respond(respond(&app, &request)));
    })
}

fn respond<R: Runtime>(app: &AppHandle<R>, request: &HttpRequest<Vec<u8>>) -> Response<Vec<u8>> {
    let response = serve(app, request);
    // `OVERLAY_DEBUG=1` logs every mod file request (for bug reports).
    if std::env::var_os("OVERLAY_DEBUG").is_some() {
        eprintln!(
            "[overlay-ui] mod request {} {} -> {}",
            request.method(),
            request.uri(),
            response.status()
        );
    }
    response
}

fn serve<R: Runtime>(app: &AppHandle<R>, request: &HttpRequest<Vec<u8>>) -> Response<Vec<u8>> {
    let path = request.uri().path();
    if let Some(mod_id) = path.strip_prefix("/__report/") {
        report_blocked(app, mod_id, request.body());
        return status(204);
    }
    let Some((mod_id, file)) = path.trim_start_matches('/').split_once('/') else {
        return status(404);
    };
    let Some((root, network)) = find_mod(app, mod_id) else {
        return status(404);
    };
    let body = if file == ".overlay/locales.json" {
        locale_listing(&root).into_bytes()
    } else {
        match resolve_file(&root, file).and_then(|p| fs::read(p).ok()) {
            Some(body) => body,
            None => return status(404),
        }
    };
    Response::builder()
        .header("Content-Type", content_type(file))
        .header("Content-Security-Policy", csp(mod_id, &network))
        // Frames have an opaque origin; their module scripts and fetches of their own
        // files are cross-origin requests and need this.
        .header("Access-Control-Allow-Origin", "*")
        .header("X-Content-Type-Options", "nosniff")
        // Developer mode reloads must see rebuilt files.
        .header("Cache-Control", "no-store")
        .body(body)
        .unwrap_or_else(|_| status(500))
}

fn find_mod<R: Runtime>(app: &AppHandle<R>, mod_id: &str) -> Option<(PathBuf, Vec<String>)> {
    let map = app.state::<ModMap>();
    if let Some(found) = map.lookup(mod_id) {
        return Some(found);
    }
    // The page may load a mod before this process has seen the mod list.
    let client = app.state::<Arc<CoreClient>>();
    let list: ModList = serde_json::from_value(client.request(Request::ListMods).ok()?).ok()?;
    map.update(&list);
    map.lookup(mod_id)
}

pub fn csp(mod_id: &str, network: &[String]) -> String {
    let base = base_url();
    let hosts: String = network.iter().map(|h| format!(" https://{h}")).collect();
    format!(
        "default-src 'none'; script-src {base}; style-src {base} 'unsafe-inline'; \
         img-src {base} data: blob:{hosts}; font-src {base} data:; media-src {base}{hosts}; \
         connect-src {base}{hosts}; frame-src 'none'; worker-src 'none'; object-src 'none'; \
         base-uri 'none'; form-action 'none'; report-uri {base}/__report/{mod_id}"
    )
}

/// `{"en": ["main.ftl"], "de": ["main.ftl"]}` for the SDK, which cannot list folders.
fn locale_listing(root: &std::path::Path) -> String {
    let mut listing: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    if let Ok(locales) = fs::read_dir(root.join("locales")) {
        for locale in locales.flatten().filter(|e| e.path().is_dir()) {
            let Ok(files) = fs::read_dir(locale.path()) else {
                continue;
            };
            let mut names: Vec<String> = files
                .flatten()
                .filter_map(|f| f.file_name().into_string().ok())
                .filter(|n| n.ends_with(".ftl"))
                .collect();
            names.sort();
            listing.insert(locale.file_name().to_string_lossy().into_owned(), names);
        }
    }
    serde_json::to_string(&listing).unwrap_or_else(|_| "{}".into())
}

fn report_blocked<R: Runtime>(app: &AppHandle<R>, mod_id: &str, body: &[u8]) {
    let report: serde_json::Value = serde_json::from_slice(body).unwrap_or_default();
    let details = &report["csp-report"];
    let text = |key: &str| details[key].as_str().unwrap_or_default().to_owned();
    let request = Request::ReportBlocked {
        mod_id: mod_id.to_owned(),
        blocked_uri: text("blocked-uri"),
        directive: text("violated-directive"),
    };
    let _ = app.state::<Arc<CoreClient>>().request(request);
}

fn content_type(file: &str) -> &'static str {
    let extension = file
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "ftl" | "txt" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        _ => "application/octet-stream",
    }
}

fn status(code: u16) -> Response<Vec<u8>> {
    Response::builder()
        .status(code)
        .body(Vec::new())
        .expect("static response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csp_allows_only_declared_https_hosts() {
        let base = base_url();
        let policy = csp("org.example.hello", &["api.hearthstonejson.com".into()]);
        assert!(policy.contains(&format!(
            "connect-src {base} https://api.hearthstonejson.com;"
        )));
        assert!(policy.contains("default-src 'none'"));
        assert!(policy.contains(&format!("script-src {base};")));
        assert!(!policy.contains("unsafe-eval"));
        assert!(policy.ends_with(&format!("report-uri {base}/__report/org.example.hello")));
    }

    #[test]
    fn content_types() {
        assert_eq!(content_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(
            content_type("assets/app.JS"),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(content_type("unknown.bin"), "application/octet-stream");
    }
}
