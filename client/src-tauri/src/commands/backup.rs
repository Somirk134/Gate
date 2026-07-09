use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::State;

use crate::{
    commands::error::{AppError, CommandResult},
    project::{project_store_path, ProjectWorkspaceState},
    runtime::{certificate_store_root, domain_store_path, runtime_data_dir, ClientRuntimeState},
};

const BACKUP_SCHEMA_VERSION: u32 = 1;
const BACKUP_VERSION: &str = "0.9";
const BACKUP_PRODUCT: &str = "Gate";
const BACKUP_FILE_NAME: &str = "gate-v0.9.gatebackup";
const MAX_BACKUP_SIZE: u64 = 128 * 1024 * 1024;
const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GateBackupFile {
    product: String,
    version: String,
    schema_version: u32,
    app_version: String,
    created_at: String,
    created_at_ms: i64,
    contents: BackupContents,
    security: BackupSecurity,
    notes: Vec<String>,
    runtime_snapshot: Value,
    projects: Vec<Value>,
    project_database: Option<EncodedFile>,
    domain_database: Option<EncodedFile>,
    certificate_metadata: Vec<CertificateMetadataBackup>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupContents {
    projects: usize,
    servers: usize,
    tunnels: usize,
    domains: usize,
    certificates: usize,
    settings: usize,
    runtime_config: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupSecurity {
    server_tokens_included: bool,
    certificate_private_keys_included: bool,
    certificate_pem_included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncodedFile {
    path: String,
    encoding: String,
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CertificateMetadataBackup {
    domain: String,
    file: String,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct RollbackFile {
    target: PathBuf,
    backup: PathBuf,
    existed: bool,
}

#[tauri::command]
pub async fn backup_export(
    runtime_state: State<'_, ClientRuntimeState>,
    project_state: State<'_, ProjectWorkspaceState>,
    destination: Option<String>,
) -> CommandResult<Value> {
    let destination_path = backup_destination(destination);
    let mut runtime_snapshot = runtime_state.backup_snapshot().await.map_err(|source| {
        AppError::from_source(
            "BACKUP_RUNTIME_SNAPSHOT_FAILED",
            "errors.backup.runtimeSnapshotFailed",
            source,
        )
    })?;
    sanitize_runtime_snapshot(&mut runtime_snapshot);

    let projects = project_state
        .list()
        .map_err(|source| {
            AppError::from_source(
                "BACKUP_PROJECT_READ_FAILED",
                "errors.backup.projectReadFailed",
                source,
            )
        })?
        .into_iter()
        .map(|project| {
            serde_json::to_value(project).map_err(|source| {
                AppError::from_source(
                    "BACKUP_PROJECT_SERIALIZE_FAILED",
                    "errors.backup.projectSerializeFailed",
                    source,
                )
            })
        })
        .collect::<CommandResult<Vec<_>>>()?;

    let certificates = collect_certificate_metadata()?;
    let contents = backup_contents(&runtime_snapshot, projects.len(), certificates.len());
    let created_at = Utc::now();
    let backup = GateBackupFile {
        product: BACKUP_PRODUCT.to_string(),
        version: BACKUP_VERSION.to_string(),
        schema_version: BACKUP_SCHEMA_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: created_at.to_rfc3339(),
        created_at_ms: created_at.timestamp_millis(),
        contents: contents.clone(),
        security: BackupSecurity {
            server_tokens_included: false,
            certificate_private_keys_included: false,
            certificate_pem_included: false,
        },
        notes: vec![
            "backup.notes.tokensExcluded".to_string(),
            "backup.notes.certificateSecretsExcluded".to_string(),
            "backup.notes.manualReconnectRequired".to_string(),
        ],
        runtime_snapshot,
        projects,
        project_database: read_encoded_file(project_store_path(), "database/projects.sqlite3")?,
        domain_database: read_encoded_file(domain_store_path(), "database/domains.sqlite3")?,
        certificate_metadata: certificates,
    };

    let bytes = serde_json::to_vec_pretty(&backup).map_err(|source| {
        AppError::from_source(
            "BACKUP_SERIALIZE_FAILED",
            "errors.backup.serializeFailed",
            source,
        )
    })?;

    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            AppError::from_source(
                "BACKUP_DIRECTORY_CREATE_FAILED",
                "errors.backup.directoryCreateFailed",
                source,
            )
        })?;
    }
    fs::write(&destination_path, &bytes).map_err(|source| {
        AppError::from_source("BACKUP_WRITE_FAILED", "errors.backup.writeFailed", source)
    })?;

    Ok(json!({
        "path": destination_path.display().to_string(),
        "fileName": destination_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(BACKUP_FILE_NAME),
        "size": bytes.len(),
        "createdAt": backup.created_at,
        "contents": contents,
        "entries": backup_entries(&backup).len(),
        "security": backup.security,
    }))
}

#[tauri::command]
pub async fn backup_preview(path: String) -> CommandResult<Value> {
    let backup = read_backup_file(Path::new(&path))?;
    validate_backup(&backup)?;

    Ok(json!({
        "valid": true,
        "path": path,
        "product": backup.product,
        "version": backup.version,
        "schemaVersion": backup.schema_version,
        "appVersion": backup.app_version,
        "createdAt": backup.created_at,
        "contents": backup.contents,
        "security": backup.security,
        "notes": backup.notes,
        "entries": backup_entries(&backup),
    }))
}

#[tauri::command]
pub async fn backup_restore(
    runtime_state: State<'_, ClientRuntimeState>,
    path: String,
) -> CommandResult<Value> {
    let backup = read_backup_file(Path::new(&path))?;
    validate_backup(&backup)?;

    runtime_state.stop_for_restore().await.map_err(|source| {
        AppError::from_source(
            "RESTORE_RUNTIME_STOP_FAILED",
            "errors.backup.runtimeStopFailed",
            source,
        )
    })?;
    let rollback = capture_rollback(&backup.certificate_metadata)?;
    let result = apply_restore_archive(&runtime_state, &backup).await;

    if let Err(error) = result {
        let rollback_ok = restore_rollback(&rollback).is_ok();
        if rollback_ok {
            cleanup_rollback(&rollback);
        }
        return Err(AppError::with_details(
            "RESTORE_APPLY_FAILED",
            "errors.backup.restoreApplyFailed",
            json!({
                "source": error.code,
                "messageKey": error.message_key,
                "rollbackOk": rollback_ok,
            }),
        ));
    }

    cleanup_rollback(&rollback);

    Ok(json!({
        "ok": true,
        "restoredAt": Utc::now().timestamp_millis(),
        "contents": backup.contents,
        "messageKey": "backup.restore.successMessage",
    }))
}

async fn apply_restore_archive(
    runtime_state: &ClientRuntimeState,
    backup: &GateBackupFile,
) -> CommandResult<()> {
    write_encoded_file(&project_store_path(), backup.project_database.as_ref())?;
    write_encoded_file(&domain_store_path(), backup.domain_database.as_ref())?;
    restore_certificate_metadata(&backup.certificate_metadata)?;
    runtime_state
        .restore_runtime_snapshot(backup.runtime_snapshot.clone())
        .await
        .map_err(|source| {
            AppError::from_source(
                "RESTORE_RUNTIME_APPLY_FAILED",
                "errors.backup.runtimeApplyFailed",
                source,
            )
        })?;
    Ok(())
}

fn backup_destination(destination: Option<String>) -> PathBuf {
    let Some(raw) = destination.filter(|value| !value.trim().is_empty()) else {
        return runtime_data_dir().join(BACKUP_FILE_NAME);
    };

    let path = PathBuf::from(raw);
    if path.is_dir() {
        path.join(BACKUP_FILE_NAME)
    } else {
        path
    }
}

fn backup_contents(
    runtime_snapshot: &Value,
    project_count: usize,
    certificate_count: usize,
) -> BackupContents {
    let settings = object_len(runtime_snapshot, "config");
    BackupContents {
        projects: project_count,
        servers: object_len(runtime_snapshot, "servers"),
        tunnels: object_len(runtime_snapshot, "tunnels"),
        domains: object_len(runtime_snapshot, "domains"),
        certificates: certificate_count,
        settings,
        runtime_config: settings,
    }
}

fn backup_entries(backup: &GateBackupFile) -> Vec<String> {
    let mut entries = vec![
        "backup.json".to_string(),
        "runtimeSnapshot".to_string(),
        "projects".to_string(),
        "certificateMetadata".to_string(),
    ];
    if backup.project_database.is_some() {
        entries.push("projectDatabase".to_string());
    }
    if backup.domain_database.is_some() {
        entries.push("domainDatabase".to_string());
    }
    entries
}

fn object_len(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_object)
        .map(|items| items.len())
        .unwrap_or_default()
}

fn read_encoded_file(path: PathBuf, backup_path: &str) -> CommandResult<Option<EncodedFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|source| {
        AppError::from_source(
            "BACKUP_FILE_READ_FAILED",
            "errors.backup.fileReadFailed",
            source,
        )
    })?;
    Ok(Some(EncodedFile {
        path: backup_path.to_string(),
        encoding: "base64".to_string(),
        data: base64_encode(&bytes),
    }))
}

