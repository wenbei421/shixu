const TEMPLATE: &str = include_str!("task.js");

/// 把任务号和提示词以 JSON 字面量写进页面脚本，避免引号和换行把脚本截断。
pub fn build_task_script(task_id: &str, prompt: &str) -> Result<String, String> {
    let task_id_json =
        serde_json::to_string(task_id).map_err(|err| format!("task_id 序列化失败: {err}"))?;
    let prompt_json =
        serde_json::to_string(prompt).map_err(|err| format!("prompt 序列化失败: {err}"))?;

    Ok(TEMPLATE
        .replace("__SHIXU_TASK_ID_JSON__", &task_id_json)
        .replace("__SHIXU_PROMPT_JSON__", &prompt_json))
}

#[cfg(test)]
mod tests {
    use super::build_task_script;

    #[test]
    fn embeds_prompt_as_json() {
        let script = match build_task_script("task-1", "说\"你好\"\n下一行") {
            Ok(script) => script,
            Err(err) => panic!("script: {err}"),
        };
        assert!(script.contains("\"task-1\""));
        assert!(script.contains("说\\\"你好\\\"\\n下一行"));
        assert!(!script.contains("__SHIXU_TASK_ID_JSON__"));
        assert!(!script.contains("__SHIXU_PROMPT_JSON__"));
    }
}
