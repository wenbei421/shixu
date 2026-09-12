use tauri::plugin::TauriPlugin;

pub fn tauri_plugin_logging() -> TauriPlugin<tauri::Wry> {
    tauri_plugin_log::Builder::new()
        .clear_targets()
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
        .level(log::LevelFilter::Info)
        .target(
            #[cfg(debug_assertions)]
            dev_logging_target(),
            #[cfg(not(debug_assertions))]
            prod_logging_target(),
        )
        .build()
}

#[allow(dead_code)]
fn dev_logging_target() -> tauri_plugin_log::Target {
    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
}

#[allow(dead_code)]
fn prod_logging_target() -> tauri_plugin_log::Target {
    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
        file_name: Some("logs".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_logging_target_creates_stdout_target() {
        let _target = dev_logging_target();
    }

    #[test]
    fn prod_logging_target_creates_logdir_target() {
        let _target = prod_logging_target();
    }
}
