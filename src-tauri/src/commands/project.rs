use tauri::Manager;
use tauri::Emitter;
use tauri::async_runtime;

#[tauri::command]
pub fn check_target_dir(directory: String, name: String) -> Result<(), String> {
  let base = std::path::Path::new(&directory);
  let project_dir = base.join(&name);
  if project_dir.exists() {
    // 判断是否非空
    match std::fs::read_dir(&project_dir) {
      Ok(mut it) => {
        if it.next().is_some() {
          return Err(format!(
            "目标文件夹 \"{}\" 非空（路径：{}），请更换名称或选择空目录。",
            name,
            project_dir.display()
          ));
        }
      },
      Err(_) => {
        // 无法读取则谨慎处理为不可覆盖
        return Err(format!(
          "目标文件夹 \"{}\" 已存在（路径：{}），请更换名称或选择空目录。",
          name,
          project_dir.display()
        ));
      }
    }
  }
  Ok(())
}

#[tauri::command]
pub fn create_project(
  app_handle: tauri::AppHandle,
  version: String,
  lang: String,
  name: String,
  directory: String,
  set_default: bool,
  features: Vec<String>,
) -> Result<(), String> {
  use std::process::Command;

  // 解析 pnpm 可执行路径（兼容 Windows 上的 pnpm.cmd / pnpm.exe）
  #[allow(unused_variables)]
  fn resolve_pnpm() -> Option<String> {
    #[cfg(windows)]
    {
      let try_bin = |bin: &str| -> bool {
        Command::new(bin)
          .arg("--version")
          .output()
          .ok()
          .is_some()
      };

      let candidates = ["pnpm", "pnpm.cmd", "pnpm.exe"];
      for c in candidates {
        if try_bin(c) {
          return Some(c.to_string());
        }
      }

      // 兜底：where 定位绝对路径
      if let Ok(out) = Command::new("where").arg("pnpm").output() {
        if let Ok(s) = String::from_utf8(out.stdout) {
          if let Some(path) = s.lines().next() {
            let p = path.trim().to_string();
            if try_bin(&p) {
              return Some(p);
            }
          }
        }
      }
      None
    }

    #[cfg(not(windows))]
    {
      Some("pnpm".to_string())
    }
  }

  let pnpm_bin = resolve_pnpm().ok_or_else(|| "pnpm not found".to_string())?;

  // optionally persist default directory
  if set_default {
    if let Ok(mut cfg_dir) = app_handle.path().app_config_dir() {
      std::fs::create_dir_all(&cfg_dir).map_err(|e| e.to_string())?;
      cfg_dir.push("config.json");
      let payload = serde_json::json!({ "default_directory": directory });
      std::fs::write(&cfg_dir, payload.to_string()).map_err(|e| e.to_string())?;
    }
  }

  // ensure base directory exists & pre-check target dir non-empty
  let base = std::path::Path::new(&directory);
  if !base.exists() { std::fs::create_dir_all(&base).map_err(|e| e.to_string())?; }
  let project_dir = base.join(&name);
  if project_dir.exists() {
    // 阻止覆盖：非空则直接终止
    match std::fs::read_dir(&project_dir) {
      Ok(mut it) => {
        if it.next().is_some() {
          return Err(format!(
            "目标文件夹 \"{}\" 非空（路径：{}），已终止创建。",
            name,
            project_dir.display()
          ));
        }
      },
      Err(_) => {
        return Err(format!(
          "目标文件夹 \"{}\" 已存在（路径：{}），已终止创建。",
          name,
          project_dir.display()
        ));
      }
    }
  }

  // build command
  let mut cmd = Command::new(&pnpm_bin);
  match version.as_str() {
    "vue3" => {
      // 使用 create-vue v3：无需显式指定 --packageManager，跟随 runner（pnpm）
      let mut args = vec!["dlx", "create-vue@latest", &name];
      if lang == "ts" { args.push("--ts"); }
      if features.iter().any(|f| f == "router") { args.push("--router"); }
      if features.iter().any(|f| f == "pinia") { args.push("--pinia"); }
      // 固定包管理器并禁用交互提示
      args.push("--packageManager");
      args.push("pnpm");
      cmd.current_dir(base).args(args).env("CI", "true");
    }
    "vue2" => {
      // Vue CLI v5 with inline preset to force Vue2 minimal (babel)
      let inline_preset = serde_json::json!({
        "vueVersion": "2",
        "plugins": {"@vue/cli-plugin-babel": {}}
      }).to_string();
      cmd.current_dir(base)
        .args(["dlx", "@vue/cli@5", "create", &name, "--inlinePreset", &inline_preset, "--packageManager", "pnpm", "--no-git", "--force"]);
    }
    _ => return Err("unknown version".into()),
  }

  let status = cmd.status().map_err(|e| e.to_string())?;
  if !status.success() {
    return Err("project creation failed".into());
  }

  // install dependencies inside created project (safety step)
  let mut install = Command::new(&pnpm_bin);
  install.current_dir(&project_dir).arg("install");
  install.status().map_err(|e| e.to_string())?;

  // 根据 features 追加依赖
  let project_dir = base.join(&name);
  let add = |dev: bool, pkgs: &[&str]| -> Result<(), String> {
    if pkgs.is_empty() { return Ok(()); }
    for p in pkgs {
      let mut c = Command::new(&pnpm_bin);
      c.current_dir(&project_dir);
      c.arg("add");
      if dev { c.arg("-D"); }
      c.arg(p);
      let st = c.status().map_err(|e| e.to_string())?;
      if !st.success() { return Err(format!("failed to add dependency: {}", p)); }
    }
    Ok(())
  };

  // 动态依赖安装：从注册库读取所有选择特性并安装（含版本与 dev 标记）
  {
    let mut path = app_handle.path().app_config_dir().map_err(|e| e.to_string())?;
    path.push("components.json");
    let registry_v = if path.exists() {
      let s = std::fs::read_to_string(&path).unwrap_or_else(|_| String::new());
      serde_json::from_str::<serde_json::Value>(&s).unwrap_or_else(|_| serde_json::json!({ "components": [] }))
    } else { serde_json::json!({ "components": [] }) };
    if let Some(arr) = registry_v.get("components").and_then(|x| x.as_array()) {
      for k in features.iter() {
        if let Some(item) = arr.iter().find(|it| it.get("key").and_then(|x| x.as_str()) == Some(k.as_str())) {
          let dev = item.get("dev").and_then(|x| x.as_bool()).unwrap_or(false);
          let s = item.get("supported");
          let vue_ok = if version == "vue3" { s.and_then(|m| m.get("vue3")).and_then(|x| x.as_bool()).unwrap_or(true) } else { s.and_then(|m| m.get("vue2")).and_then(|x| x.as_bool()).unwrap_or(true) };
          let lang_ok = if lang == "ts" { s.and_then(|m| m.get("ts")).and_then(|x| x.as_bool()).unwrap_or(true) } else { s.and_then(|m| m.get("js")).and_then(|x| x.as_bool()).unwrap_or(true) };
          if !vue_ok || !lang_ok { continue; }
          let mut pkgs: Vec<String> = Vec::new();
          if let Some(list) = item.get("packages").and_then(|x| x.as_array()) {
            for p in list {
              if let Some(name) = p.as_str() {
                // 版本拼接：若存在 versions 则选择对应版本
                let ver = if version == "vue3" { item.get("versions").and_then(|m| m.get("vue3")).and_then(|x| x.as_str()) } else { item.get("versions").and_then(|m| m.get("vue2")).and_then(|x| x.as_str()) };
                if let Some(v) = ver { pkgs.push(format!("{}@{}", name, v)); } else { pkgs.push(name.to_string()); }
              }
            }
          } else if let Some(name) = item.get("package").and_then(|x| x.as_str()) {
            let ver = if version == "vue3" { item.get("versions").and_then(|m| m.get("vue3")).and_then(|x| x.as_str()) } else { item.get("versions").and_then(|m| m.get("vue2")).and_then(|x| x.as_str()) };
            if let Some(v) = ver { pkgs.push(format!("{}@{}", name, v)); } else { pkgs.push(name.to_string()); }
          }
          if !pkgs.is_empty() {
            let pkgs_refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect();
            add(dev, &pkgs_refs)?;
          }
        }
      }
    }
  }

  // 处理自定义注册的组件库：对未知的 feature key 根据注册表安装对应包
  // 读取注册表
  let registry_v = {
    let mut path = app_handle.path().app_config_dir().map_err(|e| e.to_string())?;
    path.push("components.json");
    if path.exists() {
      let s = std::fs::read_to_string(&path).unwrap_or_else(|_| String::new());
      serde_json::from_str::<serde_json::Value>(&s).unwrap_or_else(|_| serde_json::json!({ "components": [] }))
    } else { serde_json::json!({ "components": [] }) }
  };
  if let Some(arr) = registry_v.get("components").and_then(|x| x.as_array()) {
    // 已知键集合
    let known = ["router","pinia","sass","naive-ui","vfonts","xicons"];
    for k in features.iter() {
      if known.iter().any(|x| x == k) { continue; }
      if let Some(item) = arr.iter().find(|it| it.get("key").and_then(|x| x.as_str()) == Some(k.as_str())) {
        let pkg = item.get("package").and_then(|x| x.as_str()).unwrap_or("");
        let dev = item.get("dev").and_then(|x| x.as_bool()).unwrap_or(false);
        // 支持性检查
        let ok = {
          let s = item.get("supported");
          let vue_ok = if version == "vue3" { s.and_then(|m| m.get("vue3")).and_then(|x| x.as_bool()).unwrap_or(true) } else { s.and_then(|m| m.get("vue2")).and_then(|x| x.as_bool()).unwrap_or(true) };
          let lang_ok = if lang == "ts" { s.and_then(|m| m.get("ts")).and_then(|x| x.as_bool()).unwrap_or(true) } else { s.and_then(|m| m.get("js")).and_then(|x| x.as_bool()).unwrap_or(true) };
          vue_ok && lang_ok
        };
        if !pkg.is_empty() && ok {
          let _ = add(dev, &[pkg]);
        }
      }
    }
  }

  // 配置主入口：注入样式与字体（若选择）
  let main_ts = project_dir.join("src").join("main.ts");
  let main_js = project_dir.join("src").join("main.js");
  let target_main = if main_ts.exists() { main_ts } else { main_js };
  if target_main.exists() {
    let mut content = std::fs::read_to_string(&target_main).map_err(|e| e.to_string())?;
    let mut prepend_lines: Vec<String> = Vec::new();
    if features.iter().any(|f| f == "sass") { prepend_lines.push("import './styles/main.scss'".into()); }
    if features.iter().any(|f| f == "vfonts") { prepend_lines.push("import 'vfonts/Lato.css'".into()); prepend_lines.push("import 'vfonts/FiraCode.css'".into()); }
    if features.iter().any(|f| f == "pinia") && !content.contains("pinia-plugin-persistedstate") {
      prepend_lines.push("import persisted from 'pinia-plugin-persistedstate'".into());
    }
    // 仅在未包含时插入
    let need_update = prepend_lines.iter().any(|l| !content.contains(l));
    if need_update {
      let mut header = String::new();
      for l in prepend_lines { if !content.contains(&l) { header.push_str(&l); header.push('\n'); } }
      content = format!("{}{}", header, content);
      std::fs::write(&target_main, &content).map_err(|e| e.to_string())?;
    }

    // 注入 pinia 插件初始化（Vue 3），将 app.use(createPinia()) 替换为带插件的形式
    if version == "vue3" && features.iter().any(|f| f == "pinia") {
      let new_content = if content.contains("app.use(createPinia())") {
        content.replace(
          "app.use(createPinia())",
          "const pinia = createPinia();\npinia.use(persisted);\napp.use(pinia)"
        )
      } else if content.contains("createPinia()") && content.contains("app.use(") && !content.contains("pinia.use(persisted)") {
        // 若存在 createPinia 调用但不在 app.use(createPinia()) 的形式，则尝试追加插件注册
        let mut appended = content.clone();
        if !appended.contains("const pinia = createPinia()") {
          appended = appended.replace("createPinia()", "const pinia = createPinia();\npinia.use(persisted);\npinia");
        } else {
          appended.push_str("\n// pinia persisted plugin\npinia.use(persisted)\n");
        }
        appended
      } else {
        content.clone()
      };
      if new_content != content {
        std::fs::write(&target_main, new_content).map_err(|e| e.to_string())?;
      }
    }
  }

  // 生成基础样式文件（若选择 sass）
  if features.iter().any(|f| f == "sass") {
    let styles_dir = project_dir.join("src").join("styles");
    std::fs::create_dir_all(&styles_dir).map_err(|e| e.to_string())?;
    let main_scss = styles_dir.join("main.scss");
    if !main_scss.exists() {
      let scss = r#"$primary-color: #2f54eb;
body { background: #fafafa; color: #1f1f1f; }
.app-container { max-width: 1080px; margin: 0 auto; padding: 12px; }
"#;
      std::fs::write(&main_scss, scss).map_err(|e| e.to_string())?;
    }
  }

  // 生成 Pinia 示例（Vue 3 且选择 pinia），并使用持久化
  if version == "vue3" && features.iter().any(|f| f == "pinia") {
    let stores_dir = project_dir.join("src").join("stores");
    std::fs::create_dir_all(&stores_dir).map_err(|e| e.to_string())?;
    let ext = if lang == "ts" { "ts" } else { "js" };
    let file = stores_dir.join(format!("counter.{}", ext));
    if !file.exists() {
      let content_ts = r#"import { defineStore } from 'pinia'
interface CounterState { count: number }
export const useCounterStore = defineStore('counter', {
  state: (): CounterState => ({ count: 0 }),
  actions: { increment() { this.count++ } },
  persist: true,
})
"#;
      let content_js = r#"import { defineStore } from 'pinia'
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  actions: { increment() { this.count++ } },
  persist: true,
})
"#;
      let payload = if ext == "ts" { content_ts } else { content_js };
      std::fs::write(&file, payload).map_err(|e| e.to_string())?;
    }
  }

  Ok(())
}

