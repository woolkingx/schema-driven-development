# Component Library Pattern: Schema-Driven 組件化開發

**核心洞察**: 組件 = Schema,Props = JSON Object

## 🎯 問題: 傳統組件庫的痛點

### UI 組件庫的常見問題

**1. Props 文檔過時**:
```typescript
// Button.tsx
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  onClick?: () => void;
}

// README.md (6 個月前寫的)
// Props:
// - variant: 'primary' | 'secondary'  ❌ 缺少 'danger'
// - size: 'small' | 'large'           ❌ 名稱不一致
```

**2. 前後端類型不同步**:
```typescript
// Frontend: Button.tsx
type ButtonVariant = 'primary' | 'secondary' | 'danger';

// Backend: button_config.rs
enum ButtonVariant { Primary, Secondary }  // ❌ 缺少 Danger
```

**3. 驗證邏輯分散**:
```typescript
// 每個組件都要手寫驗證
function Button({ variant, size }: ButtonProps) {
  if (!['primary', 'secondary', 'danger'].includes(variant)) {
    throw new Error('Invalid variant');
  }
  if (size === 'sm' && variant === 'danger') {
    console.warn('Small danger button not recommended');
  }
  // ... 100+ 行驗證邏輯
}
```

**4. Storybook 手動維護**:
```typescript
// Button.stories.tsx
export default {
  argTypes: {
    variant: {
      options: ['primary', 'secondary'],  // ❌ 手動維護,容易過時
      control: { type: 'select' }
    }
  }
}
```

## ✅ 解決方案: Schema-Driven Component Library

### 核心理念

```
Schema Registry (組件規格庫)
     ↓
JSON Schema (組件 Props 定義)
     ↓
Runtime Validation + Type Generation + Storybook
     ↓
前後端完美對齊
```

### 1. Schema 定義組件 Props

