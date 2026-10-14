const { invoke } = window.__TAURI__.core;
const log = (msg) => invoke('report', { msg: String(msg) });
let answered = false;
const base = navigator.userAgent.includes('Windows') ? 'http://mod.localhost' : 'mod://localhost';
window.addEventListener('message', (e) => {
  log(
    `host got postMessage from origin=${e.origin} data=${JSON.stringify(e.data)} sourceIsFrame=${e.source === frame.contentWindow}`,
  );
  // Answer only the first message, so the reply does not start a ping-pong loop.
  if (!answered) e.source.postMessage({ reply: 'pong' }, '*');
  answered = true;
});
const frame = document.createElement('iframe');
frame.sandbox = 'allow-scripts';
frame.src = `${base}/hello/index.html`;
document.body.append(frame);
setTimeout(() => log('DONE'), 4000);
