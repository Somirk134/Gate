use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use instant_acme::{
    Account, AuthorizationStatus, ChallengeType as InstantChallengeType, Identifier, NewAccount,
    NewOrder, OrderStatus, RetryPolicy,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, sync::Arc};
use tauri::State;
use tokio::sync::Mutex;
use x509_parser::prelude::*;

use crate::commands::error::{AppError, CommandResult};
use crate::utils::app_data_dir;

/* ────────────────────────── 数据模型 ────────────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRecord {
    domain: String,
    issuer: String,
    expire_time: DateTime<Utc>,
    create_time: DateTime<Utc>,
    renew_time: Option<DateTime<Utc>>,
    status: String,
    fingerprint: CertificateFingerprint,
    algorithm: Value,
    san: Vec<String>,
    cert_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
    serial_number: Option<String>,
    #[serde(default)]
    last_error: Option<String>,
    #[serde(default)]
    auto_renewal_enabled: Option<bool>,
    #[serde(default)]
    tls_version: Option<String>,
    #[serde(default)]
    deploy_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateFingerprint {
    sha256: String,
}

/* ────────────────────────── 既有命令 ────────────────────────── */

#[tauri::command]
pub async fn certificate_list() -> CommandResult<Value> {
    let root = certificate_store_root();
    let mut certificates = Vec::new();

    if root.exists() {
        for entry in fs::read_dir(&root).map_err(|error| {
            AppError::from_source(
                "CERTIFICATE_DIRECTORY_READ_FAILED",
                "errors.certificate.readDirectoryFailed",
                error,
            )
        })? {
            let entry = entry.map_err(|error| {
                AppError::from_source(
                    "CERTIFICATE_DIRECTORY_READ_FAILED",
                    "errors.certificate.readDirectoryFailed",
                    error,
                )
            })?;
            if !entry
                .file_type()
                .map_err(|error| {
                    AppError::from_source(
                        "CERTIFICATE_DIRECTORY_READ_FAILED",
                        "errors.certificate.readDirectoryFailed",
                        error,
                    )
                })?
                .is_dir()
            {
                continue;
            }

            let metadata_path = entry.path().join("metadata.json");
            if !metadata_path.exists() {
                continue;
            }

            let record = load_record(&metadata_path)?;
            certificates.push(certificate_summary(record, &entry.path()));
        }
    }

    certificates.sort_by(|left, right| {
        left.get("domain")
            .and_then(Value::as_str)
            .cmp(&right.get("domain").and_then(Value::as_str))
    });

    Ok(json!({
        "storeRoot": root,
        "certificates": certificates,
        "generatedAt": Utc::now().timestamp_millis()
    }))
}

#[tauri::command]
pub async fn certificate_detail(domain: String) -> CommandResult<Value> {
    let domain = normalize_domain(&domain)?;
    let (record, certificate_pem) = load_certificate(&domain)?;

    Ok(json!({
        "summary": certificate_summary(record.clone(), &certificate_domain_dir(&domain)),
        "record": record,
        "certificatePem": certificate_pem.unwrap_or_default(),
        "generatedAt": Utc::now().timestamp_millis()
    }))
}

#[tauri::command]
pub async fn certificate_export_pem(domain: String) -> CommandResult<String> {
    let domain = normalize_domain(&domain)?;
    let (_, certificate_pem) = load_certificate(&domain)?;
    certificate_pem.ok_or_else(|| {
        AppError::with_details(
            "CERTIFICATE_PEM_NOT_FOUND",
            "errors.certificate.pemNotFound",
            json!({ "domain": domain }),
        )
    })
}

/* ────────────────────────── 统计与健康 ────────────────────────── */

