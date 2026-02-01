# Rust: Runtime JSON Schema Validation

**核心理念**: 運行時驗證,不生成代碼

> "為什麼共用同一個 schema" - 單一事實來源
> 修改 Schema = 修改所有測試 + 文檔 + 契約

## Why Runtime (Not Code-Gen)

**優勢**:
- ✅ 無需代碼生成步驟
- ✅ Schema 變更立即生效
- ✅ jsonschema crate 比 Python 快 10-100 倍
- ✅ 與前端共用相同 schema
- ✅ 靈活的動態屬性存取

**劣勢**:
- ❌ 編譯期無法檢查 schema 錯誤
- ❌ 需要運行時載入 schema

## Dependencies

```toml
[dependencies]
jsonschema = "0.26"
serde_json = "1.0"
lazy_static = "1.4"  # Optional: for global schema registry
```

## Basic Usage

### 1. Inline Schema

```rust
use jsonschema::JSONSchema;
use serde_json::{json, Value};

fn validate_user(data: &Value) -> Result<(), Vec<String>> {
    let schema = json!({
        "type": "object",
        "properties": {
            "email": {"type": "string", "format": "email"},
            "name": {"type": "string", "minLength": 1}
        },
        "required": ["email", "name"]
    });

    let compiled = JSONSchema::compile(&schema).unwrap();

    if compiled.is_valid(data) {
        Ok(())
    } else {
        let errors = compiled.iter_errors(data)
            .map(|e| e.to_string())
            .collect();
        Err(errors)
    }
}
```

### 2. External Schema File

```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref USER_SCHEMA: JSONSchema = {
        let schema = include_str!("../schemas/user.schema.json");
        let schema_value: Value = serde_json::from_str(schema).unwrap();
        JSONSchema::compile(&schema_value).unwrap()
    };
}

fn validate_user(data: &Value) -> Result<(), Vec<String>> {
    if USER_SCHEMA.is_valid(data) {
        Ok(())
    } else {
        Err(USER_SCHEMA.iter_errors(data)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect())
    }
}
```

## Advanced Patterns

### Schema Registry Pattern

適用場景:
- 多種 schema 需要管理
- 需要動態載入 schema
- 需要 schema 版本管理

**完整實現**: `../../examples/schema-registry.rs` (260 lines)

**生產範例**: `../../case-studies/claude-tui-rs/implementation.md`

### Hot Reload Registry

適用場景:
- 開發環境需要快速迭代
- Schema 頻繁變更

**完整實現**: See SKILL.md Phase 6 - Advanced Patterns

## Performance Tips

1. **使用 lazy_static 編譯一次**:
   ```rust
   // ✅ Good: 編譯一次,多次使用
   lazy_static! {
       static ref SCHEMA: JSONSchema = compile_schema();
   }

   // ❌ Bad: 每次調用都重新編譯
   fn validate(data: &Value) {
       let schema = JSONSchema::compile(...).unwrap();
   }
   ```

2. **批量驗證時重用 validator**:
   ```rust
   let validator = JSONSchema::compile(&schema).unwrap();
   for item in items {
       validator.validate(&item)?;
   }
   ```

3. **錯誤處理只在失敗時收集**:
   ```rust
   // ✅ Good: 先檢查是否有效
   if validator.is_valid(data) {
       Ok(())
   } else {
       Err(collect_errors(data))  // 只在失敗時收集
   }
   ```

## Integration with Web Frameworks

### Axum Example

```rust
use axum::{Json, extract::rejection::JsonRejection};

async fn create_user(
    Json(payload): Json<Value>
) -> Result<Json<User>, AppError> {
    // Validate with schema
    USER_SCHEMA.validate(&payload)
        .map_err(|e| AppError::ValidationError(format!("{:?}", e.collect::<Vec<_>>())))?;

    // Deserialize to strongly typed struct
    let user: User = serde_json::from_value(payload)?;

    // Business logic...
    Ok(Json(user))
}
```

**完整 Axum 整合**: `../../examples/user-api/backend/src/main.rs`

## Testing Patterns

### Contract Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validates_valid_user() {
        let valid = json!({
            "email": "user@example.com",
            "name": "John Doe"
        });

        assert!(USER_SCHEMA.is_valid(&valid));
    }

    #[test]
    fn test_schema_rejects_invalid_email() {
        let invalid = json!({
            "email": "not-an-email",
            "name": "John"
        });

        assert!(!USER_SCHEMA.is_valid(&invalid));
    }
}
```

**完整測試範例**: `../../case-studies/claude-tui-rs/` (9 passing tests)

## Cross-Language Consistency

同一個 `user.schema.json` 可用於:
- **Rust**: jsonschema crate (this file)
- **TypeScript**: ajv (`../typescript/ajv-validation.md`)
- **Python**: jsonschema (`../python/jsonschema-validation.md`)

**保證**: 所有語言驗證邏輯完全一致

## Resources

- **jsonschema crate**: https://docs.rs/jsonschema
- **JSON Schema Spec**: https://json-schema.org/
- **Performance Benchmarks**: jsonschema is 10-100x faster than Python

## Next Steps

- Read: `schema-registry.md` - 學習 Registry 模式
- Read: `axum-integration.md` - Web 框架整合
- Try: `../../examples/schema-registry.rs` - 運行範例
- Study: `../../case-studies/claude-tui-rs/` - 真實項目案例
