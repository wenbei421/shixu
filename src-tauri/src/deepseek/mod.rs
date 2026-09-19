mod eval;
mod script;

use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use self::script::build_task_script;

const WINDOW_LABEL: &str = "deepseek-web";
const DEEPSEEK_URL: &str = "https://chat.deepseek.com/";
const DEFAULT_TIMEOUT_MS: u64 = 180_000;
const MAX_TIMEOUT_MS: u64 = 300_000;
const MIN_TIMEOUT_MS: u64 = 5_000;
const MAX_PROMPT_CHARS: usize = 100_000;

#[cfg(windows)]
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0";
#[cfg(target_os = "macos")]
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
#[cfg(not(any(windows, target_os = "macos")))]
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct DeepseekBridge {
    busy: Mutex<bool>,
}

impl DeepseekBridge {
    pub fn new() -> Self {
        Self {
            busy: Mutex::new(false),
        }
    }
}

struct BusyGuard<'a>(&'a Mutex<bool>);

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        match self.0.lock() {
            Ok(mut guard) => *guard = false,
            Err(poisoned) => *poisoned.into_inner() = false,
        }
    }
}

fn try_acquire(bridge: &DeepseekBridge) -> Option<BusyGuard<'_>> {
    let mut guard = match bridge.busy.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if *guard {
        return None;
    }
    *guard = true;
    drop(guard);
    Some(BusyGuard(&bridge.busy))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageState {
    task_id: String,
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    text: String,
    result: Option<String>,
    #[serde(default)]
    generating: bool,
    #[serde(default)]
    stable_count: u32,
    error: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteTaskResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub message: Option<String>,
    pub elapsed_ms: u64,
}

fn response(
    success: bool,
    result: Option<String>,
    error: Option<String>,
    message: Option<String>,
    started: Instant,
) -> ExecuteTaskResponse {
    ExecuteTaskResponse {
        success,
        result,
        error,
        message,
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    }
}

fn fail(error: &str, message: &str, started: Instant) -> Result<ExecuteTaskResponse, String> {
    Ok(response(
        false,
        None,
        Some(error.to_string()),
        Some(message.to_string()),
        started,
    ))
}

pub async fn open_window(app: &AppHandle) -> Result<(), String> {
    let window = ensure_window(app)?;
    reveal(&window)?;
    Ok(())
}

pub async fn execute(
    app: &AppHandle,
    bridge: &DeepseekBridge,
    prompt: String,
    timeout_ms: u64,
) -> Result<ExecuteTaskResponse, String> {
    let started = Instant::now();
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return fail("MISSING_PROMPT", "提示词不能为空", started);
    }
    if prompt.chars().count() > MAX_PROMPT_CHARS {
        return fail("PROMPT_TOO_LONG", "提示词不能超过 100000 个字符", started);
    }

    let Some(_guard) = try_acquire(bridge) else {
        return fail("TASK_RUNNING", "已有 DeepSeek 任务正在执行", started);
    };

    let timeout_ms = clamp_timeout(timeout_ms);
    let window = ensure_window(app)?;
    conceal(&window);
    ensure_chat_url(&window)?;

    let task_id = new_task_id();
    let script = build_task_script(&task_id, &prompt)?;
    let attached = match attach_script(app, &window, &script, &task_id).await {
        Ok(attached) => attached,
        Err(_) => {
            return fail("WINDOW_CLOSED", "DeepSeek 窗口已关闭", started);
        }
    };
    if !attached {
        conceal(&window);
        return fail(
            "PAGE_NOT_READY",
            "DeepSeek 网页还没准备好，请先打开窗口并完成登录",
            started,
        );
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let mut last_signature = String::new();
    let mut missing_state = 0u32;

    loop {
        if Instant::now() >= deadline {
            if let Err(err) = window
                .eval("window.__SHIXU_DEEPSEEK_CLEANUP__ && window.__SHIXU_DEEPSEEK_CLEANUP__()")
            {
                log::debug!("停止 DeepSeek 页面任务失败: {err}");
            }
            conceal(&window);
            return fail("TIMEOUT", "等待 DeepSeek 回复超时", started);
        }

        if app.get_webview_window(WINDOW_LABEL).is_none() {
            return fail("WINDOW_CLOSED", "DeepSeek 窗口已关闭", started);
        }

        match read_state(&window).await {
            Ok(Some(state)) if state.task_id == task_id => {
                let signature = format!(
                    "{}:{}:{}:{}",
                    state.event_type, state.generating, state.stable_count, state.text
                );
                if signature != last_signature {
                    last_signature = signature;
                    emit_progress(app, &state);
                }

                if state.event_type == "completed" {
                    let text = match state.result {
                        Some(result) => result,
                        None => state.text,
                    };
                    if text.trim().is_empty() {
                        conceal(&window);
                        return fail("EMPTY_RESULT", "DeepSeek 没有返回内容", started);
                    }
                    conceal(&window);
                    return Ok(response(true, Some(text), None, state.message, started));
                }

                if state.event_type == "error" {
                    if state.error.as_deref() == Some("LOGIN_REQUIRED") {
                        if let Err(err) = reveal(&window) {
                            log::debug!("登录时显示 DeepSeek 窗口失败: {err}");
                        }
                    } else {
                        conceal(&window);
                    }
                    return Ok(response(false, None, state.error, state.message, started));
                }
                missing_state = 0;
            }
            Ok(_) => {
                missing_state = missing_state.saturating_add(1);
                if missing_state == 3 || missing_state == 8 {
                    if let Err(err) = window.eval(script.clone()) {
                        log::debug!("重新注入 DeepSeek 脚本失败: {err}");
                    }
                }
            }
            Err(err) => {
                log::debug!("读取 DeepSeek 页面状态失败: {err}");
            }
        }

        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

fn clamp_timeout(timeout_ms: u64) -> u64 {
    if timeout_ms < MIN_TIMEOUT_MS {
        DEFAULT_TIMEOUT_MS
    } else if timeout_ms > MAX_TIMEOUT_MS {
        MAX_TIMEOUT_MS
    } else {
        timeout_ms
    }
}

fn new_task_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("ds-{nanos}")
}

fn ensure_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        return Ok(window);
    }

    let url = DEEPSEEK_URL
        .parse()
        .map_err(|err| format!("DeepSeek URL 解析失败: {err}"))?;

    WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::External(url))
        .title("DeepSeek")
        .inner_size(1100.0, 800.0)
        .min_inner_size(720.0, 560.0)
        .resizable(true)
        .center()
        .focused(false)
        .visible(false)
        .user_agent(USER_AGENT)
        .build()
        .map_err(|err| format!("创建 DeepSeek 窗口失败: {err}"))
}

