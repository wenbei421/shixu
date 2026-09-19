use tauri::{AppHandle, State};

use crate::deepseek::{self, DeepseekBridge, ExecuteTaskResponse};

#[tauri::command]
pub async fn open_deepseek_window(app: AppHandle) -> Result<(), String> {
    deepseek::open_window(&app).await
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteTaskRequest {
    pub prompt: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    180_000
}

#[tauri::command]
pub async fn execute_deepseek_task(
    app: AppHandle,
    bridge: State<'_, DeepseekBridge>,
    request: ExecuteTaskRequest,
) -> Result<ExecuteTaskResponse, String> {
    deepseek::execute(&app, bridge.inner(), request.prompt, request.timeout_ms).await
}
