use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use color_eyre::eyre::{eyre, Result, WrapErr};
use serde_json::{json, Value};

type BrowserRunner = dyn Fn(&[String]) -> Result<Value> + Send + Sync;

#[derive(Clone)]
pub struct AgentBrowserClient {
    session: Option<String>,
    runner: Arc<BrowserRunner>,
}

impl AgentBrowserClient {
    pub fn new(session: Option<String>) -> Self {
        Self {
            session,
            runner: Arc::new(default_runner),
        }
    }

    pub fn with_runner(session: Option<String>, runner: Arc<BrowserRunner>) -> Self {
        Self { session, runner }
    }

    pub fn current_url(&self) -> Result<String> {
        let response = self.run_json(&["get", "url"])?;
        Ok(response
            .pointer("/data/url")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string())
    }

    pub fn open_url(&self, url: &str) -> Result<()> {
        self.run_json(&["open", url]).map(|_| ())
    }

    pub fn evaluate(&self, script: &str) -> Result<Value> {
        let response = self.run_json(&["eval", script])?;
        Ok(response
            .pointer("/data/result")
            .cloned()
            .unwrap_or(Value::Null))
    }

    pub fn wait(&self, milliseconds: u64) -> Result<()> {
        self.run_json(&["wait", &milliseconds.to_string()])
            .map(|_| ())
    }

    pub fn set_input_value(&self, selector: &str, value: &str) -> Result<()> {
        let script = format!(
            "(() => {{\
            const selector = {selector:?};\
            const value = {value:?};\
            const element = document.querySelector(selector);\
            if (!(element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement)) {{\
              throw new Error(`Input not found for selector: ${{selector}}`);\
            }}\
            const prototype = Object.getPrototypeOf(element);\
            const descriptor = Object.getOwnPropertyDescriptor(prototype, 'value')\
              || Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')\
              || Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value');\
            if (!descriptor || typeof descriptor.set !== 'function') {{\
              throw new Error(`Value setter unavailable for selector: ${{selector}}`);\
            }}\
            element.focus();\
            descriptor.set.call(element, value);\
            element.dispatchEvent(new Event('input', {{ bubbles: true }}));\
            element.dispatchEvent(new Event('change', {{ bubbles: true }}));\
            element.blur();\
            return {{ valueLength: element.value.length }};\
            }})()"
        );
        self.evaluate(&script).map(|_| ())
    }

    fn run_json(&self, args: &[&str]) -> Result<Value> {
        let mut command = vec![String::from("agent-browser")];
        if let Some(session) = &self.session {
            command.push(String::from("--session"));
            command.push(session.clone());
        }
        command.push(String::from("--json"));
        command.extend(args.iter().map(|value| value.to_string()));
        (self.runner)(&command)
    }
}

fn default_runner(command: &[String]) -> Result<Value> {
    let executable = command
        .first()
        .ok_or_else(|| eyre!("agent-browser command is empty"))?;
    let mut process = Command::new(executable);
    for arg in &command[1..] {
        process.arg(arg);
    }
    let output = process
        .output()
        .wrap_err("failed to invoke agent-browser subprocess")?;
    if !output.status.success() {
        return Err(eyre!(
            "{}",
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        ));
    }
    let payload: Value = serde_json::from_slice(&output.stdout)
        .wrap_err("failed to decode agent-browser JSON response")?;
    if !payload
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(eyre!(
            "{}",
            payload
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("agent-browser request failed")
        ));
    }
    Ok(payload)
}

pub fn build_browser_action_capture(
    page: &str,
    action: &str,
    target: &str,
    status: &str,
    captured_at: &str,
    url: &str,
    document_title: &str,
    body_text: &str,
    interactive_snapshot: Vec<Value>,
    links: Vec<String>,
    inputs: Value,
    visible_actions: Vec<String>,
    resource_hosts: Vec<String>,
    local_storage_keys: Vec<String>,
    notes: Vec<String>,
    metadata: Value,
) -> Value {
    json!({
        "captured_at": captured_at,
        "page": page,
        "kind": "action_snapshot",
        "action": action,
        "target": target,
        "status": status,
        "url": url,
        "document_title": document_title,
        "body_text": body_text,
        "interactive_snapshot": interactive_snapshot,
        "links": links,
        "inputs": inputs,
        "visible_actions": visible_actions,
        "resource_hosts": resource_hosts,
        "local_storage_keys": local_storage_keys,
        "notes": notes,
        "metadata": metadata,
    })
}

pub fn capture_action_snapshot(
    client: &AgentBrowserClient,
    page: &str,
    action: &str,
    target: &str,
    status: &str,
    captured_at: &str,
    notes: Vec<String>,
    metadata: Value,
) -> Result<Value> {
    let url = client.current_url()?;
    let document_title = client
        .evaluate("document.title || ''")?
        .as_str()
        .unwrap_or_default()
        .to_string();
    let body_text = client
        .evaluate("document.body?.innerText ?? ''")?
        .as_str()
        .unwrap_or_default()
        .to_string();
    let interactive_snapshot = client
        .evaluate(
            "(() => Object.entries((() => { const refs = {}; return refs; })()).map(([ref, payload]) => ({ ref, ...payload })))()",
        )?
        .as_array()
        .cloned()
        .unwrap_or_default();
    let links = client
        .evaluate("Array.from(document.querySelectorAll('a[href]')).map((anchor) => anchor.href)")?
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let inputs = client.evaluate(
        "Object.fromEntries(Array.from(document.querySelectorAll('input, textarea, select')).map((element, index) => { const key = element.name || element.id || element.getAttribute('placeholder') || `input_${index}`; return [key, element.value ?? '']; }))",
    )?;
    let visible_actions = client
        .evaluate("Array.from(document.querySelectorAll('a, button, [role=\"button\"]')).map((element) => (element.innerText || element.textContent || '').trim()).filter(Boolean)")?
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let resource_hosts = client
        .evaluate("Array.from(new Set(performance.getEntriesByType('resource').map((entry) => { try { return new URL(entry.name).hostname; } catch { return null; } }).filter(Boolean)))")?
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let local_storage_keys = client
        .evaluate("Object.keys(window.localStorage || {})")?
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    Ok(build_browser_action_capture(
        page,
        action,
        target,
        status,
        captured_at,
        &url,
        &document_title,
        &body_text,
        interactive_snapshot,
        links,
        inputs,
        resource_hosts,
        local_storage_keys,
        notes,
        visible_actions,
        metadata,
    ))
}

pub fn screenshot_path_for(run_dir: &Path, page: &str, captured_at: &str) -> String {
    let normalized = captured_at.replace(':', "-");
    run_dir
        .join("screenshots")
        .join(format!("{page}-{normalized}.png"))
        .display()
        .to_string()
}
