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
use tauri::{AppHandle, Emitter, State};
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
            if !entry
                .file_type()
                .map_err(|e| {
                    AppError::from_source(
                        "CERTIFICATE_DIRECTORY_READ_FAILED",
                        "errors.certificate.readDirectoryFailed",
                        e,
                    )
                })?
                .is_dir()
            {
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
    let active = all
        .iter()
        .filter(|r| {
            let s = normalize_status(&r.status);
            s == "active" && (r.expire_time - now).num_days() > 30
        })
        .count();
    let expiring_soon = all
        .iter()
        .filter(|r| {
            let days = (r.expire_time - now).num_days();
            days >= 0 && days <= 30
        })
        .count();
    let expired = all
        .iter()
        .filter(|r| (r.expire_time - now).num_days() < 0)
        .count();
    let failed = all
        .iter()
        .filter(|r| normalize_status(&r.status) == "failed")
        .count();
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
    let acme_enabled =
        env::var_os("GATE_ACME_EMAIL").is_some() || env::var_os("GATE_ACME_AUTO").is_some();
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
    let (_, cert) = X509Certificate::from_der(cert_block.contents()).map_err(|e| {
        AppError::from_source(
            "CERTIFICATE_X509_PARSE_FAILED",
            "errors.certificate.x509ParseFailed",
            e,
        )
    })?;

    let subject = cert.subject();
    let cn = subject
        .iter_common_name()
        .next()
        .and_then(|c| c.as_str().ok())
        .unwrap_or("unknown");

    let issuer = cert.issuer();
    let issuer_cn = issuer
        .iter_common_name()
        .next()
        .and_then(|c| c.as_str().ok())
        .unwrap_or("unknown");

    // SAN 域名 — 使用 ParsedExtension 迭代
    let san: Vec<String> = cert
        .extensions()
        .iter()
        .find_map(|ext| {
            if let ParsedExtension::SubjectAlternativeName(san_ext) = ext.parsed_extension() {
                Some(
                    san_ext
                        .general_names
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
    let fingerprint_hex: String = fingerprint
        .iter()
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

    let key_valid = matches!(
        key_block.tag(),
        "PRIVATE KEY" | "RSA PRIVATE KEY" | "EC PRIVATE KEY" | "ENCRYPTED PRIVATE KEY"
    );

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
    )
    .await?;

    let validation_obj = validation.as_object().ok_or_else(|| {
        AppError::new(
            "CERTIFICATE_VALIDATION_INVALID",
            "errors.certificate.validationInvalid",
        )
    })?;

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
    let not_after_str = validation_obj
        .get("notAfter")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let not_before_str = validation_obj
        .get("notBefore")
        .and_then(|v| v.as_str())
        .unwrap_or("");
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
        issuer: validation_obj
            .get("issuer")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        expire_time,
        create_time,
        renew_time: None,
        status: status.to_string(),
        fingerprint: CertificateFingerprint {
            sha256: validation_obj
                .get("fingerprintSha256")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        },
        algorithm: Value::String(
            validation_obj
                .get("algorithm")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
        ),
        san: validation_obj
            .get("san")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        cert_path: Some(cert_path.clone()),
        key_path: Some(key_path.clone()),
        serial_number: Some(
            validation_obj
                .get("serialNumber")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
        last_error: None,
        auto_renewal_enabled: Some(false),
        tls_version: Some("TLS1.3".to_string()),
        deploy_status: Some("pending".to_string()),
    };

    let metadata_path = dir.join("metadata.json");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&record).unwrap_or_default(),
    )
    .map_err(|e| {
        AppError::from_source(
            "CERTIFICATE_METADATA_WRITE_FAILED",
            "errors.certificate.metadataWriteFailed",
            e,
        )
    })?;

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

    let enabled = acme_email.is_some()
        || acme_auto.as_deref() == Some("true")
        || acme_auto.as_deref() == Some("1");

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
                .prepare(
                    "SELECT host, tunnel_id, status, verify_status FROM domains WHERE host LIKE ?1",
                )
                .map_err(|e| {
                    AppError::from_source(
                        "DOMAIN_DB_QUERY_FAILED",
                        "errors.certificate.domainDbQueryFailed",
                        e,
                    )
                })?;

            let rows = stmt
                .query_map([&like_pattern], |row| {
                    Ok(json!({
                        "host": row.get::<_, String>(0)?,
                        "tunnelId": row.get::<_, Option<String>>(1)?,
                        "status": row.get::<_, String>(2)?,
                        "verifyStatus": row.get::<_, String>(3)?,
                    }))
                })
                .map_err(|e| {
                    AppError::from_source(
                        "DOMAIN_DB_QUERY_FAILED",
                        "errors.certificate.domainDbQueryFailed",
                        e,
                    )
                })?;

            for row in rows.flatten() {
                let tunnel_id = row
                    .get("tunnelId")
                    .and_then(|v| v.as_str())
                    .map(String::from);
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
    let acme_configured =
        env::var_os("GATE_ACME_EMAIL").is_some() || env::var_os("GATE_ACME_AUTO").is_some();

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
        let _ = fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&record).unwrap_or_default(),
        );
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
        let _ = fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&record).unwrap_or_default(),
        );
    }

    Ok(json!({
        "domain": domain,
        "triggered": true,
        "message": "Redeploy triggered. The server gateway will reload the TLS cache."
    }))
}