fn reveal(window: &WebviewWindow) -> Result<(), String> {
    window
        .unminimize()
        .map_err(|err| format!("还原 DeepSeek 窗口失败: {err}"))?;
    window
        .show()
        .map_err(|err| format!("显示 DeepSeek 窗口失败: {err}"))?;
    window
        .set_focus()
        .map_err(|err| format!("聚焦 DeepSeek 窗口失败: {err}"))?;
    Ok(())
}

fn conceal(window: &WebviewWindow) {
    if let Err(err) = window.hide() {
        log::debug!("隐藏 DeepSeek 窗口失败: {err}");
    }
}

fn ensure_chat_url(window: &WebviewWindow) -> Result<(), String> {
    let url = window
        .url()
        .map_err(|err| format!("读取 DeepSeek 地址失败: {err}"))?;
    let current = url.as_str();
    if current.contains("chat.deepseek.com") {
        return Ok(());
    }
    let url = DEEPSEEK_URL
        .parse()
        .map_err(|err| format!("DeepSeek URL 解析失败: {err}"))?;
    window
        .navigate(url)
        .map_err(|err| format!("打开 DeepSeek 网页失败: {err}"))
}

async fn attach_script(
    app: &AppHandle,
    window: &WebviewWindow,
    script: &str,
    task_id: &str,
) -> Result<bool, String> {
    for _attempt in 0..20 {
        if app.get_webview_window(WINDOW_LABEL).is_none() {
            return Err("DeepSeek 窗口已关闭".to_string());
        }
        if let Err(err) = window.eval(script.to_string()) {
            log::debug!("注入 DeepSeek 脚本失败: {err}");
        }
        tokio::time::sleep(Duration::from_millis(700)).await;
        if let Ok(Some(state)) = read_state(window).await {
            if state.task_id == task_id {
                emit_progress(app, &state);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

async fn read_state(window: &WebviewWindow) -> Result<Option<PageState>, String> {
    let value = eval::eval_json(window, "window.__SHIXU_DEEPSEEK__ ?? null").await?;
    state_from_value(value)
}

fn state_from_value(value: Value) -> Result<Option<PageState>, String> {
    let value = match value {
        Value::Null => return Ok(None),
        Value::String(inner) => match serde_json::from_str::<Value>(&inner) {
            Ok(Value::Null) | Err(_) => return Ok(None),
            Ok(parsed) => parsed,
        },
        other => other,
    };
    if value.is_null() {
        return Ok(None);
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(|err| format!("解析 DeepSeek 状态失败: {err}"))
}

fn emit_progress(app: &AppHandle, state: &PageState) {
    if let Err(err) = app.emit_to("main", "deepseek-task-progress", state) {
        log::debug!("发送 DeepSeek 进度失败: {err}");
    }
}
