# TypeScript: JSON Schema Validation with Ajv

**核心理念**: 與後端共用相同 schema,前端運行時驗證

## Why Ajv

**優勢**:
- ✅ 與 Rust jsonschema 共用相同 JSON Schema 文件
- ✅ 運行時驗證,捕捉動態數據錯誤
- ✅ 高性能 (預編譯 schema)
- ✅ 支持 JSON Schema Draft 7
- ✅ 自動生成 TypeScript 類型

## Installation

```bash
npm install ajv
npm install -D json-schema-to-typescript  # For type generation
```

## Basic Usage

### 1. Inline Validation

```typescript
import Ajv from 'ajv';
import addFormats from 'ajv-formats';

const ajv = new Ajv();
addFormats(ajv);  // Add format validators (email, date-time, etc.)

const schema = {
  type: 'object',
  properties: {
    email: { type: 'string', format: 'email' },
    name: { type: 'string', minLength: 1 }
  },
  required: ['email', 'name']
};

const validate = ajv.compile(schema);

function validateUser(data: unknown): data is User {
  const valid = validate(data);
  if (!valid) {
    console.error(validate.errors);
    return false;
  }
  return true;
}
```

### 2. External Schema File

```typescript
import userSchema from '../schemas/user.schema.json';

const validateUser = ajv.compile(userSchema);

export function isValidUser(data: unknown): data is User {
  return validateUser(data);
}

export function assertValidUser(data: unknown): asserts data is User {
  if (!validateUser(data)) {
    throw new Error(`Invalid user: ${JSON.stringify(validateUser.errors)}`);
  }
}
```

## Type Generation

### Generate TypeScript Types from Schema

```bash
# Generate types
json2ts schemas/user.schema.json > src/types/user.ts

# Or use script in package.json
npm run generate-types
```

**package.json**:
```json
{
  "scripts": {
    "generate-types": "json2ts schemas/*.schema.json -o src/types/"
  }
}
```

**Generated types/user.ts**:
```typescript
export interface User {
  email: string;
  name: string;
  age?: number;
  roles?: ('admin' | 'user' | 'guest')[];
}
```

**Perfect sync**: Schema 變更 → 自動生成新類型

## API Integration

### Fetch with Validation

```typescript
import { isValidUser } from './validators';
import type { User } from './types/user';

async function fetchUser(id: string): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  const data = await response.json();

  if (!isValidUser(data)) {
    throw new Error('Server returned invalid user data');
  }

  return data;  // TypeScript knows it's User now
}
```

### Form Validation

```typescript
function handleSubmit(formData: FormData) {
  const userData = Object.fromEntries(formData);

  if (!validateUser(userData)) {
    // Show validation errors
    const errors = validateUser.errors || [];
    showErrors(errors.map(e => `${e.instancePath}: ${e.message}`));
    return;
  }

  // Submit to server
  api.createUser(userData as User);
}
```

## React Integration

### Custom Hook

```typescript
import { useState } from 'react';

export function useValidation<T>(
  validator: (data: unknown) => boolean
) {
  const [errors, setErrors] = useState<string[]>([]);

  const validate = (data: unknown): data is T => {
    const valid = validator(data);
    if (!valid && validator.errors) {
      setErrors(validator.errors.map(e => e.message || ''));
    } else {
      setErrors([]);
    }
    return valid;
  };

  return { validate, errors };
}

// Usage
function UserForm() {
  const { validate, errors } = useValidation<User>(validateUser);

  const handleSubmit = (data: unknown) => {
    if (validate(data)) {
      api.createUser(data);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      {errors.map(err => <p className="error">{err}</p>)}
      {/* Form fields */}
    </form>
  );
}
```

## Schema Registry for Multiple Types

```typescript
import Ajv from 'ajv';
import userSchema from '../schemas/user.schema.json';
import sessionSchema from '../schemas/session.schema.json';

const ajv = new Ajv({ schemas: [userSchema, sessionSchema] });

// Access by $id
const validateUser = ajv.getSchema<User>('https://api.example.com/schemas/user.json');
const validateSession = ajv.getSchema<Session>('https://api.example.com/schemas/session.json');

// Type-safe validation
export function isUser(data: unknown): data is User {
  return validateUser!(data);
}
```

## Cross-Language Consistency

**Same Schema, Different Languages**:
- Backend (Rust): jsonschema crate → `../rust/jsonschema-runtime.md`
- Frontend (TypeScript): ajv → This file
- Python: jsonschema → `../python/jsonschema-validation.md`

**Guarantee**: All validate identically

## CI/CD Integration

See: `../../examples/ci-cd-workflow.yml`

Key steps:
1. Schema syntax validation (ajv-cli)
2. Generate TypeScript types
3. Type check with tsc
4. Run tests

## Testing

```typescript
import { describe, it, expect } from 'vitest';

describe('User Schema Validation', () => {
  it('validates correct user', () => {
    const valid = {
      email: 'user@example.com',
      name: 'John Doe'
    };

    expect(validateUser(valid)).toBe(true);
  });

  it('rejects invalid email', () => {
    const invalid = {
      email: 'not-an-email',
      name: 'John'
    };

    expect(validateUser(invalid)).toBe(false);
    expect(validateUser.errors).toBeDefined();
  });
});
```

## Resources

- **Ajv Documentation**: https://ajv.js.org/
- **json-schema-to-typescript**: https://github.com/bcherny/json-schema-to-typescript
- **JSON Schema**: https://json-schema.org/

## Next Steps

- Read: `type-generation.md` - 深入類型生成
- Try: `../../examples/user-api/frontend/` - 完整前端範例
- Study: `../../case-studies/claude-tui-rs/` - 跨語言一致性案例