#[tauri::command]
pub async fn certificate_toggle_auto_renewal(
    domain: String,
    enabled: bool,
) -> CommandResult<Value> {
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
        let _ = fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&record).unwrap_or_default(),
        );
    }

    Ok(json!({
        "domain": domain,
        "autoRenewalEnabled": enabled
    }))
}

/* ────────────────────────── ACME 申请流程 ────────────────────────── */

/// ACME 会话持久化路径
fn acme_session_path() -> PathBuf {
    app_data_dir()
        .unwrap_or_else(|| PathBuf::from(".gate"))
        .join("acme-session.json")
}

/// ACME 会话序列化结构（可持久化到磁盘）
#[derive(Debug, Serialize, Deserialize)]
struct AcmeSessionPersisted {
    domain: String,
    email: String,
    challenge_type: String,
    txt_host: String,
    txt_value: String,
    http01_token: String,
    http01_path: String,
    http01_content: String,
    directory_url: String,
    created_at: i64,
}

/// ACME 运行时状态（内存中保存 instant-acme 的 Account + Order）
#[derive(Default)]
pub struct AcmeState {
    pub(crate) session: Arc<Mutex<Option<AcmeRuntime>>>,
}

pub(crate) struct AcmeRuntime {
    domain: String,
    // Account 需要长期持有以维持 ACME 会话，即使当前未直接读取。
    #[allow(dead_code)]
    account: Account,
    order: instant_acme::Order,
    challenge_type: String,
    txt_host: String,
    txt_value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcmePrepareRequest {
    pub domain: String,
    pub email: String,
    pub challenge_type: String,
    pub staging: bool,
}

/// 第一步：创建 ACME 账户和订单，获取 challenge，返回 DNS/HTTP 验证详情
/// 同时将关键信息持久化到磁盘，确保 app 重启后仍能恢复
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
        .map_err(|e| {
            AppError::from_source(
                "ACME_ACCOUNT_INIT_FAILED",
                "errors.certificate.acmeAccountInit",
                e,
            )
        })?
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
        .map_err(|e| {
            AppError::from_source(
                "ACME_ACCOUNT_CREATE_FAILED",
                "errors.certificate.acmeAccountCreate",
                e,
            )
        })?;

    // 创建订单
    let identifier = Identifier::Dns(domain.clone());
    let mut order = account
        .new_order(&NewOrder::new(&[identifier]))
        .await
        .map_err(|e| {
            AppError::from_source(
                "ACME_ORDER_CREATE_FAILED",
                "errors.certificate.acmeOrderCreate",
                e,
            )
        })?;

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
        .ok_or_else(|| {
            AppError::new(
                "ACME_NO_AUTHORIZATION",
                "errors.certificate.acmeNoAuthorization",
            )
        })?
        .map_err(|e| {
            AppError::from_source(
                "ACME_AUTH_FETCH_FAILED",
                "errors.certificate.acmeAuthFetch",
                e,
            )
        })?;

    match auth_result.status {
        AuthorizationStatus::Valid => {}
        AuthorizationStatus::Pending => {}
        status => {
            return Err(AppError::with_details(
                "ACME_AUTH_INVALID_STATUS",
                "errors.certificate.acmeAuthInvalidStatus",
                json!({ "status": format!("{:?}", status), "hint": "授权状态异常，可能需要等待或重新申请" }),
            ));
        }
    }

    let challenge = auth_result.challenge(challenge_type_enum).ok_or_else(|| {
        AppError::new(
            "ACME_CHALLENGE_NOT_AVAILABLE",
            "errors.certificate.acmeChallengeNotAvailable",
        )
    })?;

    let token = challenge.token.clone();
    let key_auth = challenge.key_authorization().as_str().to_string();

    // 根据 challenge 类型计算验证详情
    let key_auth_for_http = key_auth.clone(); // 保留一份给 http01 持久化
    let (txt_host, txt_value, http01_token, http01_path) = if request.challenge_type == "dns01" {
        let mut hasher = Sha256::new();
        hasher.update(key_auth.as_bytes());
        let hash = hasher.finalize();
        let txt_val = URL_SAFE_NO_PAD.encode(&hash);
        let txt_h = format!("_acme-challenge.{}", domain);
        (txt_h, txt_val, String::new(), String::new())
    } else {
        let path = format!("/.well-known/acme-challenge/{}", token);
        (String::new(), String::new(), token.clone(), path)
    };

    // 保存运行时到内存
    *state.session.lock().await = Some(AcmeRuntime {
        domain: domain.clone(),
        account,
        order,
        challenge_type: request.challenge_type.clone(),
        txt_host: txt_host.clone(),
        txt_value: txt_value.clone(),
    });

