---
name: schema-driven-development
description: Schema-Driven Development (SDD) - Complete workflow for API development using JSON Schema as executable contracts. Covers schema design, runtime validation with Rust jsonschema, contract testing, CI/CD automation, and team collaboration patterns.
version: 1.0.0
tags: [schema-driven, api, json-schema, rust, contract-testing, tdd, ci-cd, workflow]
---

# Schema-Driven Development (SDD)

**TL;DR**: Schema-Driven Development is TDD 2.0 - using JSON Schema as executable specifications that serve as tests, documentation, and contracts simultaneously.

> 修改 Schema = 修改所有測試 + 文檔 + 契約

## 🎯 When to Use This Skill

Use this skill when:
- Building REST APIs with clear contracts
- Coordinating frontend and backend teams
- Need runtime validation without code generation
- Want single source of truth for API specifications
- Implementing microservices with contract testing
- Need automatic API documentation
- Want schema-first development workflow

**Trigger phrases:**
- "Let's use Schema-Driven Development"
- "Start with JSON Schema"
- "Create API contract"
- "Schema-first approach"
- "Contract-driven API"

## 🌟 Core Concept: Schema = Contract = Test = Doc = Object

```
Traditional TDD:
  Write Test → Write Code → Test Passes
  (Need 10+ tests for validation)

Schema-DD:
  Write Schema → Validate with Schema → Implement
  (One schema = dozens of tests!)

JSON-as-Object:
  JSON Schema → Runtime Validation → Dynamic Object
       ↓               ↓                    ↓
  結構定義        確保正確          直接操作屬性/方法
```

**Why Schema-DD is Better:**
- ✅ Declarative > Imperative
- ✅ Frontend & Backend share same contract
- ✅ Auto-generates tests/docs/types
- ✅ Runtime validation (no code-gen needed)
- ✅ Version control friendly (JSON)
- ✅ **JSON-as-Object**: 天然的欄位對齊、概念對齊

**JSON-as-Object Pattern** (核心優勢):
- ✅ JSON 本身就是 object,無需轉換
- ✅ Schema 定義結構,直接操作屬性
- ✅ 前後端使用相同路徑 (`event.message.model`)
- ✅ 添加欄位只需改 schema,無需改代碼
- ✅ 一行驗證 (`validate()`) 取代 100 行手寫驗證

**詳細說明**: `methodology/json-as-object.md`

## 📋 Complete SDD Workflow

### Phase 1: DEFINE (定義階段)

**Step 1: API Design Session**
```bash
# Team collaboration to define API endpoints
# Output: Initial schema draft
```

**Step 2: Create JSON Schema**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://api.example.com/schemas/user.json",
  "title": "User",
  "description": "User resource for authentication system",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "uuid",
      "readOnly": true
    },
    "email": {
      "type": "string",
      "format": "email"
    },
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 100
    },
    "age": {
      "type": "integer",
      "minimum": 0,
      "maximum": 150
    },
    "roles": {
      "type": "array",
      "items": {
        "enum": ["admin", "user", "guest"]
      },
      "minItems": 1,
      "uniqueItems": true
    },
    "created_at": {
      "type": "string",
      "format": "date-time",
      "readOnly": true
    }
  },
  "required": ["email", "name"],
  "additionalProperties": false
}
```

**Step 3: Store in Central Repo**
```bash
# Project structure
project/
├── schemas/
│   ├── v1/
│   │   ├── user.schema.json
│   │   ├── post.schema.json
│   │   └── comment.schema.json
│   ├── v2/
│   │   └── user.schema.json  # New version
│   └── current -> v2/        # Symlink
├── examples/
│   ├── user.valid.json       # Valid examples
│   └── user.invalid.json     # Invalid examples
└── tests/
    └── schema.test.json      # Schema meta-tests
```

### Phase 2: VALIDATE (驗證階段)

**CI/CD Pipeline (GitHub Actions)**
```yaml
# .github/workflows/schema-validation.yml
name: Schema Validation

