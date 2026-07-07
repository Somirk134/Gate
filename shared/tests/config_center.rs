use gate_shared::config::{
    AppConfig, ConfigPriority, ConfigProvider, ConfigSchema, EnvironmentConfigProvider,
    FileConfigProvider, MemoryConfigProvider, UnifiedConfigCenter,
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn loads_defaults() {
    let config = UnifiedConfigCenter::new().load_config().unwrap();

    assert_eq!(config.version, 1);
    assert_eq!(config.global.app_name, "gate");
    assert_eq!(config.runtime.worker_threads, 4);
    assert_eq!(config.http.max_header_bytes, 64 * 1024);
}

#[test]
fn memory_provider_overrides_defaults() {
    let config = UnifiedConfigCenter::new()
        .with_provider(MemoryConfigProvider::new(
            "test",
            json!({
                "runtime": {
                    "worker_threads": 8,
                    "max_tasks": 8192
                },
                "client": {
                    "theme": "dark"
                }
            }),
        ))
        .load_config()
        .unwrap();

    assert_eq!(config.runtime.worker_threads, 8);
    assert_eq!(config.runtime.max_tasks, 8192);
    assert_eq!(config.client.theme, "dark");
}

#[test]
fn environment_provider_uses_nested_keys() {
    let suffix = Uuid::new_v4().to_string().replace('-', "_");
    let prefix = format!("GATE_TEST_{suffix}");
    std::env::set_var(format!("{prefix}_RUNTIME__WORKER_THREADS"), "12");
    std::env::set_var(format!("{prefix}_CLIENT__AUTO_CONNECT"), "true");

    let config = UnifiedConfigCenter::new()
        .with_provider(EnvironmentConfigProvider::new(prefix.clone()))
        .load_config()
        .unwrap();

    std::env::remove_var(format!("{prefix}_RUNTIME__WORKER_THREADS"));
    std::env::remove_var(format!("{prefix}_CLIENT__AUTO_CONNECT"));

    assert_eq!(config.runtime.worker_threads, 12);
    assert!(config.client.auto_connect);
}

#[test]
fn file_provider_loads_toml_overlay() {
    let path = temp_file("gate-config", "toml");
    fs::write(
        &path,
        r#"
            [global]
            app_name = "gate-test"

            [http]
            listen_addr = "127.0.0.1:18080"
            route_limit = 32
        "#,
    )
    .unwrap();

    let config = UnifiedConfigCenter::new()
        .with_provider(FileConfigProvider::new(path.clone()))
        .load_config()
        .unwrap();
    fs::remove_file(path).unwrap();

    assert_eq!(config.global.app_name, "gate-test");
    assert_eq!(config.http.listen_addr, "127.0.0.1:18080");
    assert_eq!(config.http.route_limit, 32);
}

#[test]
fn cli_priority_wins_over_environment() {
    let suffix = Uuid::new_v4().to_string().replace('-', "_");
    let prefix = format!("GATE_TEST_{suffix}");
    std::env::set_var(format!("{prefix}_CLIENT__THEME"), "light");

    let config = UnifiedConfigCenter::new()
        .with_provider(EnvironmentConfigProvider::new(prefix.clone()))
        .with_provider(
            MemoryConfigProvider::new("cli", json!({ "client": { "theme": "dark" } }))
                .with_priority(ConfigPriority::Cli),
        )
        .load_config()
        .unwrap();

    std::env::remove_var(format!("{prefix}_CLIENT__THEME"));

    assert_eq!(config.client.theme, "dark");
}

#[test]
fn validation_fails_before_startup() {
    let error = UnifiedConfigCenter::new()
        .with_provider(MemoryConfigProvider::new(
            "invalid",
            json!({ "runtime": { "worker_threads": 0 } }),
        ))
        .load_config()
        .unwrap_err();

    assert!(error.to_string().contains("runtime.worker_threads"));
}

#[test]
fn schema_exports_default_config() {
    let schema = ConfigSchema::current();
    let default_json = schema.default_config_json().unwrap();
    let imported = AppConfig::import_json(&default_json).unwrap();

    assert_eq!(imported.version, schema.schema_version);
    assert!(schema.layers.iter().any(|layer| layer.name == "https"));
}

#[test]
fn provider_trait_is_object_safe() {
    let provider: Box<dyn ConfigProvider> =
        Box::new(MemoryConfigProvider::new("memory", json!({ "version": 1 })));
    let document = provider.load().unwrap();

    assert_eq!(document.priority, ConfigPriority::Memory);
}

fn temp_file(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}.{}", Uuid::new_v4(), extension))
}
