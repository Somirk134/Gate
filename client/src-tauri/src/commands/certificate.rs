use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

use crate::commands::error::{AppError, CommandResult};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateFingerprint {
    sha256: String,
}

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

            let record: CertificateRecord =
                serde_json::from_slice(&fs::read(&metadata_path).map_err(|error| {
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
                })?;
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

    json!({
        "domain": record.domain,
        "issuer": record.issuer,
        "createTime": record.create_time.to_rfc3339(),
        "expireTime": record.expire_time.to_rfc3339(),
        "renewTime": record.renew_time.map(|value| value.to_rfc3339()),
        "daysRemaining": days_remaining,
        "status": effective_status,
        "autoRenewalStatus": auto_renewal_status(days_remaining, record.renew_time, last_error.as_deref()),
        "fingerprintSha256": record.fingerprint.sha256,
        "algorithm": algorithm_name(&record.algorithm),
        "san": record.san,
        "serialNumber": record.serial_number,
        "lastError": last_error,
        "hasCertificatePem": has_certificate_pem,
        "certificatePath": domain_dir.join("certificate.pem"),
        "keyPath": domain_dir.join("private_key.pem")
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

    let record = serde_json::from_slice(&fs::read(metadata_path).map_err(|error| {
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
    })?;
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

fn runtime_data_dir() -> PathBuf {
    if let Some(appdata) = env::var_os("APPDATA") {
        return PathBuf::from(appdata).join("Gate");
    }

    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("Gate");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("Gate");
    }

    PathBuf::from(".gate")
}

fn normalize_domain(domain: &str) -> CommandResult<String> {
    let domain = domain.trim().trim_end_matches('.').to_ascii_lowercase();
    if domain.is_empty()
        || domain
            .chars()
            .any(|value| !(value.is_ascii_alphanumeric() || matches!(value, '.' | '-' | '_')))
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
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => value,
            _ => '_',
        })
        .collect()
}
