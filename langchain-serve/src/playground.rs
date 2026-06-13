//! Playground UI — serves an interactive HTML playground at GET /playground.

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::server::AppState;

type SharedState = Arc<RwLock<AppState>>;

pub async fn playground(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let chains = state.read().await.chains.clone();
    let chain_options: Vec<String> = chains
        .keys()
        .map(|k| format!(r#"<option value="{}">{}</option>"#, k, k))
        .collect();
    let chain_options_html = chain_options.join("\n");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>LangServe Playground</title>
<style>
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f5; padding: 20px; }}
  .container {{ max-width: 800px; margin: 0 auto; }}
  h1 {{ text-align: center; margin-bottom: 20px; color: #333; }}
  .panel {{ background: #fff; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); padding: 20px; margin-bottom: 20px; }}
  label {{ display: block; font-weight: 600; margin-bottom: 6px; color: #555; }}
  select, textarea, input[type=text] {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px; margin-bottom: 12px; }}
  textarea {{ resize: vertical; min-height: 80px; }}
  button {{ background: #4f46e5; color: #fff; border: none; padding: 10px 24px; border-radius: 4px; font-size: 14px; cursor: pointer; margin-right: 8px; }}
  button:hover {{ background: #4338ca; }}
  button:disabled {{ background: #a5a5a5; cursor: not-allowed; }}
  button.stream {{ background: #0d9488; }}
  button.stream:hover {{ background: #0f766e; }}
  .response {{ background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 4px; padding: 16px; min-height: 60px; white-space: pre-wrap; font-family: monospace; font-size: 13px; max-height: 400px; overflow-y: auto; }}
  .error {{ color: #dc2626; }}
  .status {{ font-size: 12px; color: #888; margin-top: 8px; }}
  .tabs {{ display: flex; gap: 0; margin-bottom: 12px; }}
  .tab {{ padding: 8px 16px; background: #e5e7eb; border: none; cursor: pointer; font-size: 13px; }}
  .tab:first-child {{ border-radius: 4px 0 0 4px; }}
  .tab:last-child {{ border-radius: 0 4px 4px 0; }}
  .tab.active {{ background: #4f46e5; color: #fff; }}
</style>
</head>
<body>
<div class="container">
  <h1>LangServe Playground</h1>
  <div class="panel">
    <label for="chain">Chain</label>
    <select id="chain">
      {chain_options_html}
    </select>
    <div class="tabs">
      <button class="tab active" data-mode="invoke" onclick="setMode(this,'invoke')">Invoke</button>
      <button class="tab" data-mode="stream" onclick="setMode(this,'stream')">Stream</button>
      <button class="tab" data-mode="batch" onclick="setMode(this,'batch')">Batch</button>
    </div>
    <label for="input">Input (JSON)</label>
    <textarea id="input">{{"query": "Hello"}}</textarea>
    <div id="batch-extra" style="display:none;">
      <label for="input2">Second Input (JSON)</label>
      <input type="text" id="input2" value='{{"query": "World"}}' />
    </div>
    <button id="sendBtn" onclick="sendRequest()">Send</button>
    <span class="status" id="status"></span>
    <label style="margin-top:16px;">Response</label>
    <div class="response" id="response">Response will appear here</div>
  </div>
</div>
<script>
let currentMode = 'invoke';
function setMode(el, mode) {{
  currentMode = mode;
  document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  el.classList.add('active');
  document.getElementById('batch-extra').style.display = mode === 'batch' ? 'block' : 'none';
}}
async function sendRequest() {{
  const chain = document.getElementById('chain').value;
  const inputVal = document.getElementById('input').value;
  const respEl = document.getElementById('response');
  const statusEl = document.getElementById('status');
  const btn = document.getElementById('sendBtn');
  btn.disabled = true;
  statusEl.textContent = 'Sending...';
  respEl.textContent = '';
  respEl.classList.remove('error');
  try {{
    let parsed;
    try {{ parsed = JSON.parse(inputVal); }} catch(e) {{ throw new Error('Invalid JSON input: ' + e.message); }}
    if (currentMode === 'invoke') {{
      const res = await fetch('/' + chain + '/invoke', {{
        method: 'POST', headers: {{'Content-Type': 'application/json'}},
        body: JSON.stringify({{input: parsed}})
      }});
      const data = await res.json();
      respEl.textContent = JSON.stringify(data, null, 2);
    }} else if (currentMode === 'stream') {{
      const res = await fetch('/' + chain + '/stream', {{
        method: 'POST', headers: {{'Content-Type': 'application/json'}},
        body: JSON.stringify({{input: parsed}})
      }});
      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let text = '';
      while (true) {{
        const {{done, value}} = await reader.read();
        if (done) break;
        text += decoder.decode(value, {{stream: true}});
        respEl.textContent = text;
      }}
    }} else if (currentMode === 'batch') {{
      const input2Val = document.getElementById('input2').value;
      let parsed2;
      try {{ parsed2 = JSON.parse(input2Val); }} catch(e) {{ throw new Error('Invalid JSON for second input: ' + e.message); }}
      const res = await fetch('/' + chain + '/batch', {{
        method: 'POST', headers: {{'Content-Type': 'application/json'}},
        body: JSON.stringify({{inputs: [parsed, parsed2]}})
      }});
      const data = await res.json();
      respEl.textContent = JSON.stringify(data, null, 2);
    }}
    statusEl.textContent = 'Done';
  }} catch (err) {{
    respEl.textContent = err.message;
    respEl.classList.add('error');
    statusEl.textContent = 'Error';
  }} finally {{
    btn.disabled = false;
  }}
}}
</script>
</body>
</html>"#
    );

    (StatusCode::OK, Html(html))
}
