# Lency Diagnostics 实现计划

## 目标

统一所有模块的错误处理系统

---

## 核心设计

### 类型定义

```rust
// lency_diagnostics/src/lib.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<String>,
}

pub struct DiagnosticSink {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticSink {
    pub fn error(&mut self, msg: impl Into<String>) -> DiagnosticBuilder;
    pub fn warning(&mut self, msg: impl Into<String>) -> DiagnosticBuilder;
    pub fn has_errors(&self) -> bool;
    pub fn emit_all(&self);
}
```

---

## 集成计划

### Step 1: 实现基础类型 (1小时)

```bash
# 编辑 lency_diagnostics/src/lib.rs
vim crates/lency_diagnostics/src/lib.rs
```

### Step 2: 更新 syntax (30分钟)

```rust
// lency_syntax/src/error.rs
use lency_diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticSink};

pub type ParseResult<T> = Result<T, Vec<Diagnostic>>;
```

### Step 3: 更新 sema (1小时)

```rust
// lency_sema/src/error.rs
use lency_diagnostics::Diagnostic;

impl From<SemanticError> for Diagnostic {
    fn from(err: SemanticError) -> Self {
        // 转换逻辑
    }
}
```

### Step 4: 更新 codegen (30分钟)

类似 sema 的处理

---

## 使用示例

```rust
let mut sink = DiagnosticSink::new();

sink.error("type mismatch")
    .span(expr.span)
    .note("expected int, found string")
    .note("in function call to 'foo'")
    .suggest("try: foo(x.to_int())")
    .emit();

if sink.has_errors() {
    sink.emit_all();
    return Err(());
}
```

---

## 预期效果

- ✅ 统一的错误格式
- ✅ 更好的错误信息
- ✅ 支持建议和注释
- ✅ 便于 LSP 集成
