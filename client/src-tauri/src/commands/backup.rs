use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tauri::State;

use crate::{
    project::{project_store_path, ProjectWorkspaceState},
    runtime::{
        certificate_store_root, domain_store_path, runtime_data_dir, runtime_store_path,
        ClientRuntimeState,
    },
};

const BACKUP_SCHEMA_VERSION: u32 = 1;
const BACKUP_PRODUCT: &str = "Gate";
const BACKUP_FILE_NAME: &str = "gate-backup.zip";
const MAX_BACKUP_SIZE: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupManifest {
    product: String,
    schema_version: u32,
    app_version: String,
    created_at: String,
    created_at_ms: i64,
    contents: BackupContents,
    files: BTreeMap<String, String>,
    security: BackupSecurity,
    notes: Vec<String>,
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
    certificate_private_keys_included: bool,
    certificate_pem_included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CertificateMetadataBackup {
    domain: String,
    file: String,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct ZipInputEntry {
    name: String,
    data: Vec<u8>,
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
) -> Result<Value, String> {
    let destination_path = backup_destination(destination);
    let runtime_snapshot = runtime_state.backup_snapshot().await?;
    let project_count = project_state.list()?.len();
    let certificates = collect_certificate_metadata()?;
    let contents = backup_contents(&runtime_snapshot, project_count, certificates.len());
    let mut files = BTreeMap::new();
    files.insert(
        "runtimeDatabase".to_string(),
        "database/client-runtime.json".to_string(),
    );
    files.insert(
        "projectDatabase".to_string(),
        "database/projects.sqlite3".to_string(),
    );
    files.insert(
        "domainDatabase".to_string(),
        "database/domains.sqlite3".to_string(),
    );
    files.insert(
        "certificateMetadata".to_string(),
        "certificate-metadata/certificates.json".to_string(),
    );

    let manifest = BackupManifest {
        product: BACKUP_PRODUCT.to_string(),
        schema_version: BACKUP_SCHEMA_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: Utc::now().to_rfc3339(),
        created_at_ms: Utc::now().timestamp_millis(),
        contents: contents.clone(),
        files,
        security: BackupSecurity {
            certificate_private_keys_included: false,
            certificate_pem_included: false,
        },
        notes: vec![
            "证书备份只包含 metadata，不包含 private_key.pem 或 certificate.pem。".to_string(),
            "恢复后 Runtime 保持停止，需要用户手动重新连接服务器和启动 Tunnel。".to_string(),
        ],
    };

    let mut entries = vec![
        ZipInputEntry {
            name: "backup.json".to_string(),
            data: serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
        },
        ZipInputEntry {
            name: "database/client-runtime.json".to_string(),
            data: serde_json::to_vec_pretty(&runtime_snapshot)
                .map_err(|error| error.to_string())?,
        },
        ZipInputEntry {
            name: "certificate-metadata/certificates.json".to_string(),
            data: serde_json::to_vec_pretty(&certificates).map_err(|error| error.to_string())?,
        },
    ];

    push_file_entry(
        &mut entries,
        "database/projects.sqlite3",
        project_store_path(),
    )?;
    push_file_entry(
        &mut entries,
        "database/domains.sqlite3",
        domain_store_path(),
    )?;

    for certificate in &certificates {
        entries.push(ZipInputEntry {
            name: format!("certificate-metadata/{}", certificate.file),
            data: serde_json::to_vec_pretty(&certificate.metadata)
                .map_err(|error| error.to_string())?,
        });
    }

    let archive = write_zip_archive(&entries)?;
    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&destination_path, &archive).map_err(|error| error.to_string())?;

    Ok(json!({
        "path": destination_path.display().to_string(),
        "fileName": destination_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(BACKUP_FILE_NAME),
        "size": archive.len(),
        "createdAt": manifest.created_at,
        "contents": contents,
        "entries": entries.len()
    }))
}

#[tauri::command]
pub async fn backup_preview(path: String) -> Result<Value, String> {
    let archive = read_backup_archive(Path::new(&path))?;
    let manifest = read_manifest(&archive)?;
    validate_manifest(&manifest)?;

    Ok(json!({
        "valid": true,
        "path": path,
        "product": manifest.product,
        "schemaVersion": manifest.schema_version,
        "appVersion": manifest.app_version,
        "createdAt": manifest.created_at,
        "contents": manifest.contents,
        "security": manifest.security,
        "notes": manifest.notes,
        "entries": archive.keys().cloned().collect::<Vec<_>>()
    }))
}

#[tauri::command]
pub async fn backup_restore(
    runtime_state: State<'_, ClientRuntimeState>,
    path: String,
) -> Result<Value, String> {
    let archive = read_backup_archive(Path::new(&path))?;
    let manifest = read_manifest(&archive)?;
    validate_manifest(&manifest)?;
    let runtime_snapshot = read_runtime_snapshot(&archive)?;
    let certificate_metadata = read_certificate_metadata(&archive)?;

    runtime_state.stop_for_restore().await?;
    let rollback = capture_rollback(&certificate_metadata)?;
    let result = apply_restore_archive(
        &runtime_state,
        &archive,
        runtime_snapshot,
        &certificate_metadata,
    )
    .await;

    if let Err(error) = result {
        let rollback_message = match restore_rollback(&rollback) {
            Ok(()) => {
                if let Ok(bytes) = fs::read(runtime_store_path()) {
                    if let Ok(snapshot) = serde_json::from_slice::<Value>(&bytes) {
                        let _ = runtime_state.restore_runtime_snapshot(snapshot).await;
                    }
                }
                cleanup_rollback(&rollback);
                "已回滚到恢复前的数据文件".to_string()
            }
            Err(rollback_error) => format!("回滚失败：{rollback_error}"),
        };
        return Err(format!("恢复失败：{error}。{rollback_message}"));
    }

    cleanup_rollback(&rollback);

    Ok(json!({
        "ok": true,
        "restoredAt": Utc::now().timestamp_millis(),
        "contents": manifest.contents,
        "message": "备份已恢复，Runtime 已停止，请手动重新连接服务器。"
    }))
}

async fn apply_restore_archive(
    runtime_state: &ClientRuntimeState,
    archive: &BTreeMap<String, Vec<u8>>,
    runtime_snapshot: Value,
    certificate_metadata: &[CertificateMetadataBackup],
) -> Result<(), String> {
    replace_file(
        &project_store_path(),
        archive.get("database/projects.sqlite3").map(Vec::as_slice),
    )?;
    replace_file(
        &domain_store_path(),
        archive.get("database/domains.sqlite3").map(Vec::as_slice),
    )?;
    restore_certificate_metadata(certificate_metadata)?;
    runtime_state
        .restore_runtime_snapshot(runtime_snapshot)
        .await?;
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

fn object_len(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_object)
        .map(|items| items.len())
        .unwrap_or_default()
}

fn push_file_entry(
    entries: &mut Vec<ZipInputEntry>,
    name: &str,
    path: PathBuf,
) -> Result<(), String> {
    if path.exists() {
        entries.push(ZipInputEntry {
            name: name.to_string(),
            data: fs::read(path).map_err(|error| error.to_string())?,
        });
    }
    Ok(())
}

fn collect_certificate_metadata() -> Result<Vec<CertificateMetadataBackup>, String> {
    let root = certificate_store_root();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut certificates = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }

        let metadata_path = entry.path().join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }

        let mut metadata: Value =
            serde_json::from_slice(&fs::read(metadata_path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
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

fn read_backup_archive(path: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > MAX_BACKUP_SIZE {
        return Err(format!(
            "备份文件过大：{} MB，当前最大支持 128 MB",
            metadata.len() / 1024 / 1024
        ));
    }
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    read_zip_archive(&bytes)
}

fn read_manifest(archive: &BTreeMap<String, Vec<u8>>) -> Result<BackupManifest, String> {
    let bytes = archive
        .get("backup.json")
        .ok_or_else(|| "备份文件缺少 backup.json".to_string())?;
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

fn validate_manifest(manifest: &BackupManifest) -> Result<(), String> {
    if manifest.product != BACKUP_PRODUCT {
        return Err("备份文件不是 Gate 产品备份".to_string());
    }
    if manifest.schema_version == 0 || manifest.schema_version > BACKUP_SCHEMA_VERSION {
        return Err(format!(
            "备份版本 {} 不受支持，当前支持到 {}",
            manifest.schema_version, BACKUP_SCHEMA_VERSION
        ));
    }
    Ok(())
}

fn read_runtime_snapshot(archive: &BTreeMap<String, Vec<u8>>) -> Result<Value, String> {
    let bytes = archive
        .get("database/client-runtime.json")
        .ok_or_else(|| "备份缺少 Runtime 数据库快照".to_string())?;
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

fn read_certificate_metadata(
    archive: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<CertificateMetadataBackup>, String> {
    let Some(bytes) = archive.get("certificate-metadata/certificates.json") else {
        return Ok(Vec::new());
    };
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

fn capture_rollback(
    certificate_metadata: &[CertificateMetadataBackup],
) -> Result<Vec<RollbackFile>, String> {
    let rollback_dir = runtime_data_dir().join(format!(
        ".restore-rollback-{}",
        Utc::now().timestamp_millis()
    ));
    fs::create_dir_all(&rollback_dir).map_err(|error| error.to_string())?;

    let mut targets = vec![
        runtime_store_path(),
        project_store_path(),
        domain_store_path(),
    ];
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
            fs::copy(&target, &backup).map_err(|error| error.to_string())?;
        }
        rollback.push(RollbackFile {
            target,
            backup,
            existed,
        });
    }

    Ok(rollback)
}

fn restore_rollback(files: &[RollbackFile]) -> Result<(), String> {
    for file in files {
        if file.existed {
            if let Some(parent) = file.target.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            fs::copy(&file.backup, &file.target).map_err(|error| error.to_string())?;
        } else if file.target.exists() {
            fs::remove_file(&file.target).map_err(|error| error.to_string())?;
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

fn replace_file(path: &Path, data: Option<&[u8]>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    if let Some(data) = data {
        fs::write(path, data).map_err(|error| error.to_string())
    } else if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())
    } else {
        Ok(())
    }
}

fn restore_certificate_metadata(certificates: &[CertificateMetadataBackup]) -> Result<(), String> {
    // 恢复证书 metadata 前先清空旧 metadata，避免备份外的旧证书记录混入恢复结果。
    clear_certificate_metadata()?;

    for certificate in certificates {
        let Some(target) = certificate_metadata_target(certificate) else {
            continue;
        };
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let bytes =
            serde_json::to_vec_pretty(&certificate.metadata).map_err(|error| error.to_string())?;
        fs::write(target, bytes).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn clear_certificate_metadata() -> Result<(), String> {
    for target in existing_certificate_metadata_targets()? {
        fs::remove_file(target).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn existing_certificate_metadata_targets() -> Result<Vec<PathBuf>, String> {
    let root = certificate_store_root();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut targets = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry
            .file_type()
            .map_err(|error| error.to_string())?
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

fn write_zip_archive(entries: &[ZipInputEntry]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut central = Vec::new();

    for entry in entries {
        validate_zip_name(&entry.name)?;
        let name = entry.name.as_bytes();
        let crc = crc32(&entry.data);
        let size = u32_len(entry.data.len(), "ZIP 条目内容")?;
        let name_len = u16_len(name.len(), "ZIP 条目名称")?;
        let local_offset = u32_len(output.len(), "ZIP 偏移")?;

        write_u32(&mut output, 0x0403_4b50);
        write_u16(&mut output, 20);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u32(&mut output, crc);
        write_u32(&mut output, size);
        write_u32(&mut output, size);
        write_u16(&mut output, name_len);
        write_u16(&mut output, 0);
        output.extend_from_slice(name);
        output.extend_from_slice(&entry.data);

        write_u32(&mut central, 0x0201_4b50);
        write_u16(&mut central, 20);
        write_u16(&mut central, 20);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u32(&mut central, crc);
        write_u32(&mut central, size);
        write_u32(&mut central, size);
        write_u16(&mut central, name_len);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u32(&mut central, 0);
        write_u32(&mut central, local_offset);
        central.extend_from_slice(name);
    }

    let central_offset = u32_len(output.len(), "ZIP 中央目录偏移")?;
    let central_size = u32_len(central.len(), "ZIP 中央目录大小")?;
    let entry_count = u16_len(entries.len(), "ZIP 条目数量")?;
    output.extend_from_slice(&central);
    write_u32(&mut output, 0x0605_4b50);
    write_u16(&mut output, 0);
    write_u16(&mut output, 0);
    write_u16(&mut output, entry_count);
    write_u16(&mut output, entry_count);
    write_u32(&mut output, central_size);
    write_u32(&mut output, central_offset);
    write_u16(&mut output, 0);

    Ok(output)
}

fn read_zip_archive(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let eocd = find_eocd(bytes).ok_or_else(|| "备份文件不是有效 ZIP".to_string())?;
    let total_entries = read_u16(bytes, eocd + 10)? as usize;
    let central_size = read_u32(bytes, eocd + 12)? as usize;
    let central_offset = read_u32(bytes, eocd + 16)? as usize;
    if central_offset
        .checked_add(central_size)
        .filter(|end| *end <= bytes.len())
        .is_none()
    {
        return Err("ZIP 中央目录越界".to_string());
    }

    let mut cursor = central_offset;
    let mut entries = BTreeMap::new();
    for _ in 0..total_entries {
        if read_u32(bytes, cursor)? != 0x0201_4b50 {
            return Err("ZIP 中央目录损坏".to_string());
        }
        let method = read_u16(bytes, cursor + 10)?;
        if method != 0 {
            return Err("当前仅支持未压缩 ZIP 备份".to_string());
        }
        let expected_crc = read_u32(bytes, cursor + 16)?;
        let compressed_size = read_u32(bytes, cursor + 20)? as usize;
        let uncompressed_size = read_u32(bytes, cursor + 24)? as usize;
        let name_len = read_u16(bytes, cursor + 28)? as usize;
        let extra_len = read_u16(bytes, cursor + 30)? as usize;
        let comment_len = read_u16(bytes, cursor + 32)? as usize;
        let local_offset = read_u32(bytes, cursor + 42)? as usize;
        let name_start = cursor + 46;
        let name_end = checked_end(name_start, name_len, bytes.len())?;
        let name = std::str::from_utf8(&bytes[name_start..name_end])
            .map_err(|error| error.to_string())?
            .to_string();
        validate_zip_name(&name)?;

        let data = read_zip_local_data(
            bytes,
            local_offset,
            compressed_size,
            uncompressed_size,
            expected_crc,
        )?;
        entries.insert(name, data);
        cursor = checked_end(name_end, extra_len + comment_len, bytes.len())?;
    }

    Ok(entries)
}

fn read_zip_local_data(
    bytes: &[u8],
    local_offset: usize,
    compressed_size: usize,
    uncompressed_size: usize,
    expected_crc: u32,
) -> Result<Vec<u8>, String> {
    if compressed_size != uncompressed_size {
        return Err("ZIP 条目大小不一致".to_string());
    }
    if read_u32(bytes, local_offset)? != 0x0403_4b50 {
        return Err("ZIP 本地文件头损坏".to_string());
    }
    let method = read_u16(bytes, local_offset + 8)?;
    if method != 0 {
        return Err("当前仅支持未压缩 ZIP 备份".to_string());
    }
    let name_len = read_u16(bytes, local_offset + 26)? as usize;
    let extra_len = read_u16(bytes, local_offset + 28)? as usize;
    let data_start = checked_end(local_offset + 30, name_len + extra_len, bytes.len())?;
    let data_end = checked_end(data_start, compressed_size, bytes.len())?;
    let data = bytes[data_start..data_end].to_vec();
    if crc32(&data) != expected_crc {
        return Err("ZIP 条目校验失败".to_string());
    }
    Ok(data)
}

fn find_eocd(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 22 {
        return None;
    }
    let start = bytes.len().saturating_sub(66_000);
    (start..=bytes.len() - 4)
        .rev()
        .find(|index| bytes[*index..*index + 4] == [0x50, 0x4b, 0x05, 0x06])
}

fn validate_zip_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with('/')
        || name.contains('\\')
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("ZIP 条目路径不安全：{name}"));
    }
    Ok(())
}

fn checked_end(start: usize, len: usize, total: usize) -> Result<usize, String> {
    start
        .checked_add(len)
        .filter(|end| *end <= total)
        .ok_or_else(|| "ZIP 数据越界".to_string())
}

fn u16_len(value: usize, label: &str) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("{label} 超出 ZIP 限制"))
}

fn u32_len(value: usize, label: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("{label} 超出 ZIP 限制"))
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let end = checked_end(offset, 2, bytes.len())?;
    Ok(u16::from_le_bytes(
        bytes[offset..end]
            .try_into()
            .map_err(|_| "ZIP u16 损坏".to_string())?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let end = checked_end(offset, 4, bytes.len())?;
    Ok(u32::from_le_bytes(
        bytes[offset..end]
            .try_into()
            .map_err(|_| "ZIP u32 损坏".to_string())?,
    ))
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}