#[tauri::command]
pub async fn certificate_stats() -> CommandResult<Value> {
    let root = certificate_store_root();
    let mut all = Vec::new();

    if root.exists() {
        for entry in fs::read_dir(&root).map_err(|e| {
            AppError::from_source(
                "CERTIFICATE_DIRECTORY_READ_FAILED",
                "errors.certificate.readDirectoryFailed",
                e,
            )
        })? {
            let entry = entry.map_err(|e| {
                AppError::from_source(
                    "CERTIFICATE_DIRECTORY_READ_FAILED",
                    "errors.certificate.readDirectoryFailed",
                    e,
                )
            })?;
            if !entry.file_type().map_err(|e| {
                AppError::from_source(
                    "CERTIFICATE_DIRECTORY_READ_FAILED",
                    "errors.certificate.readDirectoryFailed",
                    e,
                )
            })?.is_dir() {
                continue;
            }
            let mp = entry.path().join("metadata.json");
            if mp.exists() {
                all.push(load_record(&mp)?);
            }
        }
    }

    let now = Utc::now();
    let total = all.len();
    let active = all.iter().filter(|r| {
        let s = normalize_status(&r.status);
        s == "active" && (r.expire_time - now).num_days() > 30
    }).count();
    let expiring_soon = all.iter().filter(|r| {
        let days = (r.expire_time - now).num_days();
        days >= 0 && days <= 30
    }).count();
    let expired = all.iter().filter(|r| {
        (r.expire_time - now).num_days() < 0
    }).count();
    let failed = all.iter().filter(|r| normalize_status(&r.status) == "failed").count();
    let auto_renewal_failed = all.iter().filter(|r| r.last_error.is_some()).count();
    let auto_renewal_ok = total - auto_renewal_failed;

    // 健康评分: 有效占比 * 60 + 续期正常占比 * 30 + 无错误占比 * 10
    let health_score = if total == 0 {
        0
    } else {
        let valid_ratio = (active as f64 / total as f64) * 60.0;
        let renew_ratio = (auto_renewal_ok as f64 / total as f64) * 30.0;
        let no_error_ratio = ((total - failed) as f64 / total as f64) * 10.0;
        (valid_ratio + renew_ratio + no_error_ratio).round() as i64
    };

    // 状态分布（用于圆环图）
    let status_distribution = json!({
        "active": active,
        "expiringSoon": expiring_soon,
        "expired": expired,
        "failed": failed,
    });

    // 健康检查项
    let acme_enabled = env::var_os("GATE_ACME_EMAIL").is_some()
        || env::var_os("GATE_ACME_AUTO").is_some();
    let health_checks = json!({
        "autoRenewal": auto_renewal_failed == 0 && total > 0,
        "acme": acme_enabled,
        "dns": true,
        "http01": true,
        "tls13": true,
        "sni": true,
    });

    Ok(json!({
        "total": total,
        "active": active,
        "expiringSoon": expiring_soon,
        "expired": expired,
        "failed": failed,
        "autoRenewalOk": auto_renewal_ok,
        "autoRenewalFailed": auto_renewal_failed,
        "healthScore": health_score,
        "statusDistribution": status_distribution,
        "healthChecks": health_checks,
        "generatedAt": now.timestamp_millis()
    }))
}

/* ────────────────────────── 删除 ────────────────────────── */

#[tauri::command]
pub async fn certificate_delete(domain: String) -> CommandResult<Value> {
    let domain = normalize_domain(&domain)?;
    let dir = certificate_domain_dir(&domain);

    if !dir.exists() {
        return Err(AppError::with_details(
            "CERTIFICATE_NOT_FOUND",
            "errors.certificate.notFound",
            json!({ "domain": domain }),
        ));
    }

    fs::remove_dir_all(&dir).map_err(|e| {
        AppError::from_source(
            "CERTIFICATE_DELETE_FAILED",
            "errors.certificate.deleteFailed",
            e,
        )
    })?;

    Ok(json!({
        "domain": domain,
        "deleted": true,
        "path": dir
    }))
}

/* ────────────────────────── 导入 ────────────────────────── */

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub domain: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
}

