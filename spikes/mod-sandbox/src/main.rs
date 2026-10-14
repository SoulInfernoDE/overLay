//! Prints what the sandboxed mod frame could and could not do.

use tauri::http::Response;

#[tauri::command]
fn report(msg: String) {
    eprintln!("[spike] {msg}");
    if msg == "DONE" {
        std::process::exit(0);
    }
}

const MOD_HTML: &str = r#"<!doctype html><html><head><meta charset="utf-8"><script src="mod.js"></script></head><body>mod</body></html>"#;

const MOD_JS: &str = r#"
const send = (m) => parent.postMessage({ m }, '*');
send('script runs, origin=' + self.origin);
try { send('parent DOM: ' + parent.document.getElementById('secret').textContent); } catch (e) { send('parent DOM blocked: ' + e.name); }
try { localStorage.setItem('x', '1'); send('localStorage works'); } catch (e) { send('localStorage blocked: ' + e.name); }
try { eval('1+1'); send('eval allowed'); } catch (e) { send('eval blocked: ' + e.name); }
document.addEventListener('securitypolicyviolation', (e) => send('CSP violation event: ' + e.blockedURI + ' (' + e.violatedDirective + ')'));
fetch('https://api.hearthstonejson.com/v1/').then((r) => send('fetch declared domain status ' + r.status)).catch((e) => send('fetch declared domain failed: ' + e));
fetch('https://example.com/').then((r) => send('fetch UNDECLARED domain status ' + r.status)).catch((e) => send('fetch undeclared domain blocked: ' + e));
fetch('locales/en.json').then((r) => r.text()).then((t) => send('own file via relative URL: ' + t)).catch((e) => send('own file failed: ' + e));
window.addEventListener('message', (e) => send('mod got reply ' + JSON.stringify(e.data)));
"#;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![report])
        .register_uri_scheme_protocol("mod", |_ctx, request| {
            let path = request.uri().path().to_owned();
            eprintln!("[spike] mod protocol request: {} {}", request.method(), request.uri());
            let base = if cfg!(windows) { "http://mod.localhost" } else { "mod://localhost" };
            if path.starts_with("/__report") {
                eprintln!("[spike] CSP REPORT: {}", String::from_utf8_lossy(request.body()));
                return Response::builder().status(204).body(Vec::new()).unwrap();
            }
            let (body, mime) = match path.as_str() {
                "/hello/index.html" => (MOD_HTML.as_bytes().to_vec(), "text/html"),
                "/hello/mod.js" => (MOD_JS.as_bytes().to_vec(), "text/javascript"),
                "/hello/locales/en.json" => (br#"{"hi":"hello"}"#.to_vec(), "application/json"),
                _ => return Response::builder().status(404).body(Vec::new()).unwrap(),
            };
            let csp = format!(
                "default-src 'none'; script-src {base}; style-src {base} 'unsafe-inline'; \
                 connect-src {base} https://api.hearthstonejson.com; img-src {base} data:; \
                 frame-src 'none'; base-uri 'none'; form-action 'none'; report-uri {base}/__report/hello"
            );
            Response::builder()
                .header("Content-Type", mime)
                .header("Content-Security-Policy", csp)
                .header("Access-Control-Allow-Origin", "*")
                .body(body)
                .unwrap()
        })
        .run(tauri::generate_context!())
        .expect("spike failed");
}
