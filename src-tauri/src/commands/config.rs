use tauri::Manager;

#[tauri::command]
pub fn read_default_directory(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
  let mut path = match app_handle.path().app_config_dir() { Ok(p) => p, Err(_) => return Ok(None) };
  path.push("config.json");
  if !path.exists() { return Ok(None) }
  let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
  let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
  Ok(v.get("default_directory").and_then(|x| x.as_str()).map(|s| s.to_string()))
}