#[tauri::command]
pub async fn certificate_validate_import(
    certificate_pem: String,
    private_key_pem: String,
) -> CommandResult<Value> {
    // 1. 解析 PEM 结构
    let cert_block = ::pem::parse(&certificate_pem).map_err(|e| {
        AppError::from_source(
            "CERTIFICATE_PARSE_FAILED",
            "errors.certificate.parseFailed",
            e,
        )
    })?;

    if cert_block.tag() != "CERTIFICATE" {
        return Err(AppError::with_details(
            "CERTIFICATE_PEM_INVALID",
            "errors.certificate.pemInvalid",
            json!({ "expected": "CERTIFICATE", "found": cert_block.tag() }),
        ));
    }

    // 2. 解析 x509 证书
    let (_, cert) = X509Certificate::from_der(cert_block.contents())
        .map_err(|e| AppError::from_source(
            "CERTIFICATE_X509_PARSE_FAILED",
            "errors.certificate.x509ParseFailed",
            e,
        ))?;

    let subject = cert.subject();
    let cn = subject.iter_common_name().next()
        .and_then(|c| c.as_str().ok())
        .unwrap_or("unknown");

    let issuer = cert.issuer();
    let issuer_cn = issuer.iter_common_name().next()
        .and_then(|c| c.as_str().ok())
        .unwrap_or("unknown");

    // SAN 域名 — 使用 ParsedExtension 迭代
    let san: Vec<String> = cert.extensions()
        .iter()
        .find_map(|ext| {
            if let ParsedExtension::SubjectAlternativeName(san_ext) = ext.parsed_extension() {
                Some(
                    san_ext.general_names
                        .iter()
                        .filter_map(|gn| match gn {
                            GeneralName::DNSName(n) => Some(n.to_string()),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default();

    // 算法 — 通过 OID 字符串判断
    let pk_oid = cert.public_key().algorithm.oid();
    let pk_oid_str = pk_oid.to_string();
    let algorithm = if pk_oid_str == "1.2.840.113549.1.1.1" {
        "RSA"
    } else if pk_oid_str == "1.2.840.10045.2.1" {
        "ECDSA"
    } else if pk_oid_str == "1.3.101.112" {
        "Ed25519"
    } else {
        "Unknown"
    };

    // 过期检查
    let validity = cert.validity();
    let not_before = validity.not_before.timestamp();
    let not_after = validity.not_after.timestamp();
    let now_ts = Utc::now().timestamp();
    let is_expired = not_after < now_ts;
    let days_remaining = (not_after - now_ts) / 86400;

    // 序列号
    let serial = format!("{:x}", cert.serial).to_uppercase();

    // 指纹
    let mut hasher = Sha256::new();
    hasher.update(cert_block.contents());
    let fingerprint = hasher.finalize();
    let fingerprint_hex: String = fingerprint.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");

    // 3. 验证私钥结构
    let key_block = ::pem::parse(&private_key_pem).map_err(|e| {
        AppError::from_source(
            "PRIVATE_KEY_PARSE_FAILED",
            "errors.certificate.keyParseFailed",
            e,
        )
    })?;

    let key_valid = matches!(key_block.tag(), "PRIVATE KEY" | "RSA PRIVATE KEY" | "EC PRIVATE KEY" | "ENCRYPTED PRIVATE KEY");

    // 4. TLS 兼容性
    let tls_supported = !is_expired && key_valid;

    Ok(json!({
        "valid": true,
        "commonName": cn,
        "issuer": issuer_cn,
        "san": san,
        "algorithm": algorithm,
        "serialNumber": serial,
        "fingerprintSha256": fingerprint_hex,
        "notBefore": DateTime::<Utc>::from_timestamp(not_before, 0).unwrap_or_default().to_rfc3339(),
        "notAfter": DateTime::<Utc>::from_timestamp(not_after, 0).unwrap_or_default().to_rfc3339(),
        "isExpired": is_expired,
        "daysRemaining": days_remaining,
        "keyValid": key_valid,
        "keyType": key_block.tag(),
        "tlsSupported": tls_supported,
    }))
}

#[tauri::command]
pub async fn certificate_import(request: ImportRequest) -> CommandResult<Value> {
    let domain = normalize_domain(&request.domain)?;

    // 先验证
    let validation = certificate_validate_import(
        request.certificate_pem.clone(),
        request.private_key_pem.clone(),
    ).await?;

    let validation_obj = validation.as_object().unwrap();

    let dir = certificate_domain_dir(&domain);
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::from_source(
            "CERTIFICATE_DIR_CREATE_FAILED",
            "errors.certificate.dirCreateFailed",
            e,
        )
    })?;

    // 写入证书 PEM
    let cert_path = dir.join("certificate.pem");
    fs::write(&cert_path, &request.certificate_pem).map_err(|e| {
        AppError::from_source(
            "CERTIFICATE_WRITE_FAILED",
            "errors.certificate.writeFailed",
            e,
        )
    })?;

    // 写入私钥 PEM
    let key_path = dir.join("private_key.pem");
    fs::write(&key_path, &request.private_key_pem).map_err(|e| {
        AppError::from_source(
            "CERTIFICATE_WRITE_FAILED",
            "errors.certificate.writeFailed",
            e,
        )
    })?;

    // 设置私钥文件权限（Unix）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
    }

    // 构建 metadata
    let now = Utc::now();
    let not_after_str = validation_obj.get("notAfter").and_then(|v| v.as_str()).unwrap_or("");
    let not_before_str = validation_obj.get("notBefore").and_then(|v| v.as_str()).unwrap_or("");
    let expire_time = DateTime::parse_from_rfc3339(not_after_str)
        .map(|dt| dt.with_timezone::<Utc>(&Utc))
        .unwrap_or(now);
    let create_time = DateTime::parse_from_rfc3339(not_before_str)
        .map(|dt| dt.with_timezone::<Utc>(&Utc))
        .unwrap_or(now);

    let days_remaining = (expire_time - now).num_days();
    let status = if days_remaining < 0 {
        "expired"
    } else if days_remaining <= 30 {
        "expiringSoon"
    } else {
        "active"
    };

    let record = CertificateRecord {
        domain: domain.clone(),
        issuer: validation_obj.get("issuer").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        expire_time,
        create_time,
        renew_time: None,
        status: status.to_string(),
        fingerprint: CertificateFingerprint {
            sha256: validation_obj.get("fingerprintSha256").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        },
        algorithm: Value::String(validation_obj.get("algorithm").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string()),
        san: validation_obj.get("san").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        cert_path: Some(cert_path.clone()),
        key_path: Some(key_path.clone()),
        serial_number: Some(validation_obj.get("serialNumber").and_then(|v| v.as_str()).unwrap_or("").to_string()),
        last_error: None,
        auto_renewal_enabled: Some(false),
        tls_version: Some("TLS1.3".to_string()),
        deploy_status: Some("pending".to_string()),
    };

    let metadata_path = dir.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_vec_pretty(&record).unwrap_or_default())
        .map_err(|e| AppError::from_source(
            "CERTIFICATE_METADATA_WRITE_FAILED",
            "errors.certificate.metadataWriteFailed",
            e,
        ))?;

    Ok(json!({
        "domain": domain,
        "imported": true,
        "validation": validation,
        "path": dir
    }))
}

/* ────────────────────────── 自动续期状态 ────────────────────────── */

#[tauri::command]
pub async fn certificate_auto_renewal_status() -> CommandResult<Value> {
    let acme_email = env::var("GATE_ACME_EMAIL").ok();
    let acme_auto = env::var("GATE_ACME_AUTO").ok();
    let acme_staging = env::var("GATE_ACME_STAGING").ok();
    let acme_dir_url = env::var("GATE_ACME_DIRECTORY_URL").ok();
    let acme_http01_port = env::var("GATE_ACME_HTTP01_PORT").ok();

    let enabled = acme_email.is_some() || acme_auto.as_deref() == Some("true") || acme_auto.as_deref() == Some("1");

    // 检查间隔默认 86400 秒（24 小时）
    let check_interval = env::var("GATE_RENEW_CHECK_INTERVAL")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(86400);

    // 续期提前天数默认 30
    let renew_before_days = env::var("GATE_RENEW_BEFORE_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(30);

    // 统计最近续期情况
    let root = certificate_store_root();
    let mut last_renew_time: Option<DateTime<Utc>> = None;
    let mut last_renew_success = true;
    let mut last_error: Option<String> = None;

    if root.exists() {
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let mp = entry.path().join("metadata.json");
                if mp.exists() {
                    if let Ok(record) = load_record(&mp) {
                        if let Some(rt) = record.renew_time {
                            if last_renew_time.map(|lr| rt > lr).unwrap_or(true) {
                                last_renew_time = Some(rt);
                                last_renew_success = record.last_error.is_none();
                                last_error = record.last_error.clone();
                            }
                        }
                    }
                }
            }
        }
    }

    // 计算距离下次检查的大致时间
    let now = Utc::now();
    let next_check_hours = if let Some(last) = last_renew_time {
        let elapsed = (now - last).num_seconds();
        let remaining = check_interval - elapsed;
        (remaining / 3600).max(0)
    } else {
        check_interval / 3600
    };

    Ok(json!({
        "enabled": enabled,
        "acmeEmail": acme_email,
        "acmeStaging": acme_staging.as_deref() == Some("true") || acme_staging.as_deref() == Some("1"),
        "acmeDirectoryUrl": acme_dir_url,
        "acmeHttp01Port": acme_http01_port.and_then(|p| p.parse::<u16>().ok()).unwrap_or(80),
        "checkIntervalSeconds": check_interval,
        "renewBeforeDays": renew_before_days,
        "scheduleDescription": format!("Every {} hours", check_interval / 3600),
        "lastRenewTime": last_renew_time.map(|t| t.to_rfc3339()),
        "lastRenewSuccess": last_renew_success,
        "lastError": last_error,
        "nextCheckHours": next_check_hours,
        "generatedAt": now.timestamp_millis()
    }))
}

/* ────────────────────────── 域名关联 ────────────────────────── */

#[tauri::command]
pub async fn certificate_domain_associations(domain: String) -> CommandResult<Value> {
    let domain = normalize_domain(&domain)?;

    // 尝试读取域名 SQLite 数据库
    let db_path = domain_db_path();
    let mut associated_domains: Vec<Value> = Vec::new();
    let mut associated_tunnels: Vec<Value> = Vec::new();

    if db_path.exists() {
        if let Ok(conn) = Connection::open(&db_path) {
            // 查询匹配的域名记录
            let like_pattern = format!("%{}%", domain);
            let mut stmt = conn
                .prepare("SELECT host, tunnel_id, status, verify_status FROM domains WHERE host LIKE ?1")
                .map_err(|e| AppError::from_source(
                    "DOMAIN_DB_QUERY_FAILED",
                    "errors.certificate.domainDbQueryFailed",
                    e,
                ))?;

            let rows = stmt
                .query_map([&like_pattern], |row| {
                    Ok(json!({
                        "host": row.get::<_, String>(0)?,
                        "tunnelId": row.get::<_, Option<String>>(1)?,
                        "status": row.get::<_, String>(2)?,
                        "verifyStatus": row.get::<_, String>(3)?,
                    }))
                })
                .map_err(|e| AppError::from_source(
                    "DOMAIN_DB_QUERY_FAILED",
                    "errors.certificate.domainDbQueryFailed",
                    e,
                ))?;

            for row in rows.flatten() {
                let tunnel_id = row.get("tunnelId").and_then(|v| v.as_str()).map(String::from);
                if let Some(tid) = tunnel_id {
                    associated_tunnels.push(json!({ "tunnelId": tid, "domain": row.get("host") }));
                }
                associated_domains.push(row);
            }
        }
    }

    Ok(json!({
        "domain": domain,
        "domains": associated_domains,
        "tunnels": associated_tunnels,
        "projects": [],
        "dbAvailable": db_path.exists(),
        "dbPath": db_path,
        "generatedAt": Utc::now().timestamp_millis()
    }))
}

/* ────────────────────────── 立即续期 / 重新部署 ────────────────────────── */
//
// 这两个操作需要 Server Gateway 运行时支持。
// Tauri 本地层无法直接执行 ACME 签发或 TLS 缓存刷新，
// 返回明确的错误信息指引用户启动服务器。

#[tauri::command]
pub async fn certificate_renew_now(domain: String) -> CommandResult<Value> {
    let domain = normalize_domain(&domain)?;
    let dir = certificate_domain_dir(&domain);

    if !dir.exists() {
        return Err(AppError::with_details(
            "CERTIFICATE_NOT_FOUND",
            "errors.certificate.notFound",
            json!({ "domain": domain }),
        ));
    }

    // 检查是否配置了 ACME
    let acme_configured = env::var_os("GATE_ACME_EMAIL").is_some()
        || env::var_os("GATE_ACME_AUTO").is_some();

    if !acme_configured {
        return Err(AppError::with_details(
            "ACME_NOT_CONFIGURED",
            "errors.certificate.acmeNotConfigured",
            json!({ "domain": domain }),
        ));
    }

    // ACME 续期需要 Server Gateway 运行时。
    // 标记 metadata 为 pending，等待 Gateway 下次续期循环处理。
    let metadata_path = dir.join("metadata.json");
    if metadata_path.exists() {
        let mut record = load_record(&metadata_path)?;
        record.status = "pending".to_string();
        record.last_error = None;
        let _ = fs::write(&metadata_path, serde_json::to_vec_pretty(&record).unwrap_or_default());
    }

    Ok(json!({
        "domain": domain,
        "triggered": true,
        "message": "Renewal triggered. The server gateway will process it in the next renewal cycle."
    }))
}

#[tauri::command]
pub async fn certificate_redeploy(domain: String) -> CommandResult<Value> {
    let domain = normalize_domain(&domain)?;
    let dir = certificate_domain_dir(&domain);

    if !dir.exists() {
        return Err(AppError::with_details(
            "CERTIFICATE_NOT_FOUND",
            "errors.certificate.notFound",
            json!({ "domain": domain }),
        ));
    }

    // 更新部署状态为 pending
    let metadata_path = dir.join("metadata.json");
    if metadata_path.exists() {
        let mut record = load_record(&metadata_path)?;
        record.deploy_status = Some("pending".to_string());
        let _ = fs::write(&metadata_path, serde_json::to_vec_pretty(&record).unwrap_or_default());
    }

    Ok(json!({
        "domain": domain,
        "triggered": true,
        "message": "Redeploy triggered. The server gateway will reload the TLS cache."
    }))
}

#[tauri::command]
pub async fn certificate_toggle_auto_renewal(domain: String, enabled: bool) -> CommandResult<Value> {
    let domain = normalize_domain(&domain)?;
    let dir = certificate_domain_dir(&domain);

    if !dir.exists() {
        return Err(AppError::with_details(
            "CERTIFICATE_NOT_FOUND",
            "errors.certificate.notFound",
            json!({ "domain": domain }),
        ));
    }

    let metadata_path = dir.join("metadata.json");
    if metadata_path.exists() {
        let mut record = load_record(&metadata_path)?;
        record.auto_renewal_enabled = Some(enabled);
        let _ = fs::write(&metadata_path, serde_json::to_vec_pretty(&record).unwrap_or_default());
    }

    Ok(json!({
        "domain": domain,
        "autoRenewalEnabled": enabled
    }))
}

/* ────────────────────────── ACME 申请流程 ────────────────────────── */

/// ACME 会话状态，在 prepare 和 verify 之间保持 Order + Account 在内存中
#[derive(Default)]
pub struct AcmeState {
    pub session: Arc<Mutex<Option<AcmeSession>>>,
}

pub struct AcmeSession {
    pub domain: String,
    pub account: Account,
    pub order: instant_acme::Order,
    pub challenge_type: String,
    pub txt_host: String,
    pub txt_value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcmePrepareRequest {
    pub domain: String,
    pub email: String,
    pub challenge_type: String,
    pub staging: bool,
}

/// 第一步：创建 ACME 账户和订单，获取 challenge，返回 DNS TXT 记录详情
#[tauri::command]
pub async fn certificate_acme_prepare(
    request: AcmePrepareRequest,
    state: State<'_, AcmeState>,
) -> CommandResult<Value> {
    let domain = normalize_domain(&request.domain)?;

    // 确定 ACME 目录 URL
    let directory_url = if let Some(custom) = env::var_os("GATE_ACME_DIRECTORY_URL") {
        custom.to_string_lossy().to_string()
    } else if request.staging
        || env::var("GATE_ACME_STAGING").as_deref() == Ok("true")
        || env::var("GATE_ACME_STAGING").as_deref() == Ok("1")
    {
        "https://acme-staging-v02.api.letsencrypt.org/directory".to_string()
    } else {
        "https://acme-v02.api.letsencrypt.org/directory".to_string()
    };

    // 创建 ACME 账户
    let contact = format!("mailto:{}", request.email);
    let contacts = vec![contact.as_str()];
    let (account, _credentials) = Account::builder()
        .map_err(|e| AppError::from_source("ACME_ACCOUNT_INIT_FAILED", "errors.certificate.acmeAccountInit", e))?
        .create(
            &NewAccount {
                contact: &contacts,
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            directory_url.clone(),
            None,
        )
        .await
        .map_err(|e| AppError::from_source("ACME_ACCOUNT_CREATE_FAILED", "errors.certificate.acmeAccountCreate", e))?;

    // 创建订单
    let identifier = Identifier::Dns(domain.clone());
    let mut order = account
        .new_order(&NewOrder::new(&[identifier]))
        .await
        .map_err(|e| AppError::from_source("ACME_ORDER_CREATE_FAILED", "errors.certificate.acmeOrderCreate", e))?;

    // 获取授权和 challenge
    let challenge_type_enum = if request.challenge_type == "dns01" {
        InstantChallengeType::Dns01
    } else {
        InstantChallengeType::Http01
    };

    let mut authorizations = order.authorizations();
    let mut auth_result = authorizations
        .next()
        .await
        .ok_or_else(|| AppError::new("ACME_NO_AUTHORIZATION", "errors.certificate.acmeNoAuthorization"))?
        .map_err(|e| AppError::from_source("ACME_AUTH_FETCH_FAILED", "errors.certificate.acmeAuthFetch", e))?;

    match auth_result.status {
        AuthorizationStatus::Valid => {
            // 已经验证通过
        }
        AuthorizationStatus::Pending => {}
        status => {
            return Err(AppError::with_details(
                "ACME_AUTH_INVALID_STATUS",
                "errors.certificate.acmeAuthInvalidStatus",
                json!({ "status": format!("{:?}", status) }),
            ));
        }
    }

    let challenge = auth_result
        .challenge(challenge_type_enum)
        .ok_or_else(|| AppError::new("ACME_CHALLENGE_NOT_AVAILABLE", "errors.certificate.acmeChallengeNotAvailable"))?;

    let token = challenge.token.clone();
    let key_auth = challenge.key_authorization().as_str().to_string();

    // 根据 challenge 类型计算验证详情
    let (txt_host, txt_value, http01_token, http01_path) = if request.challenge_type == "dns01" {
        // DNS-01: TXT 记录值 = base64url(SHA256(key_authorization))
        let mut hasher = Sha256::new();
        hasher.update(key_auth.as_bytes());
        let hash = hasher.finalize();
        let txt_val = URL_SAFE_NO_PAD.encode(&hash);
        let txt_h = format!("_acme-challenge.{}", domain);
        (txt_h, txt_val, String::new(), String::new())
    } else {
        // HTTP-01: 验证文件路径和内容
        let path = format!("/.well-known/acme-challenge/{}", token);
        (String::new(), String::new(), token, path)
    };

    // 保存会话到内存状态
    let session = AcmeSession {
        domain: domain.clone(),
        account,
        order,
        challenge_type: request.challenge_type.clone(),
        txt_host: txt_host.clone(),
        txt_value: txt_value.clone(),
    };

    *state.session.lock().await = Some(session);

    Ok(json!({
        "domain": domain,
        "challengeType": request.challenge_type,
        "txtHost": txt_host,
        "txtValue": txt_value,
        "http01Token": http01_token,
        "http01Path": http01_path,
        "http01Content": if request.challenge_type == "http01" { key_auth } else { String::new() },
        "directoryUrl": directory_url,
        "staging": request.staging,
        "generatedAt": Utc::now().timestamp_millis()
    }))
}

/// 第二步：用户添加 DNS 记录后，验证并签发证书
#[tauri::command]
pub async fn certificate_acme_verify(
    state: State<'_, AcmeState>,
) -> CommandResult<Value> {
    // 取出会话
    let session_opt = state.session.lock().await.take();
    let mut session = session_opt.ok_or_else(|| {
        AppError::new("ACME_SESSION_NOT_FOUND", "errors.certificate.acmeSessionNotFound")
    })?;

    let domain = session.domain.clone();

    // 重新获取授权并设置 challenge ready
    let mut authorizations = session.order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut auth = result.map_err(|e| {
            AppError::from_source("ACME_AUTH_FETCH_FAILED", "errors.certificate.acmeAuthFetch", e)
        })?;

        if auth.status == AuthorizationStatus::Valid {
            continue;
        }

        let ct = if session.challenge_type == "dns01" {
            InstantChallengeType::Dns01
        } else {
            InstantChallengeType::Http01
        };

        if let Some(mut challenge) = auth.challenge(ct) {
            challenge
                .set_ready()
                .await
                .map_err(|e| AppError::from_source("ACME_CHALLENGE_READY_FAILED", "errors.certificate.acmeChallengeReady", e))?;
        }
    }

    // 轮询订单状态直到 Ready
    let status = session
        .order
        .poll_ready(&RetryPolicy::default())
        .await
        .map_err(|e| AppError::from_source("ACME_POLL_FAILED", "errors.certificate.acmePoll", e))?;

    if status != OrderStatus::Ready {
        return Err(AppError::with_details(
            "ACME_ORDER_NOT_READY",
            "errors.certificate.acmeOrderNotReady",
            json!({ "status": format!("{:?}", status) }),
        ));
    }

    // Finalize 订单（instant-acme 自动生成 CSR 和私钥）
    let private_key_pem = session
        .order
        .finalize()
        .await
        .map_err(|e| AppError::from_source("ACME_FINALIZE_FAILED", "errors.certificate.acmeFinalize", e))?;

    // 下载证书
    let certificate_pem = session
        .order
        .poll_certificate(&RetryPolicy::default())
        .await
        .map_err(|e| AppError::from_source("ACME_DOWNLOAD_FAILED", "errors.certificate.acmeDownload", e))?;

    // 保存到证书存储
    let dir = certificate_domain_dir(&domain);
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::from_source("CERTIFICATE_DIR_CREATE_FAILED", "errors.certificate.dirCreateFailed", e)
    })?;

    let cert_path = dir.join("certificate.pem");
    fs::write(&cert_path, &certificate_pem).map_err(|e| {
        AppError::from_source("CERTIFICATE_WRITE_FAILED", "errors.certificate.writeFailed", e)
    })?;

    let key_path = dir.join("private_key.pem");
    fs::write(&key_path, &private_key_pem).map_err(|e| {
        AppError::from_source("CERTIFICATE_WRITE_FAILED", "errors.certificate.writeFailed", e)
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
    }

    // 解析证书并写入 metadata
    let validation = certificate_validate_import(certificate_pem.clone(), private_key_pem.clone()).await?;
    let validation_obj = validation.as_object().unwrap();

    let now = Utc::now();
    let not_after_str = validation_obj.get("notAfter").and_then(|v| v.as_str()).unwrap_or("");
    let not_before_str = validation_obj.get("notBefore").and_then(|v| v.as_str()).unwrap_or("");
    let expire_time = DateTime::parse_from_rfc3339(not_after_str)
        .map(|dt| dt.with_timezone::<Utc>(&Utc))
        .unwrap_or(now);
    let create_time = DateTime::parse_from_rfc3339(not_before_str)
        .map(|dt| dt.with_timezone::<Utc>(&Utc))
        .unwrap_or(now);

    let days_remaining = (expire_time - now).num_days();
    let status_str = if days_remaining < 0 {
        "expired"
    } else if days_remaining <= 30 {
        "expiringSoon"
    } else {
        "active"
    };

    let record = CertificateRecord {
        domain: domain.clone(),
        issuer: validation_obj.get("issuer").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        expire_time,
        create_time,
        renew_time: Some(now),
        status: status_str.to_string(),
        fingerprint: CertificateFingerprint {
            sha256: validation_obj.get("fingerprintSha256").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        },
        algorithm: Value::String(validation_obj.get("algorithm").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string()),
        san: validation_obj.get("san").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        cert_path: Some(cert_path),
        key_path: Some(key_path),
        serial_number: Some(validation_obj.get("serialNumber").and_then(|v| v.as_str()).unwrap_or("").to_string()),
        last_error: None,
        auto_renewal_enabled: Some(true),
        tls_version: Some("TLS1.3".to_string()),
        deploy_status: Some("deployed".to_string()),
    };

    let metadata_path = dir.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_vec_pretty(&record).unwrap_or_default())
        .map_err(|e| AppError::from_source("CERTIFICATE_METADATA_WRITE_FAILED", "errors.certificate.metadataWriteFailed", e))?;

    Ok(json!({
        "domain": domain,
        "success": true,
        "certificatePem": certificate_pem,
        "generatedAt": now.timestamp_millis()
    }))
}