**schemas/components/Button.schema.json**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://ui.example.com/components/Button",
  "title": "Button Component Props",
  "description": "Primary action button with multiple variants",
  "type": "object",
  "properties": {
    "variant": {
      "type": "string",
      "enum": ["primary", "secondary", "danger", "ghost"],
      "default": "primary",
      "description": "Visual style variant"
    },
    "size": {
      "type": "string",
      "enum": ["sm", "md", "lg"],
      "default": "md",
      "description": "Button size"
    },
    "disabled": {
      "type": "boolean",
      "default": false,
      "description": "Disable button interactions"
    },
    "loading": {
      "type": "boolean",
      "default": false,
      "description": "Show loading spinner"
    },
    "icon": {
      "type": "string",
      "description": "Icon name from icon library"
    },
    "children": {
      "type": "string",
      "description": "Button label text"
    }
  },
  "required": ["children"],
  "additionalProperties": false,

  "examples": [
    {
      "variant": "primary",
      "size": "md",
      "children": "Click me"
    },
    {
      "variant": "danger",
      "size": "lg",
      "icon": "trash",
      "children": "Delete"
    }
  ]
}
```

### 2. 自動生成 TypeScript 類型

```bash
# 自動生成組件類型
json2ts schemas/components/*.schema.json > src/components/types.ts
```

**生成的 types.ts**:
```typescript
export interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  icon?: string;
  children: string;  // required
}
```

### 3. 前端組件實現

```typescript
import Ajv from 'ajv';
import buttonSchema from '../schemas/components/Button.schema.json';
import type { ButtonProps } from './types';

const ajv = new Ajv();
const validateButton = ajv.compile(buttonSchema);

export function Button(props: ButtonProps) {
  // ✅ 自動驗證 (開發模式)
  if (process.env.NODE_ENV === 'development') {
    if (!validateButton(props)) {
      console.error('Invalid Button props:', validateButton.errors);
    }
  }

  // ✅ 組件實現
  return (
    <button
      className={`btn btn-${props.variant} btn-${props.size}`}
      disabled={props.disabled || props.loading}
    >
      {props.loading && <Spinner />}
      {props.icon && <Icon name={props.icon} />}
      {props.children}
    </button>
  );
}
```

### 4. Storybook 自動生成

```typescript
// Button.stories.tsx
import buttonSchema from '../schemas/components/Button.schema.json';
import { Button } from './Button';

// ✅ 從 Schema 自動生成 argTypes
const argTypesFromSchema = (schema) => {
  const argTypes = {};
  Object.entries(schema.properties).forEach(([key, prop]) => {
    if (prop.enum) {
      argTypes[key] = {
        options: prop.enum,
        control: { type: 'select' },
        description: prop.description,
        defaultValue: prop.default
      };
    } else if (prop.type === 'boolean') {
      argTypes[key] = {
        control: { type: 'boolean' },
        description: prop.description,
        defaultValue: prop.default
      };
    }
  });
  return argTypes;
};

export default {
  title: 'Components/Button',
  component: Button,
  argTypes: argTypesFromSchema(buttonSchema)  // ✅ 自動生成!
};

// ✅ 從 examples 自動生成 stories
export const Primary = {
  args: buttonSchema.examples[0]
};

export const Danger = {
  args: buttonSchema.examples[1]
};
```

### 5. 後端配置驗證 (Rust)

```rust
use jsonschema::JSONSchema;
use serde_json::json;

lazy_static! {
    static ref BUTTON_SCHEMA: JSONSchema = {
        let schema = include_str!("../schemas/components/Button.schema.json");
        JSONSchema::compile(&serde_json::from_str(schema).unwrap()).unwrap()
    };
}

// ✅ 驗證前端傳來的按鈕配置
fn validate_button_config(config: &Value) -> Result<(), String> {
    if BUTTON_SCHEMA.is_valid(config) {
        Ok(())
    } else {
        let errors = BUTTON_SCHEMA.iter_errors(config)
            .map(|e| e.to_string())
            .collect();
        Err(format!("Invalid button config: {:?}", errors))
    }
}

// 使用場景: 用戶自定義 UI 配置
let button_config = json!({
    "variant": "primary",
    "size": "lg",
    "children": "Submit"
});

validate_button_config(&button_config)?;  // ✅ 自動驗證
```

## 🌟 組件庫架構

### Schema Registry: 組件規格中央庫

```
schemas/components/
├── Button.schema.json          # 按鈕組件
├── Input.schema.json           # 輸入框組件
├── Modal.schema.json           # 彈窗組件
├── Table.schema.json           # 表格組件
├── Form.schema.json            # 表單組件
│
├── shared/                     # 共用規格
│   ├── Color.schema.json       # 顏色規範
│   ├── Size.schema.json        # 尺寸規範
│   └── Spacing.schema.json     # 間距規範
│
└── compositions/               # 組合組件
    ├── LoginForm.schema.json   # 登錄表單 (Button + Input)
    └── DataTable.schema.json   # 數據表格 (Table + Input + Button)
```

### 共用規格: $ref 引用

**schemas/shared/Size.schema.json**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://ui.example.com/shared/Size",
  "title": "Component Size",
  "type": "string",
  "enum": ["sm", "md", "lg", "xl"],
  "default": "md"
}
```

**schemas/components/Button.schema.json** (使用共用規格):
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "size": {
      "$ref": "https://ui.example.com/shared/Size"
    }
  }
}
```

**好處**:
- ✅ 所有組件的 size 定義完全一致
- ✅ 修改 Size.schema.json → 所有組件自動更新
- ✅ 設計系統規範強制執行

### 組合組件: 引用其他組件

**schemas/compositions/LoginForm.schema.json**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Login Form",
  "type": "object",
  "properties": {
    "usernameInput": {
      "$ref": "https://ui.example.com/components/Input",
      "properties": {
        "type": {"const": "text"},
        "placeholder": {"const": "Username"}
      }
    },
    "passwordInput": {
      "$ref": "https://ui.example.com/components/Input",
      "properties": {
        "type": {"const": "password"},
        "placeholder": {"const": "Password"}
      }
    },
    "submitButton": {
      "$ref": "https://ui.example.com/components/Button",
      "properties": {
        "variant": {"const": "primary"},
        "children": {"const": "Login"}
      }
    }
  }
}
```

**自動生成組合組件**:
```typescript
import { Input } from './Input';
import { Button } from './Button';
import type { LoginFormProps } from './types';

export function LoginForm(props: LoginFormProps) {
  return (
    <form>
      <Input {...props.usernameInput} />
      <Input {...props.passwordInput} />
      <Button {...props.submitButton} />
    </form>
  );
}
```

## 🔧 工具鏈自動化

### 1. 開發時自動生成

```json
// package.json
{
  "scripts": {
    "generate:types": "json2ts schemas/components/*.json -o src/types/",
    "generate:stories": "node scripts/generate-stories.js",
    "generate:docs": "jsonschema2md schemas/components/ -o docs/",
    "generate:all": "npm run generate:types && npm run generate:stories && npm run generate:docs",
    "dev": "npm run generate:all && vite"
  }
}
```

### 2. Watch 模式: Schema 改變自動重新生成

```javascript
// scripts/watch-schemas.js
import chokidar from 'chokidar';
import { execSync } from 'child_process';

chokidar.watch('schemas/**/*.schema.json').on('change', (path) => {
  console.log(`Schema changed: ${path}`);
  execSync('npm run generate:all');
  console.log('✅ Types, stories, and docs regenerated');
});
```

### 3. CI/CD 驗證

```yaml
# .github/workflows/component-library.yml
name: Component Library CI

on: [push, pull_request]