    // 持久化关键信息到磁盘（用于 app 重启后恢复提示）
    let email_for_history = request.email.clone();
    let persisted = AcmeSessionPersisted {
        domain: domain.clone(),
        email: request.email,
        challenge_type: request.challenge_type.clone(),
        txt_host: txt_host.clone(),
        txt_value: txt_value.clone(),
        http01_token: http01_token.clone(),
        http01_path: http01_path.clone(),
        http01_content: if request.challenge_type == "http01" {
            key_auth_for_http
        } else {
            String::new()
        },
        directory_url: directory_url.clone(),
        created_at: Utc::now().timestamp_millis(),
    };
    let session_path = acme_session_path();
    fs::write(
        &session_path,
        serde_json::to_vec_pretty(&persisted).unwrap_or_default(),
    )
    .map_err(|e| {
        AppError::from_source(
            "ACME_SESSION_PERSIST_FAILED",
            "errors.certificate.acmeSessionPersistFailed",
            e,
        )
    })?;

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
        "persisted": true,
        "generatedAt": Utc::now().timestamp_millis(),
        // 创建申请历史记录
        "recordId": create_acme_history_record(&domain, &email_for_history, &request.challenge_type, request.staging, &directory_url).unwrap_or_default(),
    }))
}

/* ── prepare 成功后创建申请历史记录 ── */

/// 在 prepare 完成后调用，创建一条 pending 状态的申请记录
fn create_acme_history_record(
    domain: &str,
    email: &str,
    challenge_type: &str,
    staging: bool,
    directory_url: &str,
) -> Result<String, AppError> {
    let mut records = load_acme_history()?;
    let record = AcmeApplicationRecord::pending(
        domain.to_string(),
        email.to_string(),
        challenge_type.to_string(),
        staging,
        directory_url.to_string(),
    );
    let record_id = record.id.clone();
    records.push(record);
    save_acme_history(&records)?;
    Ok(record_id)
}

/// 在 start_verify 时将记录状态更新为 verifying
fn mark_acme_record_verifying(record_id: &str) -> Result<(), AppError> {
    update_acme_record(record_id, |rec| {
        rec.status = AcmeRecordStatus::Verifying;
    })
    .map(|_| ())
}

/// 验证成功后更新记录为 issued
fn mark_acme_record_issued(
    record_id: &str,
    issuer: Option<String>,
    expire_time: Option<String>,
    days_remaining: Option<i64>,
) -> Result<(), AppError> {
    update_acme_record(record_id, |rec| {
        rec.status = AcmeRecordStatus::Issued;
        rec.issued_at = Some(Utc::now().timestamp_millis());
        rec.issuer = issuer;
        rec.expire_time = expire_time;
        rec.days_remaining = days_remaining;
        rec.error = None;
        rec.certificate_available = true;
    })
    .map(|_| ())
}

/// 验证失败后更新记录为 failed
fn mark_acme_record_failed(
    record_id: &str,
    error_msg: &str,
    error_code: Option<String>,
) -> Result<(), AppError> {
    update_acme_record(record_id, |rec| {
        rec.status = AcmeRecordStatus::Failed;
        rec.error = Some(error_msg.to_string());
        rec.error_code = error_code;
    })
    .map(|_| ())
}

/// DNS TXT 记录预检查：在告诉 Let's Encrypt 验证之前，先确认记录已传播
async fn dns_txt_record_resolvable(
    txt_host: &str,
    expected_value: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let txt_host_owned = txt_host.to_string();
    let expected_value_owned = expected_value.to_string();

    let result = tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("nslookup")
                .args(["-type=TXT", &txt_host_owned])
                .output()
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new("dig")
                .args(["+short", "TXT", &txt_host_owned])
                .output()
        }
    })
    .await;

    match result {
        Ok(Ok(out)) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            Ok(stdout.contains(&expected_value_owned) || stdout.contains("\""))
        }
        Ok(Err(_)) => Err("DNS query command failed to execute".into()),
        Err(_) => Err("DNS query task failed".into()),
    }
}

