# JSON-as-Object Pattern

**核心洞察**: JSON = Object,直接轉換為物件組織來操作屬性及方法

## 🎯 問題: 為什麼需要這個模式?

### 傳統方式的問題

**硬編碼 Struct (Rust/Go/Java)**:
```rust
// ❌ 問題: 每次 API 變更都要改代碼
struct User {
    email: String,
    name: String,
    age: Option<u32>,
}

// 添加新欄位? → 修改 struct → 重新編譯 → 更新所有使用處
```

**手動 Map 操作 (所有語言)**:
```rust
// ❌ 問題: 失去類型安全,容易出錯
let email = data.get("email").unwrap().as_str().unwrap();
let nested = data.get("message").unwrap()
                .get("content").unwrap()
                .get("text").unwrap();
// → 太多 unwrap,隨時 panic
// → 沒有 schema 驗證,錯誤數據直接進入系統
```

**分散的驗證邏輯**:
```rust
// ❌ 問題: 驗證邏輯散落各處
if !email.contains('@') {
    return Err("invalid email");
}
if name.is_empty() {
    return Err("name required");
}
if age > 150 {
    return Err("invalid age");
}
// → 重複的驗證代碼
// → 與文檔不同步
// → 前後端驗證邏輯不一致
```

## ✅ 解決方案: JSON-as-Object Pattern

### 核心理念

```
JSON Schema (結構定義)
    ↓
Runtime Validation (確保正確)
    ↓
Dynamic Object (直接操作)
    ↓
Type-Safe Access (安全存取)
```

**天然對齊**:
- ✅ JSON 本身就是 object
- ✅ Schema 定義 object 結構
- ✅ 無需額外轉換
- ✅ 直接以 object 方式操作屬性/方法

### 實現模式

#### 1. Schema 定義結構

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MessageStart",
  "type": "object",
  "properties": {
    "type": {"const": "message_start"},
    "message": {
      "type": "object",
      "properties": {
        "id": {"type": "string"},
        "model": {"type": "string"},
        "role": {"type": "string", "enum": ["assistant"]},
        "content": {
          "type": "array",
          "items": {"type": "object"}
        }
      },
      "required": ["id", "model", "role"]
    }
  },
  "required": ["type", "message"]
}
```

#### 2. Runtime Validation + Dynamic Object

```rust
use serde_json::Value;

pub struct DynamicEvent {
    event_type: String,
    data: Value,
    schema: &'static JSONSchema,
}

impl DynamicEvent {
    pub fn new(event_type: String, data: Value) -> Self {
        let schema = SCHEMAS.get(&event_type).expect("schema not found");
        Self { event_type, data, schema }
    }

    // ✅ 驗證: Schema 自動確保數據正確
    pub fn validate(&self) -> Result<(), String> {
        if self.schema.is_valid(&self.data) {
            Ok(())
        } else {
            let errors = self.schema.iter_errors(&self.data)
                .map(|e| e.to_string())
                .collect();
            Err(format!("Validation failed: {:?}", errors))
        }
    }

    // ✅ 動態屬性存取: 像操作 object 一樣自然
    pub fn get_str(&self, path: &str) -> Option<String> {
        get_nested_value(&self.data, path)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_u64(&self, path: &str) -> Option<u64> {
        get_nested_value(&self.data, path)
            .and_then(|v| v.as_u64())
    }

    pub fn get_array(&self, path: &str) -> Option<&Vec<Value>> {
        get_nested_value(&self.data, path)
            .and_then(|v| v.as_array())
    }

    // ✅ 檢查屬性存在: 避免 unwrap panic
    pub fn has(&self, path: &str) -> bool {
        get_nested_value(&self.data, path).is_some()
    }

    // ✅ 數組長度: 安全處理數組
    pub fn len(&self, path: &str) -> Option<usize> {
        self.get_array(path).map(|arr| arr.len())
    }
}

// Helper: 支持嵌套路徑 "message.content[0].text"
fn get_nested_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        if let Some(idx_start) = part.find('[') {
            // 處理數組: "content[0]"
            let field = &part[..idx_start];
            let idx_str = &part[idx_start+1..part.len()-1];
            let idx: usize = idx_str.parse().ok()?;

            current = current.get(field)?.get(idx)?;
        } else {
            // 處理對象: "message"
            current = current.get(part)?;
        }
    }

    Some(current)
}
```

#### 3. 使用: 像操作原生 Object 一樣

```rust
// ✅ 創建 + 驗證
let event = DynamicEvent::new("message_start".to_string(), json_data);
event.validate()?;  // Schema 自動驗證