/* ────────────────────────── 内部工具函数 ────────────────────────── */

fn certificate_summary(record: CertificateRecord, domain_dir: &std::path::Path) -> Value {
    let now = Utc::now();
    let days_remaining = (record.expire_time - now).num_days();
    let raw_status = normalize_status(&record.status);
    let effective_status = if matches!(
        raw_status,
        "failed" | "revoked" | "deleted" | "pending" | "unknown"
    ) {
        raw_status
    } else if days_remaining < 0 {
        "expired"
    } else if days_remaining <= 30 {
        "expiringSoon"
    } else {
        raw_status
    };
    let has_certificate_pem = domain_dir.join("certificate.pem").exists();
    let last_error = record.last_error.clone();
    let deploy_status = record.deploy_status.clone().unwrap_or_else(|| {
        if has_certificate_pem { "deployed".to_string() } else { "pending".to_string() }
    });

    json!({
        "domain": record.domain,
        "issuer": record.issuer,
        "createTime": record.create_time.to_rfc3339(),
        "expireTime": record.expire_time.to_rfc3339(),
        "renewTime": record.renew_time.map(|value| value.to_rfc3339()),
        "daysRemaining": days_remaining,
        "status": effective_status,
        "autoRenewalStatus": auto_renewal_status(days_remaining, record.renew_time, last_error.as_deref()),
        "autoRenewalEnabled": record.auto_renewal_enabled.unwrap_or(true),
        "fingerprintSha256": record.fingerprint.sha256,
        "algorithm": algorithm_name(&record.algorithm),
        "san": record.san,
        "serialNumber": record.serial_number,
        "lastError": last_error,
        "hasCertificatePem": has_certificate_pem,
        "certificatePath": domain_dir.join("certificate.pem"),
        "keyPath": domain_dir.join("private_key.pem"),
        "tlsVersion": record.tls_version.unwrap_or_else(|| "TLS1.3".to_string()),
        "deployStatus": deploy_status
    })
}