on:
  pull_request:
    paths:
      - 'schemas/**'

jobs:
  validate-schema:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # 1. Validate schema syntax
      - name: Validate JSON Schema Syntax
        run: |
          npm install -g ajv-cli
          ajv compile -s schemas/v2/*.json

      # 2. Check backward compatibility
      - name: Check Breaking Changes
        run: |
          npm install -g @apidevtools/json-schema-diff
          json-schema-diff schemas/v1/user.schema.json schemas/v2/user.schema.json

      # 3. Lint schema
      - name: Lint Schema
        run: |
          npm install -g @stoplight/spectral-cli
          spectral lint schemas/**/*.json --ruleset .spectral.yml

      # 4. Validate examples
      - name: Validate Examples
        run: |
          ajv validate -s schemas/v2/user.schema.json -d examples/user.valid.json

      # 5. Generate documentation
      - name: Generate Docs
        run: |
          npx @adobe/jsonschema2md -d schemas/v2/ -o docs/

      - name: Upload Docs
        uses: actions/upload-artifact@v3
        with:
          name: api-docs
          path: docs/
```

### Phase 3: IMPLEMENT - Rust Backend

**Cargo.toml**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
jsonschema = "0.26"
lazy_static = "1.4"
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

**Schema Registry Pattern**
```rust
// src/schema_registry.rs
use jsonschema::JSONSchema;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct SchemaRegistry {
    schemas: HashMap<String, JSONSchema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            schemas: HashMap::new(),
        };

        // Load all schemas from directory
        registry.load_schemas("./schemas/current");
        registry
    }

    fn load_schemas(&mut self, path: &str) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let name = path.file_stem().unwrap().to_str().unwrap();
                let schema_str = std::fs::read_to_string(&path).unwrap();
                let schema: Value = serde_json::from_str(&schema_str).unwrap();

                match JSONSchema::compile(&schema) {
                    Ok(compiled) => {
                        self.schemas.insert(name.to_string(), compiled);
                    }
                    Err(e) => eprintln!("Failed to compile schema {}: {}", name, e),
                }
            }
        }
    }

    pub fn validate(&self, schema_name: &str, data: &Value) -> Result<(), Vec<String>> {
        let schema = self.schemas.get(schema_name)
            .ok_or_else(|| vec![format!("Schema '{}' not found", schema_name)])?;

        if schema.is_valid(data) {
            Ok(())
        } else {
            let errors = schema.iter_errors(data)
                .map(|e| format!("{} at {}", e, e.instance_path()))
                .collect();
            Err(errors)
        }
    }

    pub fn validate_and_deserialize<T>(&self, schema_name: &str, data: &Value) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        // 1. Schema validation
        self.validate(schema_name, data)
            .map_err(|errors| errors.join(", "))?;

        // 2. Deserialize to Rust type
        serde_json::from_value(data.clone())
            .map_err(|e| format!("Deserialization failed: {}", e))
    }
}

// Global singleton
pub static REGISTRY: LazyLock<SchemaRegistry> = LazyLock::new(|| {
    SchemaRegistry::new()
});
```

**API Handler with Schema Validation**
```rust
// src/handlers/user.rs
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use crate::schema_registry::REGISTRY;

// Error response
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.message
            }))
        ).into_response()
    }
}

// Create user endpoint
pub async fn create_user(
    Json(payload): Json<Value>
) -> Result<Json<Value>, ApiError> {
    // Step 1: Validate against schema
    REGISTRY.validate("user", &payload)
        .map_err(|errors| ApiError {
            status: StatusCode::BAD_REQUEST,
            message: format!("Validation failed: {}", errors.join(", ")),
        })?;

    // Step 2: Business logic (payload is guaranteed valid!)
    let user_id = uuid::Uuid::new_v4();
    let mut user = payload;
    user["id"] = serde_json::json!(user_id);
    user["created_at"] = serde_json::json!(chrono::Utc::now());

    // Step 3: Validate response (ensure we return valid data)
    REGISTRY.validate("user", &user)
        .map_err(|_| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Response validation failed".to_string(),
        })?;

    Ok(Json(user))
}