fn write_encoded_file(path: &Path, file: Option<&EncodedFile>) -> CommandResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            AppError::from_source(
                "RESTORE_DIRECTORY_CREATE_FAILED",
                "errors.backup.directoryCreateFailed",
                source,
            )
        })?;
    }

    if let Some(file) = file {
        if file.encoding != "base64" {
            return Err(AppError::with_details(
                "RESTORE_ENCODING_UNSUPPORTED",
                "errors.backup.encodingUnsupported",
                json!({ "encoding": file.encoding }),
            ));
        }
        let bytes = base64_decode(&file.data)?;
        fs::write(path, bytes).map_err(|source| {
            AppError::from_source(
                "RESTORE_FILE_WRITE_FAILED",
                "errors.backup.restoreFileWriteFailed",
                source,
            )
        })
    } else if path.exists() {
        fs::remove_file(path).map_err(|source| {
            AppError::from_source(
                "RESTORE_FILE_REMOVE_FAILED",
                "errors.backup.restoreFileRemoveFailed",
                source,
            )
        })
    } else {
        Ok(())
    }
}

fn read_backup_file(path: &Path) -> CommandResult<GateBackupFile> {
    let metadata = fs::metadata(path).map_err(|source| {
        AppError::from_source("BACKUP_READ_FAILED", "errors.backup.readFailed", source)
    })?;
    if metadata.len() > MAX_BACKUP_SIZE {
        return Err(AppError::with_details(
            "BACKUP_TOO_LARGE",
            "errors.backup.tooLarge",
            json!({
                "sizeMb": metadata.len() / 1024 / 1024,
                "maxMb": MAX_BACKUP_SIZE / 1024 / 1024,
            }),
        ));
    }
    let bytes = fs::read(path).map_err(|source| {
        AppError::from_source("BACKUP_READ_FAILED", "errors.backup.readFailed", source)
    })?;
    serde_json::from_slice(&bytes).map_err(|source| {
        AppError::from_source("BACKUP_PARSE_FAILED", "errors.backup.parseFailed", source)
    })
}