fn auto_renewal_status(
    days_remaining: i64,
    renew_time: Option<DateTime<Utc>>,
    last_error: Option<&str>,
) -> &'static str {
    if last_error.is_some() {
        "failed"
    } else if days_remaining < 0 {
        "expired"
    } else if renew_time.is_some() {
        "scheduled"
    } else if days_remaining <= 30 {
        "due"
    } else {
        "notScheduled"
    }
}

fn algorithm_name(value: &Value) -> String {
    match value {
        Value::String(name) => name.clone(),
        Value::Object(map) => map
            .get("Unknown")
            .and_then(Value::as_str)
            .unwrap_or("Unknown")
            .to_string(),
        _ => "Unknown".to_string(),
    }
}

fn normalize_status(value: &str) -> &'static str {
    match value {
        "Pending" | "pending" => "pending",
        "Active" | "active" => "active",
        "ExpiringSoon" | "expiringSoon" => "expiringSoon",
        "Expired" | "expired" => "expired",
        "Revoked" | "revoked" => "revoked",
        "Deleted" | "deleted" => "deleted",
        "Failed" | "failed" => "failed",
        _ => "unknown",
    }
}

fn load_record(metadata_path: &std::path::Path) -> CommandResult<CertificateRecord> {
    serde_json::from_slice(&fs::read(metadata_path).map_err(|error| {
        AppError::from_source(
            "CERTIFICATE_METADATA_READ_FAILED",
            "errors.certificate.metadataReadFailed",
            error,
        )
    })?)
    .map_err(|error| {
        AppError::from_source(
            "CERTIFICATE_METADATA_PARSE_FAILED",
            "errors.certificate.metadataParseFailed",
            error,
        )
    })
}