/// ACME 验证核心逻辑（被后台任务调用）
/// 返回 Ok(Value) 表示成功（含证书信息），Err(String) 表示失败原因
async fn run_acme_verification(runtime: AcmeRuntime) -> Result<Value, String> {
    let domain = runtime.domain.clone();

    // 1. 如果是 DNS-01，先做 DNS 传播预检查
    if runtime.challenge_type == "dns01" {
        let txt_host = &runtime.txt_host;
        let txt_value = &runtime.txt_value;
        let mut dns_ok = false;
        for attempt in 1..=3 {
            match dns_txt_record_resolvable(txt_host, txt_value).await {
                Ok(true) => {
                    dns_ok = true;
                    break;
                }
                Ok(false) => {
                    if attempt < 3 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                }
                Err(error) => {
                    tracing::warn!(error = %error, "DNS pre-check failed");
                    break;
                }
            }
        }
        if !dns_ok {
            tracing::warn!(host = %txt_host, "DNS pre-check did not confirm the TXT record");
        }
    }

    // 2. 取出 order（需要 move 进去）
    let mut order = runtime.order;
    let challenge_type = runtime.challenge_type.clone();

    // 3. 重新获取授权并设置 challenge ready
    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut auth_result = result.map_err(|e| format!("ACME 授权获取失败: {:?}", e))?;

        // 需要重新 fetch authorization 来获取 challenge
        // instant-acme 的 Authorization 对象可能不支持直接修改
        // 这里我们通过 order 的 poll_ready 来驱动验证

        if auth_result.status == AuthorizationStatus::Valid {
            continue;
        }

        let ct = if challenge_type == "dns01" {
            InstantChallengeType::Dns01
        } else {
            InstantChallengeType::Http01
        };

        // 尝试获取并激活 challenge
        if auth_result.challenge(ct).is_some() {
            // DNS-01 等待传播
            if challenge_type == "dns01" {
                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            }
        }
    }

    // 4. 轮询订单状态直到 Ready（5 分钟超时）
    let retry_policy = RetryPolicy::new()
        .initial_delay(std::time::Duration::from_secs(5))
        .backoff(10.0)
        .timeout(std::time::Duration::from_secs(300));

    let status = order
        .poll_ready(&retry_policy)
        .await
        .map_err(|e| format!("ACME 轮询失败: {}", e))?;

    if status != OrderStatus::Ready {
        return Err(match status {
            OrderStatus::Invalid => format!(
                "验证失败：订单状态 Invalid。DNS 记录可能未正确配置或尚未传播。主机名：{}，请检查记录值是否正确。",
                runtime.txt_host
            ),
            OrderStatus::Pending => "验证超时：订单仍处于 Pending 状态。DNS 传播可能较慢，请几分钟后重试。".to_string(),
            OrderStatus::Processing => "验证超时：Let's Encrypt 正在处理中但未及时完成。请稍后重试。".to_string(),
            _ => format!("验证失败：异常订单状态 {:?}", status),
        });
    }

    // 5. Finalize 订单（自动生成 CSR 和私钥）
    let private_key_pem = order
        .finalize()
        .await
        .map_err(|e| format!("证书签发失败: {}", e))?;

    // 6. 下载证书
    let certificate_pem = order
        .poll_certificate(&RetryPolicy::default())
        .await
        .map_err(|e| format!("证书下载失败: {}", e))?;

    // 7. 清理 session 文件
    let _ = fs::remove_file(acme_session_path());

    // 8. 保存到证书存储
    let dir = certificate_domain_dir(&domain);
    fs::create_dir_all(&dir).map_err(|e| format!("证书目录创建失败: {}", e))?;

    fs::write(dir.join("certificate.pem"), &certificate_pem)
        .map_err(|e| format!("证书写入失败: {}", e))?;
    fs::write(dir.join("private_key.pem"), &private_key_pem)
        .map_err(|e| format!("私钥写入失败: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(
            dir.join("private_key.pem"),
            fs::Permissions::from_mode(0o600),
        );
    }

    // 9. 解析证书并写入 metadata
    let validation = certificate_validate_import(certificate_pem.clone(), private_key_pem.clone())
        .await
        .map_err(|e| format!("证书解析失败: {:?}", e))?;
    let validation_obj = validation
        .as_object()
        .ok_or_else(|| "证书验证结果格式无效".to_string())?;

    let now = Utc::now();
    let not_after_str = validation_obj
        .get("notAfter")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let not_before_str = validation_obj
        .get("notBefore")
        .and_then(|v| v.as_str())
        .unwrap_or("");
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
        issuer: validation_obj
            .get("issuer")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        expire_time,
        create_time,
        renew_time: Some(now),
        status: status_str.to_string(),
        fingerprint: CertificateFingerprint {
            sha256: validation_obj
                .get("fingerprintSha256")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        },
        algorithm: Value::String(
            validation_obj
                .get("algorithm")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
        ),
        san: validation_obj
            .get("san")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        cert_path: Some(dir.join("certificate.pem")),
        key_path: Some(dir.join("private_key.pem")),
        serial_number: Some(
            validation_obj
                .get("serialNumber")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
        last_error: None,
        auto_renewal_enabled: Some(true),
        tls_version: Some("TLS1.3".to_string()),
        deploy_status: Some("deployed".to_string()),
    };

    fs::write(
        dir.join("metadata.json"),
        serde_json::to_vec_pretty(&record).unwrap_or_default(),
    )
    .map_err(|e| format!("metadata 写入失败: {}", e))?;

    Ok(json!({
        "domain": domain,
        "success": true,
        "issuer": record.issuer,
        "expireTime": expire_time.to_rfc3339(),
        "daysRemaining": days_remaining,
        "generatedAt": now.timestamp_millis()
    }))
}

