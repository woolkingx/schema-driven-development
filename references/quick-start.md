# Schema-Driven Development - Quick Start

## 🚀 5分鐘快速開始

### Step 1: 創建第一個 Schema (2分鐘)

```bash
mkdir -p schemas/v1 examples
```

**schemas/v1/user.schema.json**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "User",
  "type": "object",
  "properties": {
    "email": {
      "type": "string",
      "format": "email"
    },
    "name": {
      "type": "string",
      "minLength": 1
    }
  },
  "required": ["email", "name"],
  "additionalProperties": false
}
```

**examples/user.valid.json**:
```json
{
  "email": "alice@example.com",
  "name": "Alice"
}
```

### Step 2: Rust 專案設置 (1分鐘)

```bash
cargo new my-api
cd my-api
```

**Cargo.toml**:
```toml
[dependencies]
serde_json = "1.0"
jsonschema = "0.26"
lazy_static = "1.4"
```

### Step 3: 實現驗證 (2分鐘)

**src/main.rs**:
```rust
use jsonschema::JSONSchema;
use serde_json::{json, Value};
use lazy_static::lazy_static;

lazy_static! {
    static ref USER_SCHEMA: JSONSchema = {
        let schema = json!({
            "type": "object",
            "properties": {
                "email": {"type": "string", "format": "email"},
                "name": {"type": "string", "minLength": 1}
            },
            "required": ["email", "name"],
            "additionalProperties": false
        });
        JSONSchema::compile(&schema).unwrap()
    };
}

fn main() {
    // ✅ Valid user
    let valid_user = json!({
        "email": "alice@example.com",
        "name": "Alice"
    });
    assert!(USER_SCHEMA.is_valid(&valid_user));
    println!("✅ Valid user accepted!");

    // ❌ Invalid user (no email)
    let invalid_user = json!({
        "name": "Bob"
    });

    if let Err(_) = USER_SCHEMA.validate(&invalid_user) {
        println!("❌ Invalid user rejected!");
        for error in USER_SCHEMA.iter_errors(&invalid_user) {
            println!("  - {}", error);
        }
    }
}
```

**運行**:
```bash
cargo run
```

**輸出**:
```
✅ Valid user accepted!
❌ Invalid user rejected!
  - "email" is a required property
```

---

## 🎯 下一步

### 1. Web API 整合

安裝 Axum:
```bash
cargo add axum tokio serde --features tokio/full,serde/derive
```

**src/main.rs**:
```rust
use axum::{
    routing::post,
    extract::Json,
    http::StatusCode,
    Router,
};
use serde_json::Value;
use lazy_static::lazy_static;
use jsonschema::JSONSchema;

lazy_static! {
    static ref USER_SCHEMA: JSONSchema = {
        let schema_str = include_str!("../schemas/v1/user.schema.json");
        let schema: Value = serde_json::from_str(schema_str).unwrap();
        JSONSchema::compile(&schema).unwrap()
    };
}

async fn create_user(
    Json(payload): Json<Value>
) -> Result<Json<Value>, (StatusCode, String)> {
    // 驗證請求
    USER_SCHEMA.validate(&payload)
        .map_err(|e| (
            StatusCode::BAD_REQUEST,
            format!("Validation failed: {}",
                USER_SCHEMA.iter_errors(&payload)
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ))?;

    // 業務邏輯...
    Ok(Json(payload))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/users", post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

**測試**:
```bash
# ✅ Valid request
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"email": "alice@example.com", "name": "Alice"}'

# ❌ Invalid request
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Bob"}'
```

### 2. 添加 CI/CD

**.github/workflows/schema-validation.yml**:
```yaml
name: Schema Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Validate Schema Syntax
        run: |
          npm install -g ajv-cli
          ajv compile -s schemas/v1/*.json

      - name: Validate Examples
        run: |
          ajv validate -s schemas/v1/user.schema.json -d examples/user.valid.json

      - name: Run Rust Tests
        run: cargo test
```

### 3. 生成文檔

```bash
# 安裝工具
npm install -g @adobe/jsonschema2md

# 生成 Markdown 文檔
jsonschema2md -d schemas/v1 -o docs/

# 查看生成的文檔
cat docs/user.md
```

---

## 📚 完整範例

查看 `references/` 目錄下的完整範例:
- `schema-registry.rs` - Schema Registry 實現
- `api-handler.rs` - API Handler 範例
- `contract-tests.rs` - 契約測試範例
- `ci-cd.yml` - CI/CD 配置範例

---

## 🎓 學習路徑

1. ✅ **快速開始** (你在這裡!)
2. 📖 閱讀主 SKILL.md 了解完整 Workflow
3. 🔨 實現 Schema Registry Pattern
4. 🧪 添加 Property-Based Testing
5. 🚀 設置 CI/CD Pipeline
6. 📊 整合 OpenAPI 文檔生成

---

## 💡 常見問題

**Q: Schema 放哪裡?**
A: 建議放在 `schemas/vX/` 目錄,使用版本號管理。

**Q: 需要生成代碼嗎?**
A: 不需要!運行時驗證即可,這是 Schema-DD 的優勢。

**Q: 前端怎麼用?**
A: 使用 TypeScript 可以生成類型,使用 Ajv 做運行時驗證。

**Q: 性能如何?**
A: Rust jsonschema 非常快 (比 Python 快 10-100x),編譯一次可重複使用。

**Q: 如何處理版本演進?**
A: 使用 `schemas/v1/`, `schemas/v2/` 分別存放,API 支持多版本。