// Update user endpoint
pub async fn update_user(
    Path(user_id): Path<String>,
    Json(payload): Json<Value>
) -> Result<Json<Value>, ApiError> {
    // Partial update schema (different from create)
    REGISTRY.validate("user-update", &payload)
        .map_err(|errors| ApiError {
            status: StatusCode::BAD_REQUEST,
            message: format!("Validation failed: {}", errors.join(", ")),
        })?;

    // Business logic...
    let updated_user = perform_update(user_id, payload).await?;

    // Validate full response
    REGISTRY.validate("user", &updated_user)?;

    Ok(Json(updated_user))
}
```

**Main Application**
```rust
// src/main.rs
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::CorsLayer;

mod schema_registry;
mod handlers;

#[tokio::main]
async fn main() {
    // Ensure schemas are loaded at startup
    println!("Loaded {} schemas", schema_registry::REGISTRY.schemas.len());

    let app = Router::new()
        .route("/api/users", post(handlers::user::create_user))
        .route("/api/users/:id", get(handlers::user::get_user))
        .route("/api/users/:id", put(handlers::user::update_user))
        .route("/api/users/:id", delete(handlers::user::delete_user))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### Phase 4: CONTRACT TESTING

**Property-Based Testing with Schema**
```rust
// tests/contract_tests.rs
use proptest::prelude::*;
use serde_json::json;
use crate::schema_registry::REGISTRY;

proptest! {
    #[test]
    fn schema_rejects_invalid_emails(email in "[a-z]{5,10}") {
        // Schema auto-tests: invalid email format
        let data = json!({
            "name": "Alice",
            "email": email  // No @ sign
        });

        assert!(REGISTRY.validate("user", &data).is_err());
    }

    #[test]
    fn schema_rejects_out_of_range_ages(age in 151..1000i32) {
        // Schema auto-tests: age out of range
        let data = json!({
            "name": "Alice",
            "email": "alice@example.com",
            "age": age
        });

        assert!(REGISTRY.validate("user", &data).is_err());
    }

    #[test]
    fn schema_accepts_valid_data(
        name in "[A-Z][a-z]{2,20}",
        age in 0..150i32
    ) {
        // Schema auto-tests: valid data
        let data = json!({
            "name": name,
            "email": format!("{}@example.com", name.to_lowercase()),
            "age": age,
            "roles": ["user"]
        });

        assert!(REGISTRY.validate("user", &data).is_ok());
    }
}
```

**Integration Tests**
```rust
#[tokio::test]
async fn test_create_user_api() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/users")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                    "name": "Alice",
                    "email": "alice@example.com",
                    "age": 30,
                    "roles": ["user"]
                }"#))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: Value = serde_json::from_slice(&body).unwrap();

    // Response automatically validated by handler
    assert!(user["id"].is_string());
    assert_eq!(user["email"], "alice@example.com");
}

#[tokio::test]
async fn test_invalid_request_rejected() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/users")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                    "name": "",
                    "email": "not-an-email"
                }"#))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

### Phase 5: FRONTEND INTEGRATION

**TypeScript Type Generation**
```bash
# Generate TypeScript types from JSON Schema
npm install -g json-schema-to-typescript
json-schema-to-typescript schemas/v2/user.schema.json > src/types/User.ts
```

**Generated Types**
```typescript
// src/types/User.ts (auto-generated)
export interface User {
  id?: string;
  email: string;
  name: string;
  age?: number;
  roles: ("admin" | "user" | "guest")[];
  created_at?: string;
}
```

**Runtime Validation (Client-Side)**
```typescript
// src/api/users.ts
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import userSchema from '../schemas/user.schema.json';
import type { User } from '../types/User';

const ajv = new Ajv();
addFormats(ajv);
const validateUser = ajv.compile<User>(userSchema);

export async function createUser(userData: Partial<User>): Promise<User> {
  // Client-side validation before sending
  if (!validateUser(userData)) {
    throw new Error(`Validation failed: ${JSON.stringify(validateUser.errors)}`);
  }

  const response = await fetch('/api/users', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(userData),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message);
  }

  const user = await response.json();

  // Validate response
  if (!validateUser(user)) {
    throw new Error('Server returned invalid data');
  }

  return user;
}
```

## 🎨 Advanced Patterns

### Pattern 1: Schema Composition

**Base Schema**
```json
{
  "$id": "https://api.example.com/schemas/base-resource.json",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "uuid",
      "readOnly": true
    },
    "created_at": {
      "type": "string",
      "format": "date-time",
      "readOnly": true
    },
    "updated_at": {
      "type": "string",
      "format": "date-time",
      "readOnly": true
    }
  },
  "required": ["id", "created_at"]
}
```

**Extended Schema**
```json
{
  "$id": "https://api.example.com/schemas/user.json",
  "allOf": [
    { "$ref": "base-resource.json" },
    {
      "type": "object",
      "properties": {
        "email": { "type": "string", "format": "email" },
        "name": { "type": "string" }
      },
      "required": ["email", "name"]
    }
  ]
}
```

### Pattern 2: Schema Versioning

```rust
pub struct VersionedRegistry {
    registries: HashMap<String, SchemaRegistry>,
}

