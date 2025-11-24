use tauri::Manager;

fn menu_path(app_handle: &tauri::AppHandle) -> Option<std::path::PathBuf> {
  app_handle.path().app_config_dir().ok().map(|mut p| { p.push("tool_menu.json"); p })
}

#[tauri::command]
pub fn read_tool_menu(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
  let path = menu_path(&app_handle).ok_or_else(|| "config dir not available".to_string())?;
  if !path.exists() {
    let default = serde_json::json!({
      "visible": true,
      "tools": [
        {"key":"vue","label":"Vue","route":"/tool/vue","enabled":true},
        {"key":"java","label":"Java","route":"/tool/java","enabled":true},
        {"key":"node","label":"Node","route":"/tool/node","enabled":true},
        {"key":"laravel","label":"Laravel","route":"/tool/laravel","enabled":true}
      ]
    });
    if let Some(dir) = path.parent() { std::fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
    std::fs::write(&path, default.to_string()).map_err(|e| e.to_string())?;
    return Ok(default);
  }
  let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
  let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
  Ok(v)
}

#[tauri::command]
pub fn save_tool_menu(app_handle: tauri::AppHandle, payload: serde_json::Value) -> Result<(), String> {
  let path = menu_path(&app_handle).ok_or_else(|| "config dir not available".to_string())?;
  if let Some(dir) = path.parent() { std::fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
  std::fs::write(&path, payload.to_string()).map_err(|e| e.to_string())?;
  Ok(())
}
