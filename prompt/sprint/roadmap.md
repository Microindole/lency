# Lency 编译器架构规划

> **更新时间**: 2026-01-16  
> **版本**: 2.0

## 🏗️ 编译器架构

### 核心模块划分

```
lency_cli           # 命令行入口
lency_driver        # 编译流程驱动器
├── lency_syntax    # 词法+语法分析
├── lency_sema      # 语义分析
├── lency_monomorph # 泛型单态化 ⚠️ 待重构
├── lency_codegen   # LLVM 代码生成
└── lency_runtime   # 运行时库

lency_diagnostics   # 统一错误诊断 ⚠️ 待实现
```

### ✅ 架构状态

**已完成**:
- ✅ `lency_monomorph` 已迁移到独立 crate，被 `lency_driver` 使用
- ✅ `lency_diagnostics` 统一诊断系统已实现并集成
  - `SemanticError` 和 `CodegenError` 支持转换为 `Diagnostic`
  - `CompileError` 支持 `collect_to_sink()` 和 `emit()` 方法

---

## 📋 重构计划

### Phase 1: 模块化重构 (本周)

#### 1.1 重构 lency_monomorph
将 `lency_sema/src/monomorphize/` 迁移到 `lency_monomorph`

**迁移内容**:
- `collector.rs` - 收集泛型实例化
- `mangling.rs` - 名称混淆
- `rewriter.rs` - AST 重写
- `specializer.rs` - 模板特化

**依赖关系**:
```
lency_monomorph
├── depends on: lency_syntax (AST)
└── used by: lency_driver
```

#### 1.2 实现 lency_diagnostics

**核心类型**:
```rust
pub struct Diagnostic {
    level: DiagnosticLevel,  // Error/Warning/Info
    message: String,
    span: Span,
    notes: Vec<String>,
    suggestions: Vec<Suggestion>,
}

pub struct DiagnosticSink {
    diagnostics: Vec<Diagnostic>,
}
```

**使用者**:
- lency_syntax (解析错误)
- lency_sema (语义错误)
- lency_codegen (代码生成错误)

---

## 🎯 功能完成度

### 已完成 (65%)

- ✅ **词法分析** - Logos
- ✅ **语法分析** - Chumsky
- ✅ **语义分析** - 完整
  - 名称解析、类型推断、类型检查
  - 空安全、泛型约束
- ✅ **泛型系统** - 单态化（需重构）
- ✅ **代码生成** - LLVM IR
- ✅ **运行时** - Vec, HashMap, 文件 I/O
- ✅ **标准库** - 5个模块

### 进行中 (20%)

- 🔄 **错误诊断** - 基础实现
- 🔄 **标准库** - 需扩展
- 🔄 **测试覆盖** - 53个测试

### 未开始 (15%)

- ❌ **包管理器**
- ❌ **LSP 支持**
- ❌ **调试信息** (DWARF)
- ❌ **自举编译器**

---

## 🚀 开发路线图

### Sprint 14: 架构重构 ✅ 完成

**目标**: 模块化清晰，职责分明

- [x] 迁移单态化到 lency_monomorph
- [x] 实现 lency_diagnostics
- [x] 更新所有模块使用新错误系统
- [x] 文档和注释完善

### Sprint 15: 泛型增强 ✅ 完成

**目标**: Result 内置方法支持

- [x] Result<T, E> 内置方法 (is_ok, is_err, unwrap_or)
- [ ] Option<T> 方法支持 (可选)
- [ ] 模式匹配增强 (可选)

### Sprint 16: 标准库扩展 (1-2周)

**目标**: 实用的标准库

- [ ] Result/Option 方法链
- [ ] lib/json - JSON 解析
- [ ] lib/http - HTTP 客户端（基础）
- [ ] std/time - 时间处理

### Sprint 17-20: 工具链 (1-2月)

**包管理器**:
- [ ] lency.toml 配置
- [ ] 依赖解析
- [ ] 包下载和缓存

**语言服务器**:
- [ ] LSP 基础框架
- [ ] 代码补全
- [ ] 跳转定义
- [ ] 错误提示

### Sprint 21+: 自举准备 (3-6月)

**用 Lency 重写编译器**:
1. 数据结构（Vec, HashMap, String）
2. Lexer（正则引擎）
3. Parser（解析器组合子）
4. AST 和语义分析
5. 代码生成（LLVM 绑定）

---

## 📐 设计原则

### 1. 清晰的模块边界

每个 crate 只做一件事：
- `syntax` - 只关心 AST
- `sema` - 只关心语义
- `monomorph` - 只关心单态化
- `codegen` - 只关心 LLVM

### 2. 统一的错误处理

所有错误通过 `diagnostics` 报告：
```rust
use lency_diagnostics::{Diagnostic, DiagnosticLevel};

// 统一格式
let diag = Diagnostic::error("type mismatch")
    .span(expr.span)
    .note("expected int, found string")
    .suggest("try using int_to_string()");
```

### 3. 最小依赖

- `syntax` 不依赖其他 lency crate
- `sema` 只依赖 `syntax` 和 `diagnostics`
- `monomorph` 只依赖 `syntax`
- `codegen` 依赖处理后的 AST

---

## 🎨 语言特色

遵循 assets/Lency.txt 的哲学：

1. **简洁** - 无冗余语法
2. **规范** - C系结构
3. **清晰** - 明确的错误信息
4. **安全** - 默认空安全

---

## 📊 下一步行动

### 立即执行 (今天)

1. ✅ 更新 assets/roadmap.md
2. ✅ Sprint 14 架构重构完成
3. [ ] 开始 Sprint 15 泛型增强

### 本周完成

4. [ ] 完成模块迁移
5. [ ] 实现基础 diagnostics
6. [ ] 通过所有测试

### 本月目标

7. [ ] 泛型 enum 完整支持
8. [ ] 标准库扩展到 10+ 模块
9. [ ] 100+ 集成测试
