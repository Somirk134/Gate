use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

/// 跨命令保存检查到的更新对象，供「下载」与「安装」两步复用。
/// Tauri 的 Update 对象不可跨 IPC 序列化，因此缓存在后端内存中。
pub struct UpdateState(pub Mutex<Option<Update>>);

impl Default for UpdateState {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

/// 检查更新后回传给前端的载荷，字段名使用 camelCase 以便前端直接消费。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePayload {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
    pub url: String,
    /// 是否支持应用内下载安装（签名校验通过且平台匹配时为 true）。
    pub installable: bool,
}

/// 向 GitHub Releases 查询最新版本。
/// channel 参数仅作预留（当前统一走 latest.json，不区分 channel）。
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    #[allow(unused_variables)] channel: Option<String>,
) -> Result<UpdatePayload, String> {
    let current_version = app.package_info().version.to_string();
    let releases_url = release_page_url(&app);

    let updater = match app.updater() {
        Ok(u) => u,
        // 未配置 updater（无 pubkey/endpoints）时降级为「禁用」分支。
        Err(_) => {
            return Ok(UpdatePayload {
                available: false,
                current_version,
                version: None,
                notes: None,
                date: None,
                url: releases_url,
                installable: false,
            });
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.notes.clone();
            let date = update.date.map(|d| d.to_string());
            let url = update.url.clone();

            // 缓存更新对象，供后续 download / install 命令使用。锁仅短暂持有。
            app.state::<UpdateState>().0.lock().unwrap().replace(update);

            Ok(UpdatePayload {
                available: true,
                current_version,
                version: Some(version),
                notes,
                date,
                url,
                installable: true,
            })
        }
        Ok(None) => Ok(UpdatePayload {
            available: false,
            current_version,
            version: None,
            notes: None,
            date: None,
            url: releases_url,
            installable: false,
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// 下载更新包到本地（不立即安装）。
#[tauri::command]
pub async fn download_update(app: AppHandle) -> Result<(), String> {
    // take 把 Update 移出 Mutex 并立即释放锁，避免跨 await 持有 MutexGuard。
    let update = {
        let mut guard = app.state::<UpdateState>().0.lock().unwrap();
        guard.take()
    }
    .ok_or_else(|| "NO_PENDING_UPDATE".to_string())?;

    let result = update.download(|_chunk, _total| {}).await.map_err(|e| e.to_string());

    // 下载完成后放回 state，供 install_update 复用同一对象。
    app.state::<UpdateState>().0.lock().unwrap().replace(update);
    result
}

/// 安装已下载的更新包并重启应用。
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = {
        let mut guard = app.state::<UpdateState>().0.lock().unwrap();
        guard.take()
    }
    .ok_or_else(|| "NO_PENDING_UPDATE".to_string())?;

    update.install().map_err(|e| e.to_string())
}

fn release_page_url(app: &AppHandle) -> String {
    app.config()
        .bundle
        .homepage
        .clone()
        .map(|home| format!("{}/releases", home.trim_end_matches('/')))
        .unwrap_or_else(|| "https://github.com/Somirk134/Gate/releases".into())
}