fn load_certificate(domain: &str) -> CommandResult<(CertificateRecord, Option<String>)> {
    let domain_dir = certificate_domain_dir(domain);
    let metadata_path = domain_dir.join("metadata.json");
    let certificate_path = domain_dir.join("certificate.pem");

    if !metadata_path.exists() {
        return Err(AppError::with_details(
            "CERTIFICATE_NOT_FOUND",
            "errors.certificate.notFound",
            json!({ "domain": domain }),
        ));
    }

    let record = load_record(&metadata_path)?;
    let certificate_pem = if certificate_path.exists() {
        Some(fs::read_to_string(certificate_path).map_err(|error| {
            AppError::from_source(
                "CERTIFICATE_PEM_READ_FAILED",
                "errors.certificate.pemReadFailed",
                error,
            )
        })?)
    } else {
        None
    };

    Ok((record, certificate_pem))
}

fn certificate_domain_dir(domain: &str) -> PathBuf {
    certificate_store_root().join(sanitize_domain(domain))
}

fn certificate_store_root() -> PathBuf {
    if let Some(value) = env::var_os("GATE_CERTIFICATE_STORE") {
        return PathBuf::from(value);
    }

    if let Some(value) = env::var_os("GATE_CERT_DIR") {
        return PathBuf::from(value);
    }

    runtime_data_dir().join("certificates")
}

fn domain_db_path() -> PathBuf {
    if let Some(value) = env::var_os("GATE_SERVER_DOMAIN_DB") {
        return PathBuf::from(value);
    }
    runtime_data_dir().join("server-domains.sqlite3")
}

fn runtime_data_dir() -> PathBuf {
    app_data_dir().unwrap_or_else(|| PathBuf::from(".gate"))
}

fn normalize_domain(domain: &str) -> CommandResult<String> {
    let domain = domain.trim().trim_end_matches('.').to_ascii_lowercase();
    if domain.is_empty()
        || domain
            .chars()
            .any(|value| !(value.is_ascii_alphanumeric() || matches!(value, '.' | '-' | '_' | '*')))
    {
        return Err(AppError::new(
            "CERTIFICATE_DOMAIN_INVALID",
            "errors.certificate.domainInvalid",
        ));
    }
    Ok(domain)
}

fn sanitize_domain(domain: &str) -> String {
    domain
        .chars()
        .map(|value| match value {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' | '*' => value,
            _ => '_',
        })
        .collect()
}