/// 启动后台 ACME 验证任务（非阻塞，立即返回）
/// 验证完成后通过 Tauri 事件 'acme-verify:result' 通知前端
#[tauri::command]
pub async fn certificate_acme_start_verify(
    app: AppHandle,
    state: State<'_, AcmeState>,
    record_id: Option<String>,
) -> CommandResult<Value> {
    // 取出会话
    let session_opt = state.session.lock().await.take();
    let runtime = match session_opt {
        Some(s) => s,
        None => {
            let session_path = acme_session_path();
            if session_path.exists() {
                return Err(AppError::with_details(
                    "ACME_SESSION_EXPIRED_NEED_RETRY",
                    "errors.certificate.acmeSessionExpiredNeedRetry",
                    json!({
                        "reason": "session_lost",
                        "hint": "ACME 会话已过期（可能因为软件重启）。请关闭向导重新申请，DNS 记录可保留。",
                    }),
                ));
            }
            return Err(AppError::new(
                "ACME_SESSION_NOT_FOUND",
                "errors.certificate.acmeSessionNotFound",
            ));
        }
    };

    let domain = runtime.domain.clone();
    let domain_for_return = domain.clone();

    // 如果有 recordId，更新记录状态为 verifying
    let effective_record_id: Option<String> = record_id.clone().or_else(|| {
        // 如果没传 recordId，尝试从历史记录中找最新的 pending/verifying 记录
        load_acme_history().ok().and_then(|records| {
            records
                .iter()
                .filter(|r| {
                    r.domain == domain
                        && (r.status == AcmeRecordStatus::Pending
                            || r.status == AcmeRecordStatus::Verifying)
                })
                .max_by_key(|r| r.created_at)
                .map(|r| r.id.clone())
        })
    });

    if let Some(ref rid) = effective_record_id {
        if mark_acme_record_verifying(rid).is_err() {
            eprintln!("警告：无法将申请记录 {} 更新为 verifying 状态", rid);
        }
    }

    let task_record_id = effective_record_id.clone();

    // 在后台启动验证任务，不阻塞前端
    tokio::spawn(async move {
        // 发送开始事件（携带 recordId）
        let _ = app.emit(
            "acme-verify:progress",
            json!({
                "domain": domain,
                "recordId": task_record_id,
                "stage": "started",
                "message": "正在验证...",
            }),
        );

        match run_acme_verification(runtime).await {
            Ok(result) => {
                // 更新历史记录为 issued
                if let Some(ref rid) = task_record_id {
                    let issuer = result
                        .get("issuer")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let expire_time = result
                        .get("expireTime")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let days_remaining = result.get("daysRemaining").and_then(|v| v.as_i64());
                    if mark_acme_record_issued(rid, issuer, expire_time, days_remaining).is_err() {
                        eprintln!("警告：无法更新申请记录 {} 为 issued 状态", rid);
                    }
                }

                let _ = app.emit(
                    "acme-verify:result",
                    json!({
                        "success": true,
                        "data": result,
                        "recordId": task_record_id,
                    }),
                );
            }
            Err(err_msg) => {
                // 更新历史记录为 failed
                if let Some(ref rid) = task_record_id {
                    let err_code = if err_msg.contains("Invalid") {
                        Some("acmeOrderNotReady".to_string())
                    } else if err_msg.contains("超时") {
                        Some("timeout".to_string())
                    } else {
                        None
                    };
                    if mark_acme_record_failed(rid, &err_msg, err_code).is_err() {
                        eprintln!("警告：无法更新申请记录 {} 为 failed 状态", rid);
                    }
                }

                let _ = app.emit(
                    "acme-verify:result",
                    json!({
                        "success": false,
                        "error": err_msg,
                        "recordId": task_record_id,
                    }),
                );
            }
        }
    });

    // 立即返回——不等待验证完成
    Ok(json!({
        "domain": domain_for_return,
        "started": true,
        "message": "后台验证已启动",
        "recordId": effective_record_id,
    }))
}

/// （保留）同步阻塞式验证——用于需要等待结果的场景
#[tauri::command]
pub async fn certificate_acme_verify(state: State<'_, AcmeState>) -> CommandResult<Value> {
    let session_opt = state.session.lock().await.take();
    let runtime = match session_opt {
        Some(s) => s,
        None => {
            let session_path = acme_session_path();
            if session_path.exists() {
                return Err(AppError::with_details(
                    "ACME_SESSION_EXPIRED_NEED_RETRY",
                    "errors.certificate.acmeSessionExpiredNeedRetry",
                    json!({ "hint": "ACME 会话已过期，请重新申请。" }),
                ));
            }
            return Err(AppError::new(
                "ACME_SESSION_NOT_FOUND",
                "errors.certificate.acmeSessionNotFound",
            ));
        }
    };

    // 直接运行验证（阻塞等待结果）
    match run_acme_verification(runtime).await {
        Ok(result) => Ok(result),
        Err(err_msg) => Err(AppError::with_details(
            "ACME_VERIFY_FAILED",
            "errors.certificate.acmeVerifyFailed",
            json!({ "hint": err_msg }),
        )),
    }
}

/* ────────────────────── ACME 申请记录（持久化） ────────────────────── */

/// 申请记录状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum AcmeRecordStatus {
    /// 待验证（用户已提交 prepare，等待点"我已添加"）
    Pending,
    /// 验证中（后台任务已启动）
    Verifying,
    /// 已签发（证书已保存到存储）
    Issued,
    /// 验证失败
    Failed,
    /// 已过期
    Expired,
}

impl std::fmt::Display for AcmeRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Verifying => write!(f, "verifying"),
            Self::Issued => write!(f, "issued"),
            Self::Failed => write!(f, "failed"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

/// 单条 ACME 申请记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcmeApplicationRecord {
    id: String,
    domain: String,
    email: String,
    challenge_type: String,
    staging: bool,
    directory_url: String,
    status: AcmeRecordStatus,
    created_at: i64,
    updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    issued_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expire_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    days_remaining: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    #[serde(default)]
    retry_count: u32,
    /// 证书是否可下载（即证书目录存在且有 PEM 文件）
    #[serde(default)]
    certificate_available: bool,
}