jobs:
  validate-schemas:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Validate JSON Schemas
        run: |
          npm install -g ajv-cli
          ajv compile -s schemas/**/*.schema.json

      - name: Check Types Generated
        run: |
          npm run generate:types
          git diff --exit-code src/types/

      - name: Validate Examples
        run: |
          for schema in schemas/components/*.json; do
            ajv validate -s "$schema" -d "$schema" --valid
          done

      - name: Build Storybook
        run: |
          npm run generate:stories
          npm run build-storybook
```

## 📊 實際效益

### 開發效率提升

**添加新組件**:

傳統方式 (2-3 小時):
- ❌ 寫 TypeScript interface (30 分鐘)
- ❌ 實現組件邏輯 (1 小時)
- ❌ 寫 Storybook stories (30 分鐘)
- ❌ 寫驗證邏輯 (30 分鐘)
- ❌ 更新文檔 (30 分鐘)

Schema-DD 方式 (30 分鐘):
- ✅ 寫 Schema (10 分鐘)
- ✅ 實現組件邏輯 (20 分鐘)
- ✅ Types/Stories/Docs 自動生成 (0 分鐘)

**效率提升**: **4-6 倍**

### 維護成本降低

**修改組件 Props**:

傳統方式:
- ❌ 修改 interface → 修改文檔 → 修改 Storybook → 修改驗證 → 通知前後端
- ❌ 容易漏改,導致不一致

Schema-DD 方式:
- ✅ 修改 Schema → 自動生成一切 → CI 自動驗證
- ✅ 單一事實來源,不可能不一致

**維護成本**: **降低 80%**

### 設計系統一致性

**強制規範**:
```json
// schemas/shared/Color.schema.json
{
  "enum": ["primary", "secondary", "success", "danger", "warning", "info"]
}
```

所有組件的 `color` prop 自動強制使用這 6 種顏色:
- ✅ Button.schema.json → `$ref: Color`
- ✅ Badge.schema.json → `$ref: Color`
- ✅ Alert.schema.json → `$ref: Color`

**結果**: 設計系統 100% 一致,無法違反

## 🎯 使用場景

### 1. UI 組件庫 (最佳場景)

- ✅ Material-UI, Ant Design, Chakra UI 風格組件
- ✅ Props 自動驗證 + 類型生成
- ✅ Storybook 自動生成
- ✅ 設計系統規範強制執行

### 2. 低代碼平台

```json
// 用戶拖拽配置 UI
{
  "page": {
    "header": {
      "$ref": "Header",
      "logo": "logo.png",
      "nav": [...]
    },
    "content": {
      "$ref": "LoginForm",
      "submitButton": {
        "variant": "primary"
      }
    }
  }
}
```

**自動驗證**: Schema 確保配置合法
**自動渲染**: 動態生成 UI

### 3. 配置管理系統

```json
// 後台配置前端 UI
{
  "dashboards": [
    {
      "$ref": "Dashboard",
      "widgets": [
        {"$ref": "ChartWidget", "type": "line"},
        {"$ref": "TableWidget", "columns": [...]}
      ]
    }
  ]
}
```

### 4. 多語言組件庫

**同一個 Schema**:
- TypeScript (前端組件)
- Rust (後端渲染/驗證)
- Swift (iOS 原生組件)
- Kotlin (Android 原生組件)

**保證**: 所有平台組件 Props 100% 一致

## 🚀 最佳實踐

### 1. Schema 版本管理

```
schemas/
├── v1/
│   └── components/
│       └── Button.schema.json
├── v2/
│   └── components/
│       └── Button.schema.json  (Breaking change: 移除 'outline' variant)
└── current -> v2/
```

### 2. 漸進式採用

```typescript
// 階段 1: 新組件用 Schema
Button.schema.json → Button.tsx

// 階段 2: 舊組件遷移
OldModal.tsx → Modal.schema.json → Modal.tsx

// 階段 3: 全部組件 Schema-driven
100% 組件有 Schema → 自動生成文檔站
```

### 3. Schema 即文檔

```json
{
  "properties": {
    "variant": {
      "description": "按鈕視覺風格。Primary 用於主要操作,Danger 用於破壞性操作",
      "examples": ["primary", "danger"]
    }
  }
}
```

**自動生成 Markdown 文檔**:
```markdown
## variant

按鈕視覺風格。Primary 用於主要操作,Danger 用於破壞性操作

- Type: `string`
- Enum: `primary`, `secondary`, `danger`, `ghost`
- Default: `primary`
- Examples: `primary`, `danger`
```

## 📖 延伸閱讀

- **JSON-as-Object Pattern**: `json-as-object.md`
- **Schema Registry**: `../examples/schema-registry.rs`
- **Rust 實現**: `../references/rust/jsonschema-runtime.md`
- **TypeScript 實現**: `../references/typescript/ajv-validation.md`

## 🎉 總結

**Component Library Pattern 核心價值**:

1. **組件 = Schema**: Props 定義即規格
2. **自動生成一切**: Types + Stories + Docs + Validation
3. **設計系統強制**: 共用規格 ($ref) 保證一致性
4. **跨平台對齊**: 前後端、Web/iOS/Android 完美同步
5. **效率提升 4-6 倍**: 30 分鐘 vs 2-3 小時

**這就是 Schema-DD 的組件化殺手級應用**!
