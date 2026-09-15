#[cfg(not(debug_assertions))]
use plugins::logging;

use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

pub mod commands;
mod error;
mod id;
pub mod plugins;
mod sqlite;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        if let Err(err) = fix_path_env::fix() {
            log::warn!("fix-path-env failed: {err}; falling back to well-known dirs");
        }
    }
    let mut builder =
        tauri::Builder::default().plugin(tauri_plugin_single_instance::init(|_, _, _| {}));

    // CrabNebula DevTools prevents other logging plugins from working
    // https://docs.crabnebula.dev/devtools/troubleshoot/log-plugins/
    #[cfg(debug_assertions)]
    {
        let devtools = tauri_plugin_devtools::init();
        builder = builder.plugin(devtools);
        builder = builder.plugin(tauri_plugin_pilot::init());
    }

    #[cfg(not(debug_assertions))]
    {
        builder = builder.plugin(logging::tauri_plugin_logging());
    }
    builder
        .plugin(tauri_plugin_decorum::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_db_status,
            commands::list_inspiration_categories,
            commands::create_inspiration_category,
            commands::update_inspiration_category,
            commands::delete_inspiration_category,
            commands::list_inspiration_tags,
            commands::create_inspiration_tag,
            commands::update_inspiration_tag,
            commands::delete_inspiration_tag,
            commands::list_inspirations,
            commands::get_inspiration,
            commands::create_inspiration,
            commands::update_inspiration,
            commands::delete_inspiration,
        ])
        .setup(|app| {
            sqlite::set_db(app).map_err(|e| e.to_string())?;

            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();

            #[cfg(target_os = "macos")]
            {
                main_window.set_traffic_lights_inset(12.0, 16.0).unwrap();
                main_window.make_transparent().unwrap();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