// 异步后台创建项目：立即返回，后台执行并推送事件日志，避免卡顿
#[tauri::command]
pub async fn create_project_async(
  app_handle: tauri::AppHandle,
  version: String,
  lang: String,
  name: String,
  directory: String,
  set_default: bool,
  features: Vec<String>,
) -> Result<(), String> {
  use std::process::Command;

  // 事件名常量
  const EVT_LOG: &str = "project:create_log";
  const EVT_DONE: &str = "project:create_done";

  // 复制参数供后台线程使用
  let handle = app_handle.clone();
  let handle_log = handle.clone();
  let handle_done = handle.clone();
  let version_c = version.clone();
  let lang_c = lang.clone();
  let name_c = name.clone();
  let dir_c = directory.clone();
  let features_c = features.clone();

  // 发送日志的便捷函数
  let send_log = move |msg: String| {
    let _ = handle_log.emit(EVT_LOG, serde_json::json!({"line": msg}));
  };

  // 后台阻塞线程执行原逻辑，分阶段推送日志
  async_runtime::spawn_blocking(move || {
    let emit = |s: &str| send_log(s.to_string());
    emit(&format!("开始创建项目：{}", name_c));
    emit(&format!("目标目录：{}", dir_c));
    emit(&format!("Vue 版本：{}，语言：{}", version_c, lang_c));
    if features_c.is_empty() { emit("选择特性：无"); } else { emit(&format!("选择特性：{}", features_c.join(", "))); }

    // 解析 pnpm 可执行路径
    #[allow(unused_variables)]
    fn resolve_pnpm() -> Option<String> {
      #[cfg(windows)]
      {
        let try_bin = |bin: &str| -> bool { Command::new(bin).arg("--version").output().ok().is_some() };
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
    let pnpm_bin = match resolve_pnpm() {
      Some(p) => p,
      None => {
        let _ = handle_done.emit(EVT_DONE, serde_json::json!({"success": false, "error": "pnpm not found"}));
        return;
      }
    };

    // 持久化默认目录（可选）
    if set_default {
      if let Ok(mut cfg_dir) = handle.path().app_config_dir() {
        let _ = std::fs::create_dir_all(&cfg_dir);
        cfg_dir.push("config.json");
        let payload = serde_json::json!({ "default_directory": dir_c });
        let _ = std::fs::write(&cfg_dir, payload.to_string());
      }
    }

    // 目录检查与准备
    let base = std::path::Path::new(&dir_c);
    if !base.exists() { let _ = std::fs::create_dir_all(&base); }
    let project_dir = base.join(&name_c);
    if project_dir.exists() {
      match std::fs::read_dir(&project_dir) {
        Ok(mut it) => {
          if it.next().is_some() {
            let _ = handle_done.emit(EVT_DONE, serde_json::json!({
              "success": false,
              "error": format!("目标文件夹 \"{}\" 非空（路径：{}），已终止创建。", name_c, project_dir.display())
            }));
            return;
          }
        },
        Err(_) => {
          let _ = handle_done.emit(EVT_DONE, serde_json::json!({
            "success": false,
            "error": format!("目标文件夹 \"{}\" 已存在（路径：{}），已终止创建。", name_c, project_dir.display())
          }));
          return;
        }
      }
    }

    emit("初始化脚手架与依赖安装...");

    // 构建脚手架命令
    let mut cmd = Command::new(&pnpm_bin);
    match version_c.as_str() {
      "vue3" => {
        // 使用 create-vite 非交互模板，避免交互式选择导致卡顿
        let mut args = vec!["dlx", "create-vite@latest", &name_c, "--", "--template"];
        args.push(if lang_c == "ts" { "vue-ts" } else { "vue" });
        cmd.current_dir(base).args(args);
      }
      "vue2" => {
        let inline_preset = serde_json::json!({"vueVersion": "2", "plugins": {"@vue/cli-plugin-babel": {}}}).to_string();
        cmd.current_dir(base).args(["dlx", "@vue/cli@5", "create", &name_c, "--inlinePreset", &inline_preset, "--packageManager", "pnpm", "--no-git", "--force"]);
      }
      _ => {
        let _ = handle_done.emit(EVT_DONE, serde_json::json!({"success": false, "error": "unknown version"}));
        return;
      }
    }

    match cmd.output() {
      Ok(out) => {
        if out.status.success() {
          emit("脚手架创建完成。");
        } else {
          let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
          let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
          if !stdout.is_empty() { emit(&format!("脚手架输出：{}", stdout)); }
          if !stderr.is_empty() { emit(&format!("脚手架错误：{}", stderr)); }
          let _ = handle_done.emit(EVT_DONE, serde_json::json!({"success": false, "error": "project creation failed"}));
          return;
        }
      }
      Err(e) => {
        let _ = handle_done.emit(EVT_DONE, serde_json::json!({"success": false, "error": format!("执行失败：{}", e)}));
        return;
      }
    }

    // 安装依赖
    let mut install = Command::new(&pnpm_bin);
    install.current_dir(&project_dir).arg("install");
    let _ = install.status();
    emit("依赖安装完成。");

    // 追加依赖与文件调整（复用同步实现的核心逻辑，省略错误终止，仅日志提示）
    let add = |dev: bool, pkgs: &[&str]| -> Result<(), String> {
      if pkgs.is_empty() { return Ok(()); }
      for p in pkgs {
        let mut c = Command::new(&pnpm_bin);
        c.current_dir(&project_dir);
        c.arg("add");
        if dev { c.arg("-D"); }
        c.arg(p);
        let st = c.status().map_err(|e| e.to_string())?;
        if !st.success() { return Err(format!("failed to add dependency: {}", p)); }
        emit(&format!("已安装依赖：{}{}", if dev { "(dev) " } else { "" }, p));
      }
      Ok(())
    };

    // 动态依赖安装（与同步版相同）
    let mut reg_path = match handle.path().app_config_dir() { Ok(p) => p, Err(_) => std::path::PathBuf::new() };
    reg_path.push("components.json");
    let registry_v = if reg_path.exists() {
      let s = std::fs::read_to_string(&reg_path).unwrap_or_else(|_| String::new());
      serde_json::from_str::<serde_json::Value>(&s).unwrap_or_else(|_| serde_json::json!({ "components": [] }))
    } else { serde_json::json!({ "components": [] }) };
    if let Some(arr) = registry_v.get("components").and_then(|x| x.as_array()) {
      for k in features_c.iter() {
        if let Some(item) = arr.iter().find(|it| it.get("key").and_then(|x| x.as_str()) == Some(k.as_str())) {
          let dev = item.get("dev").and_then(|x| x.as_bool()).unwrap_or(false);
          let s = item.get("supported");
          let vue_ok = if version_c == "vue3" { s.and_then(|m| m.get("vue3")).and_then(|x| x.as_bool()).unwrap_or(true) } else { s.and_then(|m| m.get("vue2")).and_then(|x| x.as_bool()).unwrap_or(true) };
          let lang_ok = if lang_c == "ts" { s.and_then(|m| m.get("ts")).and_then(|x| x.as_bool()).unwrap_or(true) } else { s.and_then(|m| m.get("js")).and_then(|x| x.as_bool()).unwrap_or(true) };
          if !vue_ok || !lang_ok { continue; }
          let mut pkgs: Vec<String> = Vec::new();
          if let Some(list) = item.get("packages").and_then(|x| x.as_array()) {
            for p in list { if let Some(name) = p.as_str() {
              let ver = if version_c == "vue3" { item.get("versions").and_then(|m| m.get("vue3")).and_then(|x| x.as_str()) } else { item.get("versions").and_then(|m| m.get("vue2")).and_then(|x| x.as_str()) };
              if let Some(v) = ver { pkgs.push(format!("{}@{}", name, v)); } else { pkgs.push(name.to_string()); }
            } }
          } else if let Some(name) = item.get("package").and_then(|x| x.as_str()) {
            let ver = if version_c == "vue3" { item.get("versions").and_then(|m| m.get("vue3")).and_then(|x| x.as_str()) } else { item.get("versions").and_then(|m| m.get("vue2")).and_then(|x| x.as_str()) };
            if let Some(v) = ver { pkgs.push(format!("{}@{}", name, v)); } else { pkgs.push(name.to_string()); }
          }
          if !pkgs.is_empty() { let pkgs_refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect(); let _ = add(dev, &pkgs_refs); }
        }
      }
    }

    // 写入样式与示例 store（略去错误终止）
    let main_ts = project_dir.join("src").join("main.ts");
    let main_js = project_dir.join("src").join("main.js");
    let target_main = if main_ts.exists() { main_ts } else { main_js };
    if target_main.exists() {
      if let Ok(mut content) = std::fs::read_to_string(&target_main) {
        let mut prepend_lines: Vec<String> = Vec::new();
        if features_c.iter().any(|f| f == "sass") { prepend_lines.push("import './styles/main.scss'".into()); }
        if features_c.iter().any(|f| f == "vfonts") { prepend_lines.push("import 'vfonts/Lato.css'".into()); prepend_lines.push("import 'vfonts/FiraCode.css'".into()); }
        if features_c.iter().any(|f| f == "pinia") && !content.contains("pinia-plugin-persistedstate") {
          prepend_lines.push("import persisted from 'pinia-plugin-persistedstate'".into());
        }
        let need_update = prepend_lines.iter().any(|l| !content.contains(l));
        if need_update {
          let mut header = String::new();
          for l in prepend_lines { if !content.contains(&l) { header.push_str(&l); header.push('\n'); } }
          content = format!("{}{}", header, content);
          let _ = std::fs::write(&target_main, &content);
        }
      }
    }

    // 生成样式与 pinia 示例
    if features_c.iter().any(|f| f == "sass") {
      let styles_dir = project_dir.join("src").join("styles");
      let _ = std::fs::create_dir_all(&styles_dir);
      let main_scss = styles_dir.join("main.scss");
      if !main_scss.exists() {
        let scss = r#"$primary-color: #2f54eb;
body { background: #fafafa; color: #1f1f1f; }
.app-container { max-width: 1080px; margin: 0 auto; padding: 12px; }
"#;
        let _ = std::fs::write(&main_scss, scss);
      }
    }

    if version_c == "vue3" && features_c.iter().any(|f| f == "pinia") {
      let stores_dir = project_dir.join("src").join("stores");
      let _ = std::fs::create_dir_all(&stores_dir);
      let ext = if lang_c == "ts" { "ts" } else { "js" };
      let file = stores_dir.join(format!("counter.{}", ext));
      if !file.exists() {
        let content_ts = r#"import { defineStore } from 'pinia'
interface CounterState { count: number }
export const useCounterStore = defineStore('counter', {
  state: (): CounterState => ({ count: 0 }),
  actions: { increment() { this.count++ } },
  persist: true,
})
"#;
        let content_js = r#"import { defineStore } from 'pinia'
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  actions: { increment() { this.count++ } },
  persist: true,
})
"#;
        let payload = if ext == "ts" { content_ts } else { content_js };
        let _ = std::fs::write(&file, payload);
      }
    }

    emit("项目创建成功。");
    let _ = handle_done.emit(EVT_DONE, serde_json::json!({"success": true}));
  });

  // 立即返回，前端订阅事件显示进度
  Ok(())
}