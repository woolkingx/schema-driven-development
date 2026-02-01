// Schema Registry Implementation
// 生產級別的 Schema 管理系統

use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub type ValidationResult = Result<(), Vec<String>>;

/// Schema Registry - 管理所有 API Schemas
pub struct SchemaRegistry {
    schemas: HashMap<String, JSONSchema>,
    schema_sources: HashMap<String, Value>,
}

impl SchemaRegistry {
    /// 從目錄載入所有 schemas
    pub fn from_directory<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let mut registry = Self {
            schemas: HashMap::new(),
            schema_sources: HashMap::new(),
        };

        registry.load_schemas(path)?;
        Ok(registry)
    }

    /// 載入指定目錄的所有 JSON Schema 文件
    fn load_schemas<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let schema_str = fs::read_to_string(&path)?;
                let schema: Value = serde_json::from_str(&schema_str)
                    .map_err(|e| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid JSON in {}: {}", name, e)
                    ))?;

                match JSONSchema::compile(&schema) {
                    Ok(compiled) => {
                        println!("✅ Loaded schema: {}", name);
                        self.schema_sources.insert(name.clone(), schema);
                        self.schemas.insert(name, compiled);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to compile schema {}: {}", name, e);
                    }
                }
            }
        }

        println!("📊 Total schemas loaded: {}", self.schemas.len());
        Ok(())
    }

    /// 驗證數據是否符合指定 schema
    pub fn validate(&self, schema_name: &str, data: &Value) -> ValidationResult {
        let schema = self.schemas.get(schema_name)
            .ok_or_else(|| vec![format!("Schema '{}' not found", schema_name)])?;

        if schema.is_valid(data) {
            Ok(())
        } else {
            let errors = schema
                .iter_errors(data)
                .map(|e| format!("{} at {}", e, e.instance_path()))
                .collect();
            Err(errors)
        }
    }

    /// 快速檢查數據是否有效 (不返回詳細錯誤)
    pub fn is_valid(&self, schema_name: &str, data: &Value) -> bool {
        self.schemas
            .get(schema_name)
            .map(|schema| schema.is_valid(data))
            .unwrap_or(false)
    }

    /// 獲取所有已載入的 schema 名稱
    pub fn list_schemas(&self) -> Vec<&String> {
        self.schemas.keys().collect()
    }

    /// 獲取 schema 的源 JSON (用於文檔生成)
    pub fn get_schema_source(&self, name: &str) -> Option<&Value> {
        self.schema_sources.get(name)
    }
}

/// 支持多版本的 Schema Registry
pub struct VersionedRegistry {
    versions: HashMap<String, SchemaRegistry>,
    default_version: String,
}

impl VersionedRegistry {
    /// 創建多版本 registry
    pub fn new(base_path: &str, versions: &[&str]) -> Result<Self, std::io::Error> {
        let mut registries = HashMap::new();

        for version in versions {
            let path = format!("{}/{}", base_path, version);
            match SchemaRegistry::from_directory(&path) {
                Ok(registry) => {
                    println!("📦 Loaded version: {}", version);
                    registries.insert(version.to_string(), registry);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to load version {}: {}", version, e);
                }
            }
        }

        let default_version = versions.last()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "At least one version required"
            ))?
            .to_string();

        Ok(Self {
            versions: registries,
            default_version,
        })
    }

    /// 使用指定版本驗證
    pub fn validate(&self, version: Option<&str>, schema_name: &str, data: &Value) -> ValidationResult {
        let version = version.unwrap_or(&self.default_version);

        let registry = self.versions.get(version)
            .ok_or_else(|| vec![format!("Version '{}' not found", version)])?;

        registry.validate(schema_name, data)
    }

    /// 列出所有版本
    pub fn list_versions(&self) -> Vec<&String> {
        self.versions.keys().collect()
    }
}

/// 支持熱重載的 Schema Registry
pub struct HotReloadRegistry {
    registry: Arc<RwLock<SchemaRegistry>>,
    path: String,
}

impl HotReloadRegistry {
    /// 創建支持熱重載的 registry
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let registry = Arc::new(RwLock::new(
            SchemaRegistry::from_directory(path)?
        ));

        Ok(Self {
            registry,
            path: path.to_string(),
        })
    }

    /// 手動重載 schemas
    pub fn reload(&self) -> Result<(), std::io::Error> {
        let new_registry = SchemaRegistry::from_directory(&self.path)?;
        *self.registry.write().unwrap() = new_registry;
        println!("🔄 Schemas reloaded from {}", self.path);
        Ok(())
    }

    /// 驗證數據
    pub fn validate(&self, schema_name: &str, data: &Value) -> ValidationResult {
        self.registry.read().unwrap().validate(schema_name, data)
    }

    /// 啟動自動監聽文件變化 (需要 notify crate)
    #[cfg(feature = "watch")]
    pub fn start_watching(&self) -> notify::Result<()> {
        use notify::{Watcher, RecursiveMode, watcher};
        use std::sync::mpsc::channel;
        use std::time::Duration;

        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1))?;
        watcher.watch(&self.path, RecursiveMode::Recursive)?;

        let registry = Arc::clone(&self.registry);
        let path = self.path.clone();

        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(_event) => {
                        if let Ok(new_registry) = SchemaRegistry::from_directory(&path) {
                            *registry.write().unwrap() = new_registry;
                            println!("🔄 Auto-reloaded schemas");
                        }
                    }
                    Err(e) => eprintln!("Watch error: {}", e),
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_registry() {
        let registry = SchemaRegistry::from_directory("./schemas/v1")
            .expect("Failed to load schemas");

        // Valid data
        let valid_user = json!({
            "email": "alice@example.com",
            "name": "Alice"
        });
        assert!(registry.validate("user", &valid_user).is_ok());

        // Invalid data
        let invalid_user = json!({
            "name": "Bob"  // Missing email
        });
        assert!(registry.validate("user", &invalid_user).is_err());
    }

    #[test]
    fn test_versioned_registry() {
        let registry = VersionedRegistry::new("./schemas", &["v1", "v2"])
            .expect("Failed to load versions");

        let data = json!({"email": "test@example.com", "name": "Test"});

        // Test v1
        assert!(registry.validate(Some("v1"), "user", &data).is_ok());

        // Test v2
        assert!(registry.validate(Some("v2"), "user", &data).is_ok());

        // Test default version (v2)
        assert!(registry.validate(None, "user", &data).is_ok());
    }
}
