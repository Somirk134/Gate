use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::commands::error::{AppError, CommandResult};

/// 跨命令保存检查到的更新对象与已下载字节，供「下载」与「安装」两步复用。
/// Tauri 的 Update 对象不可跨 IPC 序列化，因此缓存在后端内存中。
pub struct UpdateState(pub Mutex<Option<(Update, Vec<u8>)>>);

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

/// GitHub Releases API 返回的最新 Release 结构。
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    html_url: String,
}

fn release_page_url(app: &AppHandle) -> String {
    app.config()
        .bundle
        .homepage
        .clone()
        .map(|home| format!("{}/releases", home.trim_end_matches('/')))
        .unwrap_or_else(|| "https://github.com/Somirk134/Gate/releases".into())
}

/// 从 homepage URL 提取 GitHub API 路径。
fn github_api_url(homepage: &str) -> String {
    // https://github.com/Somirk134/Gate → https://api.github.com/repos/Somirk134/Gate/releases/latest
    if let Some(rest) = homepage.strip_prefix("https://github.com") {
        format!("https://api.github.com{rest}/releases/latest")
    } else if let Some(rest) = homepage.strip_prefix("http://github.com") {
        format!("https://api.github.com{rest}/releases/latest")
    } else {
        "https://api.github.com/repos/Somirk134/Gate/releases/latest".into()
    }
}

/// 简易语义版本比较：返回正数表示 a > b（a 更新），0 表示相等。
fn compare_versions(a: &str, b: &str) -> i32 {
    let pa: Vec<u32> = a.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let pb: Vec<u32> = b.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let len = pa.len().max(pb.len());
    for i in 0..len {
        let va = pa.get(i).copied().unwrap_or(0);
        let vb = pb.get(i).copied().unwrap_or(0);
        if va != vb {
            return va.cmp(&vb) as i32;
        }
    }
    0
}

/// 通过 GitHub REST API 直接查询最新 Release 版本（Tauri updater 不可用时的回退方案）。
async fn check_via_github_api(
    app: &AppHandle,
    current_version: &str,
) -> CommandResult<UpdatePayload> {
    let releases_url = release_page_url(app);
    let api_url = github_api_url(&releases_url.replace("/releases", ""));

    let client = reqwest::Client::builder()
        .user_agent("gate-client-updater")
        .build()
        .map_err(|source| {
            AppError::from_source(
                "UPDATE_CLIENT_FAILED",
                "errors.updateCheckFailed",
                source,
            )
        })?;

    let release: GithubRelease = client
        .get(&api_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|source| {
            AppError::from_source(
                "UPDATE_CHECK_FAILED",
                "errors.updateCheckFailed",
                source,
            )
        })?
        .json()
        .await
        .map_err(|source| {
            AppError::from_source(
                "UPDATE_RESPONSE_INVALID",
                "errors.updateResponseInvalid",
                source,
            )
        })?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let is_newer = compare_versions(&latest_version, current_version) > 0;

    Ok(if is_newer {
        UpdatePayload {
            available: true,
            current_version: current_version.into(),
            version: Some(latest_version),
            notes: release.body,
            date: release.published_at,
            url: releases_url,
            installable: false, // 通过 API 查询无法获取签名安装包
        }
    } else {
        UpdatePayload {
            available: false,
            current_version: current_version.into(),
            version: Some(latest_version),
            notes: None,
            date: None,
            url: releases_url,
            installable: false,
        }
    })
}

/// 向 GitHub Releases / Tauri updater 查询最新版本。
/// 优先使用 Tauri 内置 updater 插件；若未配置则回退到 GitHub REST API。
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    #[allow(unused_variables)] channel: Option<String>,
) -> CommandResult<UpdatePayload> {
    let current_version = app.package_info().version.to_string();

    // 尝试 Tauri 内置 Updater 插件
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => {
            // Updater 未配置时回退到 GitHub API 直接查询
            return check_via_github_api(&app, &current_version).await;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.body.clone();
            let date = update.date.map(|d| d.to_string());

            app.state::<UpdateState>()
                .0
                .lock()
                .map_err(|_| {
                    AppError::new(
                        "UPDATE_STATE_UNAVAILABLE",
                        "errors.updateStateUnavailable",
                    )
                })?
                .replace((update, Vec::new()));

            Ok(UpdatePayload {
                available: true,
                current_version,
                version: Some(version),
                notes,
                date,
                url: release_page_url(&app),
                installable: true,
            })
        }
        Ok(None) => Ok(UpdatePayload {
            available: false,
            current_version,
            version: None,
            notes: None,
            date: None,
            url: release_page_url(&app),
            installable: false,
        }),
        Err(e) => {
            // Updater 自身检查失败时也尝试 GitHub API 回退
            tracing::warn!("Tauri updater check 失败，尝试 GitHub API 回退：{e}");
            check_via_github_api(&app, &current_version).await
        }
    }
}

/// 下载更新包到本地（不立即安装）。
#[tauri::command]
pub async fn download_update(app: AppHandle) -> CommandResult<()> {
    let (update, _) = {
        let state = app.state::<UpdateState>();
        let mut guard = state.0.lock().map_err(|_| {
            AppError::new(
                "UPDATE_STATE_UNAVAILABLE",
                "errors.updateStateUnavailable",
            )
        })?;
        guard.take()
    }
    .ok_or_else(|| AppError::new("UPDATE_NOT_PENDING", "errors.updateNoDownload"))?;

    let bytes = update
        .download(|_chunk, _total| {}, || {})
        .await
        .map_err(|source| {
            AppError::from_source(
                "UPDATE_DOWNLOAD_FAILED",
                "errors.updateDownloadFailed",
                source,
            )
        })?;

    {
        let state = app.state::<UpdateState>();
        let mut guard = state.0.lock().map_err(|_| {
            AppError::new(
                "UPDATE_STATE_UNAVAILABLE",
                "errors.updateStateUnavailable",
            )
        })?;
        guard.replace((update, bytes));
    }
    Ok(())
}

/// 安装已下载的更新包并重启应用。
#[tauri::command]
pub async fn install_update(app: AppHandle) -> CommandResult<()> {
    let (update, bytes) = {
        let state = app.state::<UpdateState>();
        let mut guard = state.0.lock().map_err(|_| {
            AppError::new(
                "UPDATE_STATE_UNAVAILABLE",
                "errors.updateStateUnavailable",
            )
        })?;
        guard.take()
    }
    .ok_or_else(|| AppError::new("UPDATE_NOT_PENDING", "errors.updateNoInstall"))?;

    update.install(bytes).map_err(|source| {
        AppError::from_source(
            "UPDATE_INSTALL_FAILED",
            "errors.updateInstallFailed",
            source,
        )
    })
}