fn validate_backup(backup: &GateBackupFile) -> CommandResult<()> {
    if backup.product != BACKUP_PRODUCT {
        return Err(AppError::with_details(
            "BACKUP_PRODUCT_MISMATCH",
            "errors.backup.productMismatch",
            json!({ "product": backup.product }),
        ));
    }
    if backup.version != BACKUP_VERSION {
        return Err(AppError::with_details(
            "BACKUP_VERSION_UNSUPPORTED",
            "errors.backup.versionUnsupported",
            json!({
                "version": backup.version,
                "supported": BACKUP_VERSION,
            }),
        ));
    }
    if backup.schema_version == 0 || backup.schema_version > BACKUP_SCHEMA_VERSION {
        return Err(AppError::with_details(
            "BACKUP_SCHEMA_UNSUPPORTED",
            "errors.backup.schemaUnsupported",
            json!({
                "schemaVersion": backup.schema_version,
                "supported": BACKUP_SCHEMA_VERSION,
            }),
        ));
    }
    Ok(())
}

fn sanitize_runtime_snapshot(value: &mut Value) {
    if let Some(servers) = value.get_mut("servers").and_then(Value::as_object_mut) {
        for server in servers.values_mut() {
            if let Some(server) = server.as_object_mut() {
                server.insert("token".to_string(), Value::String(String::new()));
                server.insert("lastError".to_string(), Value::Null);
                server.insert("sessionId".to_string(), Value::Null);
                server.insert(
                    "status".to_string(),
                    Value::String("disconnected".to_string()),
                );
            }
        }
    }
    if let Some(logs) = value.get_mut("logs") {
        *logs = Value::Array(Vec::new());
    }
    if let Some(active_server_id) = value.get_mut("activeServerId") {
        *active_server_id = Value::Null;
    }
}