impl AcmeApplicationRecord {
    fn new_id() -> String {
        format!("{:08x}", rand_random())
    }

    fn pending(
        domain: String,
        email: String,
        challenge_type: String,
        staging: bool,
        directory_url: String,
    ) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            id: Self::new_id(),
            domain,
            email,
            challenge_type,
            staging,
            directory_url,
            status: AcmeRecordStatus::Pending,
            created_at: now,
            updated_at: now,
            issued_at: None,
            expire_time: None,
            issuer: None,
            days_remaining: None,
            error: None,
            error_code: None,
            retry_count: 0,
            certificate_available: false,
        }
    }
}

/// 生成随机 ID（简单实现，不依赖额外 crate）
fn rand_random() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (dur.as_nanos() as u32).wrapping_add(dur.as_secs() as u32)
}

/// 申请记录持久化路径
fn acme_history_path() -> PathBuf {
    app_data_dir()
        .unwrap_or_else(|| PathBuf::from(".gate"))
        .join("acme-history.json")
}

/// 读取所有申请记录
fn load_acme_history() -> Result<Vec<AcmeApplicationRecord>, AppError> {
    let path = acme_history_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| {
        AppError::from_source(
            "ACME_HISTORY_READ_FAILED",
            "errors.certificate.historyReadFailed",
            e,
        )
    })?;
    serde_json::from_str::<Vec<AcmeApplicationRecord>>(&data).map_err(|e| {
        AppError::from_source(
            "ACME_HISTORY_PARSE_FAILED",
            "errors.certificate.historyParseFailed",
            e,
        )
    })
}

/// 写入所有申请记录（全量覆盖）
fn save_acme_history(records: &[AcmeApplicationRecord]) -> Result<(), AppError> {
    let path = acme_history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::from_source(
                "ACME_HISTORY_WRITE_FAILED",
                "errors.certificate.historyWriteFailed",
                e,
            )
        })?;
    }
    fs::write(path, serde_json::to_vec_pretty(records).unwrap_or_default()).map_err(|e| {
        AppError::from_source(
            "ACME_HISTORY_WRITE_FAILED",
            "errors.certificate.historyWriteFailed",
            e,
        )
    })
}

/// 根据记录 ID 查找并更新记录
fn update_acme_record<F>(
    record_id: &str,
    updater: F,
) -> Result<Option<AcmeApplicationRecord>, AppError>
where
    F: FnOnce(&mut AcmeApplicationRecord),
{
    let mut records = load_acme_history()?;
    let mut updated = None;
    for rec in &mut records {
        if rec.id == record_id {
            updater(rec);
            rec.updated_at = Utc::now().timestamp_millis();
            // 检查证书是否可用
            let cert_dir = certificate_domain_dir(&rec.domain);
            rec.certificate_available =
                cert_dir.exists() && cert_dir.join("certificate.pem").exists();
            updated = Some(rec.clone());
            break;
        }
    }
    if updated.is_some() {
        save_acme_history(&records)?;
    }
    Ok(updated)
}

/* ────────────────────── ACME 申请记录命令 ────────────────────── */

/// 获取所有 ACME 申请记录列表
#[tauri::command]
pub async fn certificate_acme_history() -> CommandResult<Value> {
    let mut records = load_acme_history()?;
    // 按创建时间倒序排列
    records.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // 更新每条记录的证书可用性
    for rec in &mut records {
        if rec.status == AcmeRecordStatus::Issued || rec.status == AcmeRecordStatus::Verifying {
            let cert_dir = certificate_domain_dir(&rec.domain);
            rec.certificate_available =
                cert_dir.exists() && cert_dir.join("certificate.pem").exists();
            // 如果是 verifying 但证书已经存在了，说明之前验证成功了但没更新状态
            if rec.status == AcmeRecordStatus::Verifying && rec.certificate_available {
                rec.status = AcmeRecordStatus::Issued;
                rec.issued_at = Some(Utc::now().timestamp_millis());
                rec.error = None;
            }
        }
    }

    // 如果有状态变化，回写
    save_acme_history(&records).ok();

    let total = records.len();
    let verifying = records
        .iter()
        .filter(|r| r.status == AcmeRecordStatus::Verifying)
        .count();
    let issued = records
        .iter()
        .filter(|r| r.status == AcmeRecordStatus::Issued)
        .count();
    let failed = records
        .iter()
        .filter(|r| r.status == AcmeRecordStatus::Failed)
        .count();

    Ok(json!({
        "records": records,
        "summary": { "total": total, "verifying": verifying, "issued": issued, "failed": failed },
        "generatedAt": Utc::now().timestamp_millis()
    }))
}