// ✅ 直接存取屬性: 天然對齊
let model = event.get_str("message.model");           // Some("claude-sonnet-4-5")
let message_id = event.get_str("message.id");         // Some("msg_123")
let role = event.get_str("message.role");             // Some("assistant")

// ✅ 嵌套存取: 支持深層路徑
let first_content = event.get_str("message.content[0].text");

// ✅ 安全檢查: 避免 panic
if event.has("message.usage") {
    let tokens = event.get_u64("message.usage.input_tokens");
}

// ✅ 數組處理: 優雅迭代
if let Some(len) = event.len("message.content") {
    for i in 0..len {
        let path = format!("message.content[{}].text", i);
        if let Some(text) = event.get_str(&path) {
            println!("Content {}: {}", i, text);
        }
    }
}
```

## 🌟 核心優勢

### 1. 天然的欄位對齊

**Schema 定義 → JSON 數據 → Object 操作**:
```
Schema:       "message.model": {"type": "string"}
   ↓
JSON Data:    {"message": {"model": "claude-sonnet-4-5"}}
   ↓
Object API:   event.get_str("message.model")
```

**完美對齊**:
- 欄位名稱一致
- 類型定義一致
- 嵌套結構一致
- 概念模型一致

### 2. 天然的概念對齊

**前端 (TypeScript)**:
```typescript
const event = await fetchEvent();
const model = event.message.model;        // object.property
const text = event.message.content[0].text;  // object.array[i].property
```

**後端 (Rust)**:
```rust
let event = DynamicEvent::new(...);
let model = event.get_str("message.model");     // 概念相同
let text = event.get_str("message.content[0].text");  // 路徑相同
```

**概念統一**:
- 前後端使用相同路徑字符串
- 相同的嵌套邏輯
- 相同的數組索引方式
- 前端改欄位 → 後端自動跟隨 (Schema 驅動)

### 3. Schema 自動驗證

**無需手寫驗證**:
```rust
// ❌ 傳統: 手寫 100+ 行驗證代碼
if !data.contains_key("message") { ... }
if !data["message"].is_object() { ... }
if !data["message"]["model"].is_string() { ... }
// ... 重複 100 次

// ✅ Schema-DD: 一行驗證
event.validate()?;  // Schema 自動處理所有驗證
```

**自動獲得**:
- 類型檢查 (string/number/boolean/array/object)
- 必填欄位檢查 (required)
- 格式驗證 (email/uuid/date-time)
- 範圍驗證 (min/max/minLength/maxLength)
- 枚舉驗證 (enum)
- 嵌套結構驗證 (遞歸驗證)

### 4. 零樣板代碼

**添加新欄位只需修改 Schema**:

```diff
// schemas/message_start.schema.json
{
  "properties": {
    "message": {
      "properties": {
        "model": {"type": "string"},
+       "temperature": {"type": "number", "minimum": 0, "maximum": 1}
      }
    }
  }
}
```

**自動獲得**:
- ✅ 驗證邏輯自動更新
- ✅ `event.get_f64("message.temperature")` 立即可用
- ✅ 前端類型自動生成 (TypeScript)
- ✅ 文檔自動更新
- ✅ 無需修改 Rust 代碼

## 🔧 實現技巧

### 1. 支持多種數據類型

```rust
impl DynamicEvent {
    pub fn get_str(&self, path: &str) -> Option<String> { /* ... */ }
    pub fn get_u64(&self, path: &str) -> Option<u64> { /* ... */ }
    pub fn get_i64(&self, path: &str) -> Option<i64> { /* ... */ }
    pub fn get_f64(&self, path: &str) -> Option<f64> { /* ... */ }
    pub fn get_bool(&self, path: &str) -> Option<bool> { /* ... */ }
    pub fn get_array(&self, path: &str) -> Option<&Vec<Value>> { /* ... */ }
    pub fn get_object(&self, path: &str) -> Option<&Map<String, Value>> { /* ... */ }
}
```

### 2. 支持嵌套路徑

```rust
// 簡單屬性
event.get_str("type")                    // "message_start"

// 嵌套對象
event.get_str("message.model")           // "claude-sonnet-4-5"

// 數組索引
event.get_str("message.content[0].text") // "Hello"