fn collect_certificate_metadata() -> CommandResult<Vec<CertificateMetadataBackup>> {
    let root = certificate_store_root();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut certificates = Vec::new();
    for entry in fs::read_dir(root).map_err(|source| {
        AppError::from_source(
            "CERTIFICATE_METADATA_READ_FAILED",
            "errors.backup.certificateMetadataReadFailed",
            source,
        )
    })? {
        let entry = entry.map_err(|source| {
            AppError::from_source(
                "CERTIFICATE_METADATA_READ_FAILED",
                "errors.backup.certificateMetadataReadFailed",
                source,
            )
        })?;
        if !entry
            .file_type()
            .map_err(|source| {
                AppError::from_source(
                    "CERTIFICATE_METADATA_READ_FAILED",
                    "errors.backup.certificateMetadataReadFailed",
                    source,
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

        let mut metadata: Value =
            serde_json::from_slice(&fs::read(metadata_path).map_err(|source| {
                AppError::from_source(
                    "CERTIFICATE_METADATA_READ_FAILED",
                    "errors.backup.certificateMetadataReadFailed",
                    source,
                )
            })?)
            .map_err(|source| {
                AppError::from_source(
                    "CERTIFICATE_METADATA_PARSE_FAILED",
                    "errors.backup.certificateMetadataParseFailed",
                    source,
                )
            })?;
        sanitize_certificate_metadata(&mut metadata);
        let domain = metadata
            .get("domain")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| entry.file_name().to_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".to_string());
        certificates.push(CertificateMetadataBackup {
            file: format!("{}/metadata.json", sanitize_path_segment(&domain)),
            domain,
            metadata,
        });
    }

    certificates.sort_by(|left, right| left.domain.cmp(&right.domain));
    Ok(certificates)
}

fn sanitize_certificate_metadata(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for key in [
                "privateKey",
                "private_key",
                "keyPem",
                "key_pem",
                "certificatePem",
                "certificate_pem",
                "certPem",
                "cert_pem",
            ] {
                map.remove(key);
            }
            for child in map.values_mut() {
                sanitize_certificate_metadata(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                sanitize_certificate_metadata(item);
            }
        }
        _ => {}
    }
}

fn sanitize_path_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => ch,
            _ => '_',
        })
        .collect()
}

fn capture_rollback(
    certificate_metadata: &[CertificateMetadataBackup],
) -> CommandResult<Vec<RollbackFile>> {
    let rollback_dir = runtime_data_dir().join(format!(
        ".restore-rollback-{}",
        Utc::now().timestamp_millis()
    ));
    fs::create_dir_all(&rollback_dir).map_err(|source| {
        AppError::from_source(
            "RESTORE_ROLLBACK_CREATE_FAILED",
            "errors.backup.rollbackCreateFailed",
            source,
        )
    })?;

    let mut targets = vec![project_store_path(), domain_store_path()];
    targets.extend(existing_certificate_metadata_targets()?);
    for certificate in certificate_metadata {
        if let Some(target) = certificate_metadata_target(certificate) {
            targets.push(target);
        }
    }
    targets.sort();
    targets.dedup();

    let mut rollback = Vec::new();
    for (index, target) in targets.into_iter().enumerate() {
        let backup = rollback_dir.join(format!("file-{index}"));
        let existed = target.exists();
        if existed {
            fs::copy(&target, &backup).map_err(|source| {
                AppError::from_source(
                    "RESTORE_ROLLBACK_COPY_FAILED",
                    "errors.backup.rollbackCreateFailed",
                    source,
                )
            })?;
        }
        rollback.push(RollbackFile {
            target,
            backup,
            existed,
        });
    }

    Ok(rollback)
}

fn restore_rollback(files: &[RollbackFile]) -> CommandResult<()> {
    for file in files {
        if file.existed {
            if let Some(parent) = file.target.parent() {
                fs::create_dir_all(parent).map_err(|source| {
                    AppError::from_source(
                        "RESTORE_ROLLBACK_FAILED",
                        "errors.backup.rollbackFailed",
                        source,
                    )
                })?;
            }
            fs::copy(&file.backup, &file.target).map_err(|source| {
                AppError::from_source(
                    "RESTORE_ROLLBACK_FAILED",
                    "errors.backup.rollbackFailed",
                    source,
                )
            })?;
        } else if file.target.exists() {
            fs::remove_file(&file.target).map_err(|source| {
                AppError::from_source(
                    "RESTORE_ROLLBACK_FAILED",
                    "errors.backup.rollbackFailed",
                    source,
                )
            })?;
        }
    }
    Ok(())
}

fn cleanup_rollback(files: &[RollbackFile]) {
    let Some(directory) = files
        .first()
        .and_then(|file| file.backup.parent())
        .map(Path::to_path_buf)
    else {
        return;
    };
    let _ = fs::remove_dir_all(directory);
}