impl VersionedRegistry {
    pub fn new() -> Self {
        let mut registries = HashMap::new();
        registries.insert("v1".to_string(), SchemaRegistry::from_path("./schemas/v1"));
        registries.insert("v2".to_string(), SchemaRegistry::from_path("./schemas/v2"));
        Self { registries }
    }

    pub fn validate(&self, version: &str, schema: &str, data: &Value) -> Result<(), Vec<String>> {
        self.registries
            .get(version)
            .ok_or_else(|| vec![format!("Version {} not found", version)])?
            .validate(schema, data)
    }
}

// API Handler
pub async fn create_user_versioned(
    Path(version): Path<String>,  // v1 or v2
    Json(payload): Json<Value>
) -> Result<Json<Value>, ApiError> {
    VERSIONED_REGISTRY.validate(&version, "user", &payload)?;
    // ...
}
```

### Pattern 3: Schema Hot Reload

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub struct HotReloadRegistry {
    registry: Arc<RwLock<SchemaRegistry>>,
}

impl HotReloadRegistry {
    pub fn new(path: &str) -> Self {
        let registry = Arc::new(RwLock::new(SchemaRegistry::from_path(path)));

        // Watch for file changes
        let registry_clone = Arc::clone(&registry);
        let path_clone = path.to_string();

        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
            watcher.watch(&path_clone, RecursiveMode::Recursive).unwrap();

            loop {
                match rx.recv() {
                    Ok(_) => {
                        println!("Schema changed, reloading...");
                        let new_registry = SchemaRegistry::from_path(&path_clone);
                        *registry_clone.write().unwrap() = new_registry;
                    }
                    Err(e) => eprintln!("Watch error: {}", e),
                }
            }
        });

        Self { registry }
    }

    pub fn validate(&self, schema_name: &str, data: &Value) -> Result<(), Vec<String>> {
        self.registry.read().unwrap().validate(schema_name, data)
    }
}
```

## 🛠️ Tool Integration

### OpenAPI Integration

```bash
# Convert JSON Schemas to OpenAPI
npm install -g @apidevtools/swagger-cli

# Generate OpenAPI spec
cat > openapi.yaml <<EOF
openapi: 3.0.0
info:
  title: My API
  version: 2.0.0
paths:
  /users:
    post:
      requestBody:
        content:
          application/json:
            schema:
              \$ref: './schemas/v2/user.schema.json'
      responses:
        '201':
          content:
            application/json:
              schema:
                \$ref: './schemas/v2/user.schema.json'
EOF

# Bundle into single file
swagger-cli bundle openapi.yaml --outfile dist/openapi.json
```

