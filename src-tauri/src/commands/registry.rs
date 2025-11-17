use tauri::Manager;
use tauri::async_runtime;
use tokio::time::{timeout, Duration};

fn registry_path(app_handle: &tauri::AppHandle) -> Option<std::path::PathBuf> {
  app_handle.path().app_config_dir().ok().map(|mut p| { p.push("components.json"); p })
}

#[tauri::command]
pub fn read_component_registry(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
  let path = registry_path(&app_handle).ok_or_else(|| "config dir not available".to_string())?;
  if !path.exists() {
    // 默认内容
    let default = serde_json::json!({
      "components": [
        {
          "key": "router",
          "label": "vue-router",
          "packages": ["vue-router"],
          "desc": "路由管理",
          "versions": { "vue2": "^3.x", "vue3": "^4.6.3" },
          "supported": { "vue2": true, "vue3": true, "ts": true, "js": true },
          "dev": false
        },
        {
          "key": "pinia",
          "label": "pinia",
          "packages": ["pinia", "pinia-plugin-persistedstate"],
          "desc": "轻量状态管理",
          "versions": { "vue3": "^2.3.1" },
          "supported": { "vue2": true, "vue3": true, "ts": true, "js": true },
          "dev": false
        },
        {
          "key": "sass",
          "label": "sass",
          "packages": ["sass", "sass-loader"],
          "desc": "CSS 预处理器",
          "versions": { "vue2": "^1.94.0", "vue3": "^1.94.0" },
          "supported": { "vue2": true, "vue3": true, "ts": true, "js": true },
          "dev": true
        },
        {
          "key": "naive-ui",
          "label": "naive-ui",
          "packages": ["naive-ui"],
          "desc": "Vue 3 组件库",
          "supported": { "vue2": false, "vue3": true, "ts": true, "js": false }
        },
        {
          "key": "vfonts",
          "label": "vfonts",
          "packages": ["vfonts"],
          "desc": "网页与代码字体",
          "supported": { "vue2": true, "vue3": true, "ts": true, "js": true }
        },
        {
          "key": "xicons",
          "label": "@vicons/ionicons5",
          "packages": ["@vicons/ionicons5"],
          "desc": "图标库",
          "supported": { "vue2": true, "vue3": true, "ts": true, "js": true }
        }
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
pub fn save_component_registry(app_handle: tauri::AppHandle, payload: serde_json::Value) -> Result<(), String> {
  let path = registry_path(&app_handle).ok_or_else(|| "config dir not available".to_string())?;
  if let Some(dir) = path.parent() { std::fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
  std::fs::write(&path, payload.to_string()).map_err(|e| e.to_string())?;
  Ok(())
}

// 通过 pnpm 查询 npm 包所有版本（动态）
#[tauri::command]
pub async fn fetch_npm_versions(package: String) -> Result<Vec<String>, String> {
  use std::process::Command;

  #[allow(unused_variables)]
  fn resolve_pnpm() -> Option<String> {
    #[cfg(windows)]
    {
      let try_bin = |bin: &str| -> bool {
        Command::new(bin).arg("--version").output().ok().is_some()
      };
      for c in ["pnpm", "pnpm.cmd", "pnpm.exe"] { if try_bin(c) { return Some(c.to_string()); } }
      if let Ok(out) = Command::new("where").arg("pnpm").output() {
        if let Ok(s) = String::from_utf8(out.stdout) {
          if let Some(path) = s.lines().next() { let p = path.trim().to_string(); if try_bin(&p) { return Some(p); } }
        }
      }
      None
    }
    #[cfg(not(windows))]
    { Some("pnpm".to_string()) }
  }

  let pnpm_bin = resolve_pnpm().ok_or_else(|| "pnpm not found".to_string())?;

  // 将阻塞的子进程调用放到阻塞线程池，并加超时避免卡住
  let pkg = package.clone();
  let bin = pnpm_bin.clone();
  let handle = async_runtime::spawn_blocking(move || {
    Command::new(&bin)
      .args(["view", &pkg, "versions", "--json"])
      .output()
  });

  // 先处理超时与 JoinError，再处理子进程的 io::Error
  let output_res = timeout(Duration::from_secs(8), handle)
    .await
    .map_err(|_| "查询 npm 版本超时（8s）".to_string())?
    .map_err(|e| format!("执行 pnpm 失败: {e}"))?; // JoinError
  let output = output_res.map_err(|e| format!("执行 pnpm 失败: {e}"))?; // io::Error

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("查询版本失败：{}", stderr.trim()));
  }

  let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
  let mut versions: Vec<String> = match serde_json::from_str::<serde_json::Value>(&stdout) {
    Ok(serde_json::Value::Array(arr)) => arr.into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
    Ok(_) => Vec::new(),
    Err(_) => Vec::new(),
  };
  versions.reverse();
  if versions.len() > 100 { versions.truncate(100); }
  Ok(versions)
}