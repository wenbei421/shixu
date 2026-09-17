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
            commands::list_muse_notes,
            commands::get_muse_note,
            commands::get_muse_note_counts,
            commands::list_muse_tags,
            commands::list_muse_projects,
            commands::list_muse_related_notes,
            commands::create_muse_note,
            commands::update_muse_note_content,
            commands::update_muse_note_status,
            commands::update_muse_note_project,
            commands::add_muse_tag_to_note,
            commands::remove_muse_tag_from_note,
            commands::archive_muse_note,
            commands::unarchive_muse_note,
            commands::delete_muse_note,
            commands::update_muse_note_source,
            commands::list_muse_trash,
            commands::restore_muse_note,
            commands::purge_muse_note,
            commands::empty_muse_trash,
            commands::purge_expired_muse_notes,
            commands::rename_muse_tag,
            commands::update_muse_tag_color,
            commands::merge_muse_tags,
            commands::delete_muse_tag,
            commands::create_muse_project,
            commands::update_muse_project,
            commands::delete_muse_project,
            commands::backup_muse_db,
            commands::auto_backup_muse_db,
            commands::list_muse_backups,
            commands::restore_muse_db,
            commands::restart_app,
        ])
        .setup(|app| {
            sqlite::set_db(app).map_err(|e| e.to_string())?;

            // 启动时异步：自动备份 + 回收站 30 天清理（不阻塞首屏）
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::run_muse_startup_maintenance(handle).await {
                    log::warn!("Muse startup maintenance failed: {e}");
                }
            });

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
