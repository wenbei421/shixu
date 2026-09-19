use std::time::Duration;

use serde_json::Value;
use tauri::WebviewWindow;

/// 在 DeepSeek 窗口里执行脚本并取回 JSON。
///
/// 不让网页调用 Tauri command。远程页面一旦拿到 invoke，就得给它单独开权限；
/// 那会把整个应用的命令 ACL 打开。这里改成由 Rust 主动读取页面上的状态对象。
pub async fn eval_json(window: &WebviewWindow, script: &str) -> Result<Value, String> {
    #[cfg(windows)]
    {
        windows_eval(window, script).await
    }
    #[cfg(not(windows))]
    {
        let _ = (window, script);
        Err("UNSUPPORTED".to_string())
    }
}

#[cfg(windows)]
async fn windows_eval(window: &WebviewWindow, script: &str) -> Result<Value, String> {
    use std::sync::{Arc, Mutex};

    use webview2_com::ExecuteScriptCompletedHandler;
    use windows::core::HSTRING;

    let (tx, rx) = tokio::sync::oneshot::channel();
    let slot = Arc::new(Mutex::new(Some(tx)));
    let slot_for_webview = Arc::clone(&slot);
    let script = script.to_string();

    let dispatched = window.with_webview(move |webview| {
        let controller = webview.controller();
        // SAFETY: CoreWebView2 / ExecuteScript are the WebView2 APIs for this window.
        // The completion handler is referenced by WebView2 until the script finishes.
        let core = match unsafe { controller.CoreWebView2() } {
            Ok(core) => core,
            Err(err) => {
                send_result(&slot_for_webview, Err(format!("读取 WebView2 失败: {err}")));
                return;
            }
        };
        let js = HSTRING::from(script);
        let slot_for_callback = Arc::clone(&slot_for_webview);
        let handler = ExecuteScriptCompletedHandler::create(Box::new(move |_error, result| {
            send_result(&slot_for_callback, Ok(result));
            Ok(())
        }));
        if let Err(err) = unsafe { core.ExecuteScript(&js, &handler) } {
            send_result(&slot_for_webview, Err(format!("执行页面脚本失败: {err}")));
        }
    });

    if let Err(err) = dispatched {
        send_result(&slot, Err(format!("无法访问 DeepSeek 窗口: {err}")));
    }

    let raw = match tokio::time::timeout(Duration::from_secs(8), rx).await {
        Ok(Ok(Ok(raw))) => raw,
        Ok(Ok(Err(err))) => return Err(err),
        Ok(Err(_)) => return Err("页面脚本结果通道已关闭".to_string()),
        Err(_) => return Err("读取 DeepSeek 页面超时".to_string()),
    };

    if raw.is_empty() || raw == "null" {
        return Ok(Value::Null);
    }

    match serde_json::from_str(&raw) {
        Ok(value) => Ok(value),
        Err(err) => Err(format!("解析页面结果失败: {err}")),
    }
}

#[cfg(windows)]
fn send_result(
    slot: &std::sync::Mutex<Option<tokio::sync::oneshot::Sender<Result<String, String>>>>,
    value: Result<String, String>,
) {
    let sender = match slot.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    if let Some(sender) = sender {
        match sender.send(value) {
            Ok(()) => {}
            Err(_) => log::debug!("DeepSeek 页面结果已经没有接收方"),
        }
    }
}