// 深層嵌套
event.get_u64("message.usage.input_tokens")  // 100
```

### 3. 安全的錯誤處理

```rust
// ✅ 返回 Option: 使用者明確處理
match event.get_str("message.model") {
    Some(model) => println!("Model: {}", model),
    None => println!("Model not found"),
}

// ✅ 使用 ? 操作符鏈式調用
let model = event.get_str("message.model")?;

// ✅ unwrap_or 提供默認值
let temp = event.get_f64("message.temperature").unwrap_or(1.0);
```

### 4. Schema Introspection

```rust
impl SchemaRegistry {
    // 查詢 schema 支持哪些屬性
    pub fn get_properties(&self, event_type: &str) -> Vec<String> {
        // 從 schema 解析 properties
    }

    // 檢查事件類型是否存在
    pub fn has(&self, event_type: &str) -> bool {
        self.schemas.contains_key(event_type)
    }

    // 獲取所有註冊的事件類型
    pub fn types(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }
}
```

## 📊 對比: 三種方式

### 傳統硬編碼 Struct

```rust
struct MessageStart {
    r#type: String,
    message: Message,
}

struct Message {
    id: String,
    model: String,
    role: String,
    content: Vec<Content>,
}
```

**問題**:
- ❌ 添加欄位 → 修改代碼 → 重新編譯
- ❌ 驗證邏輯分散
- ❌ 與前端類型不同步
- ❌ 樣板代碼多

### 手動 Map 操作

```rust
let email = data.get("email").unwrap().as_str().unwrap();
```

**問題**:
- ❌ 失去類型安全
- ❌ unwrap 隨時 panic
- ❌ 無自動驗證
- ❌ 難以重構

### JSON-as-Object (Schema-DD)

```rust
let event = DynamicEvent::new("message_start", data);
event.validate()?;
let model = event.get_str("message.model");
```

**優勢**:
- ✅ Schema 自動驗證
- ✅ 類型安全 (Option<T>)
- ✅ 零樣板代碼
- ✅ 前後端對齊
- ✅ 靈活擴展

## 🎯 使用場景

### 適合

- ✅ **流式事件**: 多種事件類型 (14+)
- ✅ **API Gateway**: 路由不同 schema 的請求
- ✅ **插件系統**: 動態載入 schema
- ✅ **微服務**: Schema Registry 統一管理
- ✅ **快速迭代**: 頻繁添加/修改欄位

### 不適合

- ❌ **核心數據模型**: 使用 Rust struct (強類型保證)
- ❌ **性能關鍵路徑**: serde 反序列化更快
- ❌ **內部 API**: 不需要跨語言一致性

## 🚀 最佳實踐

### 1. 混合使用

```rust
// ✅ 核心模型用 struct
#[derive(Deserialize)]
struct User {
    id: UserId,
    email: Email,
    created_at: DateTime<Utc>,
}

// ✅ 動態事件用 DynamicEvent
let event = DynamicEvent::new("user_created", json!({
    "user_id": user.id,
    "email": user.email,
}));
event.validate()?;
```

### 2. 提前驗證

```rust
// ✅ 在邊界驗證一次
let event = DynamicEvent::new(event_type, data);
event.validate()?;  // 一次驗證,後續安全使用

// ✅ 後續操作無需重複驗證
let model = event.get_str("message.model").unwrap();  // 已驗證,可以 unwrap
```

### 3. 清晰的錯誤信息

```rust
if let Err(e) = event.validate() {
    log::warn!("Invalid event: {}", e);
    // e 包含 schema 驗證的詳細錯誤
    // "message.model: required property missing"
    // "message.temperature: 1.5 is greater than maximum 1.0"
}
```

## 📖 延伸閱讀

- **Rust 實現**: `../references/rust/jsonschema-runtime.md`
- **TypeScript 實現**: `../references/typescript/ajv-validation.md`
- **真實案例**: `../case-studies/claude-tui-rs/` (14 種事件類型)
- **Schema Registry**: `../examples/schema-registry.rs`

## 🎉 總結

**JSON-as-Object Pattern 的核心價值**:

1. **天然對齊**: JSON 本身就是 object,無需轉換
2. **概念統一**: 前後端使用相同路徑,相同概念
3. **零樣板**: 添加欄位只需改 schema,無需改代碼
4. **自動驗證**: Schema 即測試,一行驗證代替 100 行代碼
5. **安全存取**: Option<T> 取代 unwrap,避免 panic

**這就是 Schema-DD 的精髓**: Schema 不只是文檔,而是運行時的**活的類型系統**。