fn restore_certificate_metadata(certificates: &[CertificateMetadataBackup]) -> CommandResult<()> {
    // 恢复前先清理旧 metadata，避免备份外的旧证书记录混入恢复结果。
    clear_certificate_metadata()?;

    for certificate in certificates {
        let Some(target) = certificate_metadata_target(certificate) else {
            continue;
        };
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                AppError::from_source(
                    "RESTORE_DIRECTORY_CREATE_FAILED",
                    "errors.backup.directoryCreateFailed",
                    source,
                )
            })?;
        }
        let bytes = serde_json::to_vec_pretty(&certificate.metadata).map_err(|source| {
            AppError::from_source(
                "RESTORE_CERTIFICATE_METADATA_SERIALIZE_FAILED",
                "errors.backup.certificateMetadataSerializeFailed",
                source,
            )
        })?;
        fs::write(target, bytes).map_err(|source| {
            AppError::from_source(
                "RESTORE_CERTIFICATE_METADATA_WRITE_FAILED",
                "errors.backup.certificateMetadataWriteFailed",
                source,
            )
        })?;
    }
    Ok(())
}

fn clear_certificate_metadata() -> CommandResult<()> {
    for target in existing_certificate_metadata_targets()? {
        fs::remove_file(target).map_err(|source| {
            AppError::from_source(
                "RESTORE_CERTIFICATE_METADATA_REMOVE_FAILED",
                "errors.backup.certificateMetadataRemoveFailed",
                source,
            )
        })?;
    }
    Ok(())
}

fn existing_certificate_metadata_targets() -> CommandResult<Vec<PathBuf>> {
    let root = certificate_store_root();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut targets = Vec::new();
    for entry in fs::read_dir(root).map_err(|source| {
        AppError::from_source(
            "CERTIFICATE_METADATA_READ_FAILED",
            "errors.backup.certificateMetadataReadFailed",
            source,
        )
    })? {
        let entry = entry.map_err(|source| {
            AppError::from_source(
                "CERTIFICATE_METADATA_READ_FAILED",
                "errors.backup.certificateMetadataReadFailed",
                source,
            )
        })?;
        if !entry
            .file_type()
            .map_err(|source| {
                AppError::from_source(
                    "CERTIFICATE_METADATA_READ_FAILED",
                    "errors.backup.certificateMetadataReadFailed",
                    source,
                )
            })?
            .is_dir()
        {
            continue;
        }
        let metadata_path = entry.path().join("metadata.json");
        if metadata_path.exists() {
            targets.push(metadata_path);
        }
    }
    Ok(targets)
}

fn certificate_metadata_target(certificate: &CertificateMetadataBackup) -> Option<PathBuf> {
    let normalized = certificate.file.replace('\\', "/");
    let parts = normalized.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0].is_empty() || parts[1] != "metadata.json" {
        return None;
    }
    if parts[0] != sanitize_path_segment(parts[0]) {
        return None;
    }
    Some(
        certificate_store_root()
            .join(parts[0])
            .join("metadata.json"),
    )
}

fn base64_encode(data: &[u8]) -> String {
    let mut output = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);

        output.push(BASE64_TABLE[(first >> 2) as usize] as char);
        output.push(BASE64_TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(
                BASE64_TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char,
            );
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(BASE64_TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

fn base64_decode(input: &str) -> CommandResult<Vec<u8>> {
    let bytes = input.trim().as_bytes();
    if bytes.len() % 4 != 0 {
        return Err(AppError::new(
            "RESTORE_BASE64_INVALID",
            "errors.backup.base64Invalid",
        ));
    }

    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let mut values = [0_u8; 4];
        let mut padding = 0;
        for (index, byte) in chunk.iter().enumerate() {
            if *byte == b'=' {
                padding += 1;
                values[index] = 0;
            } else if let Some(position) =
                BASE64_TABLE.iter().position(|candidate| candidate == byte)
            {
                values[index] = position as u8;
            } else {
                return Err(AppError::new(
                    "RESTORE_BASE64_INVALID",
                    "errors.backup.base64Invalid",
                ));
            }
        }

        output.push((values[0] << 2) | (values[1] >> 4));
        if padding < 2 {
            output.push((values[1] << 4) | (values[2] >> 2));
        }
        if padding == 0 {
            output.push((values[2] << 6) | values[3]);
        }
    }
    Ok(output)
}