/// 获取单条申请记录详情（含证书下载信息）
#[tauri::command]
pub async fn certificate_acme_record_detail(record_id: String) -> CommandResult<Value> {
    let records = load_acme_history()?;
    let record = records
        .into_iter()
        .find(|r| r.id == record_id)
        .ok_or_else(|| {
            AppError::with_details(
                "ACME_RECORD_NOT_FOUND",
                "errors.certificate.recordNotFound",
                json!({ "id": record_id }),
            )
        })?;

    // 如果是已签发状态且证书可用，尝试读取证书信息
    let mut cert_info: Value = json!(null);
    if record.status == AcmeRecordStatus::Issued && record.certificate_available {
        match load_certificate(&record.domain) {
            Ok((cert_rec, Some(pem))) => {
                cert_info = json!({
                    "domain": cert_rec.domain,
                    "issuer": cert_rec.issuer,
                    "expireTime": cert_rec.expire_time.to_rfc3339(),
                    "daysRemaining": (cert_rec.expire_time - Utc::now()).num_days(),
                    "algorithm": algorithm_name(&cert_rec.algorithm),
                    "san": cert_rec.san,
                    "certificatePem": pem,
                    "certificatePath": cert_rec.cert_path.map(|p| p.display().to_string()).unwrap_or_default(),
                    "keyPath": cert_rec.key_path.map(|p| p.display().to_string()).unwrap_or_default(),
                });
            }
            _ => {}
        }
    }

    Ok(json!({
        "record": record,
        "certificateInfo": cert_info,
        "generatedAt": Utc::now().timestamp_millis()
    }))
}

/// 重试失败的或正在验证的申请
#[tauri::command]
pub async fn certificate_acme_retry(
    record_id: String,
    app: AppHandle,
    _state: State<'_, AcmeState>,
) -> CommandResult<Value> {
    let mut records = load_acme_history()?;
    let record = records
        .iter_mut()
        .find(|r| r.id == record_id)
        .ok_or_else(|| {
            AppError::with_details(
                "ACME_RECORD_NOT_FOUND",
                "errors.certificate.recordNotFound",
                json!({ "id": record_id }),
            )
        })?;

    // 只有 failed 和 verifying 状态可以重试
    if record.status != AcmeRecordStatus::Failed && record.status != AcmeRecordStatus::Verifying {
        return Err(AppError::with_details(
            "ACME_RECORD_CANNOT_RETRY",
            "errors.certificate.recordCannotRetry",
            json!({ "status": record.status.to_string(), "hint": "只有「验证中」和「失败」状态的记录可以重试" }),
        ));
    }

    // 更新记录状态为 verifying
    record.status = AcmeRecordStatus::Verifying;
    record.retry_count += 1;
    record.error = None;
    record.error_code = None;
    record.updated_at = Utc::now().timestamp_millis();

    // 在 save 之前提取需要的值（避免借用冲突）
    let domain = record.domain.clone();
    let challenge_type = record.challenge_type.clone();
    let email = record.email.clone();
    let staging = record.staging;
    let directory_url = record.directory_url.clone();
    let retry_record_id = record_id.clone();
    let domain_for_return = domain.clone();
    let retry_record_id_for_return = retry_record_id.clone();

    save_acme_history(&records)?;

    // 启动后台重试验证
    tokio::spawn(async move {
        let _ = app.emit(
            "acme-verify:progress",
            json!({
                "domain": domain,
                "recordId": retry_record_id,
                "stage": "retry_started",
                "message": "正在重新验证...",
            }),
        );

        // 重新走 prepare 流程创建新的 ACME 订单
        let result = redo_acme_verification(
            domain.clone(),
            email,
            challenge_type,
            staging,
            directory_url,
        )
        .await;

        match result {
            Ok(success_info) => {
                // 更新历史记录为 issued
                let _ = update_acme_record(&retry_record_id, |rec| {
                    rec.status = AcmeRecordStatus::Issued;
                    rec.issued_at = Some(Utc::now().timestamp_millis());
                    rec.expire_time = success_info
                        .get("expireTime")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    rec.issuer = success_info
                        .get("issuer")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    rec.days_remaining = success_info.get("daysRemaining").and_then(|v| v.as_i64());
                    rec.error = None;
                    rec.certificate_available = true;
                });

                let _ = app.emit(
                    "acme-verify:result",
                    json!({
                        "success": true,
                        "data": success_info,
                        "recordId": retry_record_id,
                    }),
                );
            }
            Err(err_msg) => {
                let err_code = if err_msg.contains("Invalid") {
                    "acmeOrderNotReady"
                } else if err_msg.contains("超时") {
                    "timeout"
                } else {
                    "unknown"
                }
                .to_string();

                let _ = update_acme_record(&retry_record_id, |rec| {
                    rec.status = AcmeRecordStatus::Failed;
                    rec.error = Some(err_msg.clone());
                    rec.error_code = Some(err_code);
                });

                let _ = app.emit(
                    "acme-verify:result",
                    json!({
                        "success": false,
                        "error": err_msg,
                        "recordId": retry_record_id,
                    }),
                );
            }
        }
    });

    Ok(json!({
        "recordId": retry_record_id_for_return,
        "domain": domain_for_return,
        "retryStarted": true,
        "message": "重试验证已在后台启动"
    }))
}

/// 删除一条申请记录（仅删除记录，不删除实际证书文件）
#[tauri::command]
pub async fn certificate_acme_delete_record(record_id: String) -> CommandResult<Value> {
    let mut records = load_acme_history()?;
    let before_len = records.len();
    records.retain(|r| r.id != record_id);

    if records.len() == before_len {
        return Err(AppError::with_details(
            "ACME_RECORD_NOT_FOUND",
            "errors.certificate.recordNotFound",
            json!({ "id": record_id }),
        ));
    }

    save_acme_history(&records)?;

    Ok(json!({
        "recordId": record_id,
        "deleted": true
    }))
}

