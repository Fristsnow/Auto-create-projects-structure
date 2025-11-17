/*
 * @Author: FirstsnowLucky firstsnow1119@163.com
 * @Date: 2025-11-17 08:58:23
 * @LastEditors: FirstsnowLucky firstsnow1119@163.com
 * @LastEditTime: 2025-11-17 09:01:20
 * @FilePath: \Auto-create-projects-structure\src-tauri\src\commands\env.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
#[tauri::command]
pub fn check_environment() -> serde_json::Value {
  use std::process::Command;

  // 检测 node
  let node = Command::new("node")
    .arg("--version")
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .map(|s| s.trim().to_string());

  // 加强版 pnpm 检测（兼容 Windows 上的 pnpm.cmd / pnpm.exe）
  #[cfg(windows)]
  let pnpm = {
    let try_ver = |bin: &str| -> Option<String> {
      Command::new(bin)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
    };

    // 依次尝试 pnpm / pnpm.cmd / pnpm.exe
    try_ver("pnpm").or_else(|| try_ver("pnpm.cmd")).or_else(|| try_ver("pnpm.exe")).or_else(|| {
      // 兜底：使用 where 定位绝对路径后再执行
      Command::new("where")
        .arg("pnpm")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|p| p.trim().to_string()))
        .and_then(|path| try_ver(&path))
    })
  };

  #[cfg(not(windows))]
  let pnpm = Command::new("pnpm")
    .arg("--version")
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .map(|s| s.trim().to_string());

  serde_json::json!({ "node": node, "pnpm": pnpm })
}

#[tauri::command]
pub fn get_system_paths() -> serde_json::Value {
  // 仅针对 Windows 进行基于 USERPROFILE 的常用目录推断
  #[cfg(windows)]
  {
    let base = std::env::var("USERPROFILE").unwrap_or_default();
    let base = std::path::PathBuf::from(base);
    let path_of = |name: &str| {
      let mut p = base.clone();
      p.push(name);
      if p.exists() { Some(p.to_string_lossy().to_string()) } else { None }
    };
    return serde_json::json!({
      "desktop": path_of("Desktop"),
      "downloads": path_of("Downloads"),
      "documents": path_of("Documents"),
      "pictures": path_of("Pictures")
    });
  }

  // 非 Windows 平台：返回空，后续可扩展
  #[cfg(not(windows))]
  {
    serde_json::json!({})
  }
}