### Documentation Generation

```bash
# Generate beautiful docs from OpenAPI
npx @redocly/cli build-docs dist/openapi.json --output docs/index.html

# Or use RapiDoc
npm install rapidoc
```

### Mock Server

```bash
# Generate mock server from schemas
npm install -g @stoplight/prism-cli

# Run mock server
prism mock dist/openapi.json
# Mock server running at http://localhost:4010
```

## 📊 Best Practices

### 1. Schema Organization

```
schemas/
├── common/               # Shared schemas
│   ├── error.json
│   ├── pagination.json
│   └── base-resource.json
├── v1/                   # Version 1
│   ├── user.json
│   └── post.json
├── v2/                   # Version 2 (with changes)
│   ├── user.json        # Modified
│   └── post.json
│   └── comment.json     # New resource
└── current -> v2/       # Symlink to latest
```

### 2. Schema Naming Conventions

```json
{
  "$id": "https://api.example.com/schemas/user.json",
  "title": "User",
  "description": "User resource for authentication system",

  "definitions": {
    "CreateUserRequest": {},
    "UpdateUserRequest": {},
    "UserResponse": {}
  }
}
```

### 3. Error Handling

```rust
pub enum SchemaError {
    ValidationFailed(Vec<String>),
    SchemaNotFound(String),
    InvalidSchema(String),
}

impl IntoResponse for SchemaError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SchemaError::ValidationFailed(errors) => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "Validation failed",
                    "details": errors
                })
            ),
            SchemaError::SchemaNotFound(name) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": format!("Schema '{}' not found", name)
                })
            ),
            SchemaError::InvalidSchema(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": format!("Invalid schema: {}", msg)
                })
            ),
        };

        (status, Json(message)).into_response()
    }
}
```

### 4. Performance Optimization

```rust
// Compile schemas at startup
lazy_static! {
    static ref USER_SCHEMA: JSONSchema = {
        let schema = include_str!("../schemas/current/user.json");
        let schema_value: Value = serde_json::from_str(schema).unwrap();
        JSONSchema::compile(&schema_value).unwrap()
    };
}

// Use compiled schema (fast)
fn validate_user(data: &Value) -> Result<(), ValidationError> {
    if USER_SCHEMA.is_valid(data) {
        Ok(())
    } else {
        Err(ValidationError::from(USER_SCHEMA.iter_errors(data)))
    }
}
```

## 📖 Resources

### JSON Schema
- [JSON Schema Official Site](https://json-schema.org/)
- [Understanding JSON Schema](https://json-schema.org/understanding-json-schema/)
- [JSON Schema Store](https://www.schemastore.org/)

### Rust Ecosystem
- [jsonschema crate](https://docs.rs/jsonschema)
- [serde_json](https://docs.rs/serde_json)
- [Axum Web Framework](https://docs.rs/axum)

### Tools
- [Specmatic](https://specmatic.io/) - Contract Testing Platform
- [Pactflow](https://pactflow.io/) - Contract Testing Service
- [Spectral](https://stoplight.io/open-source/spectral) - JSON/YAML Linter

### Examples
- [SDD Toolkit](https://github.com/tylerburleigh/claude-sdd-toolkit)
- [API Examples](https://github.com/OAI/OpenAPI-Specification/tree/main/examples)

## 🎓 Summary

Schema-Driven Development transforms API development by:

1. **Single Source of Truth**: One schema → types, tests, docs
2. **TDD 2.0**: Schema = executable specification
3. **Runtime Validation**: No code generation needed (Rust jsonschema)
4. **Team Collaboration**: Frontend/Backend share same contract
5. **CI/CD Automation**: Automatic validation, testing, docs
6. **Version Management**: Git-friendly JSON schemas
7. **Performance**: Rust implementation = 10-100x faster validation

**Start with**:
1. Define JSON Schema
2. Set up Schema Registry
3. Implement runtime validation
4. Add contract tests
5. Automate CI/CD

**Remember**: 修改 Schema = 修改一切!