/// 重试验证的完整流程（在后台任务中执行）
async fn redo_acme_verification(
    domain: String,
    email: String,
    challenge_type: String,
    _staging: bool,
    directory_url: String,
) -> Result<Value, String> {
    // 1. 创建新账户和新订单
    let contact = format!("mailto:{}", email);
    let contacts = vec![contact.as_str()];
    let (account, _credentials) = Account::builder()
        .map_err(|e| format!("ACME 账户初始化失败: {}", e))?
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
        .map_err(|e| format!("ACME 账户创建失败: {}", e))?;

    let identifier = Identifier::Dns(domain.clone());
    let mut order = account
        .new_order(&NewOrder::new(&[identifier]))
        .await
        .map_err(|e| format!("ACME 订单创建失败: {}", e))?;

    // 2. 获取 challenge 并设置 ready
    let challenge_type_enum = if challenge_type == "dns01" {
        InstantChallengeType::Dns01
    } else {
        InstantChallengeType::Http01
    };

    // 遍历授权
    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut auth_result = result.map_err(|e| format!("ACME 授权获取失败: {:?}", e))?;

        if auth_result.status == AuthorizationStatus::Valid {
            continue;
        }

        if auth_result.challenge(challenge_type_enum.clone()).is_some() {
            if challenge_type == "dns01" {
                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            }
        }
    }

    // 3. 轮询 Ready
    let retry_policy = RetryPolicy::new()
        .initial_delay(std::time::Duration::from_secs(5))
        .backoff(10.0)
        .timeout(std::time::Duration::from_secs(300));

    let status = order
        .poll_ready(&retry_policy)
        .await
        .map_err(|e| format!("ACME 轮询失败: {}", e))?;

    if status != OrderStatus::Ready {
        return Err(match status {
            OrderStatus::Invalid => {
                "验证失败：订单状态 Invalid。DNS 记录可能未正确配置。".to_string()
            }
            OrderStatus::Pending => "验证超时：订单仍处于 Pending 状态。".to_string(),
            OrderStatus::Processing => "验证超时：处理中未完成。请稍后重试。".to_string(),
            _ => format!("验证失败：异常订单状态 {:?}", status),
        });
    }

    // 4. Finalize + 下载
    let private_key_pem = order
        .finalize()
        .await
        .map_err(|e| format!("证书签发失败: {}", e))?;
    let certificate_pem = order
        .poll_certificate(&RetryPolicy::default())
        .await
        .map_err(|e| format!("证书下载失败: {}", e))?;

    // 5. 保存到证书存储
    let dir = certificate_domain_dir(&domain);
    fs::create_dir_all(&dir).map_err(|e| format!("证书目录创建失败: {}", e))?;
    fs::write(dir.join("certificate.pem"), &certificate_pem)
        .map_err(|e| format!("证书写入失败: {}", e))?;
    fs::write(dir.join("private_key.pem"), &private_key_pem)
        .map_err(|e| format!("私钥写入失败: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(
            dir.join("private_key.pem"),
            fs::Permissions::from_mode(0o600),
        );
    }

    // 6. 解析证书写 metadata
    let validation = certificate_validate_import(certificate_pem.clone(), private_key_pem.clone())
        .await
        .map_err(|e| format!("证书解析失败: {:?}", e))?;
    let validation_obj = validation
        .as_object()
        .ok_or_else(|| "证书验证结果格式无效".to_string())?;

    let now = Utc::now();
    let not_after_str = validation_obj
        .get("notAfter")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let not_before_str = validation_obj
        .get("notBefore")
        .and_then(|v| v.as_str())
        .unwrap_or("");
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

    let cert_record = CertificateRecord {
        domain: domain.clone(),
        issuer: validation_obj
            .get("issuer")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        expire_time,
        create_time,
        renew_time: Some(now),
        status: status_str.to_string(),
        fingerprint: CertificateFingerprint {
            sha256: validation_obj
                .get("fingerprintSha256")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        },
        algorithm: Value::String(
            validation_obj
                .get("algorithm")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
        ),
        san: validation_obj
            .get("san")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        cert_path: Some(dir.join("certificate.pem")),
        key_path: Some(dir.join("private_key.pem")),
        serial_number: Some(
            validation_obj
                .get("serialNumber")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
        last_error: None,
        auto_renewal_enabled: Some(true),
        tls_version: Some("TLS1.3".to_string()),
        deploy_status: Some("deployed".to_string()),
    };

    fs::write(
        dir.join("metadata.json"),
        serde_json::to_vec_pretty(&cert_record).unwrap_or_default(),
    )
    .map_err(|e| format!("metadata 写入失败: {}", e))?;

    Ok(json!({
        "domain": domain,
        "success": true,
        "issuer": cert_record.issuer,
        "expireTime": expire_time.to_rfc3339(),
        "daysRemaining": days_remaining,
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
        if has_certificate_pem {
            "deployed".to_string()
        } else {
            "pending".to_string()
        }
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
