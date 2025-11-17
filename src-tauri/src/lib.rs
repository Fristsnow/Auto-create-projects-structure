use tauri::Manager;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // 注册对话框插件（用于目录选择等）
      app.handle().plugin(tauri_plugin_dialog::init())?;
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::env::check_environment,
      commands::env::get_system_paths,
      commands::project::check_target_dir,
      commands::project::create_project,
      commands::project::create_project_async,
      commands::config::read_default_directory,
      commands::registry::read_component_registry,
      commands::registry::save_component_registry,
      commands::registry::fetch_npm_versions
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
