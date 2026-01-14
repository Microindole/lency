# 工具脚本指南

Lency 项目包含一套实用脚本，位于 `scripts/` 目录下，用于维护代码质量和提高开发效率。

## 1. 运行所有检查

**脚本**: `scripts/run_checks.sh`

这是提交代码前推荐运行的主脚本。它会按顺序执行以下检查：
1. `cargo fmt` (格式化检查)
2. `cargo clippy` (Lint 检查)
3. `cargo test` (单元测试)
4. `run_lcy_tests.sh` (.lcy 集成测试)
5. `check_file_size.py` (文件大小检查)
6. `check_todos.py` (扫描 TODOs)

**用法**:
```bash
./scripts/run_checks.sh
```

如果任何步骤失败（TODO 检查除外），脚本将以非零状态码退出。

## 2. .lcy 集成测试

**脚本**: `scripts/run_lcy_tests.sh`

用于测试 `tests/integration/` 目录下的所有 `.lcy` 文件，防止修复 bug 时破坏已有语言特性。

**预期失败标记**:
在 `.lcy` 文件开头添加 `// @expect-error` 注释可以标记该测试为预期失败：
- `// @expect-error` - 纯粹的错误检测测试
- `// @expect-error: TODO - 描述` - 功能尚未实现
- `// @expect-error: FIXME - 描述` - 编译器 bug 需要修复

**用法**:
```bash
./scripts/run_lcy_tests.sh
```

**输出示例**:
```
✅ tests/integration/enums/match_runtime.lcy
🔶 tests/integration/adt/match_exhaust_fail.lcy (expected failure)
❌ tests/integration/some_broken_test.lcy

📊 Results: 30 passed, 16 expected failures, 1 unexpected failures
```

## 3. 文件大小检查

**脚本**: `scripts/check_file_size.py`

用于确保源文件不会变得过大，难以维护。支持 Rust (`.rs`) 和 Python (`.py`) 文件。

**配置**:
- **警告阈值**: 300 行代码（不含注释和空行）
- **错误阈值**: 500 行代码

**用法**:
```bash
python3 scripts/check_file_size.py
```

## 4. TODO 扫描

**脚本**: `scripts/check_todos.py`

扫描代码库中的 `TODO`, `FIXME`, `XXX` 标记，帮助跟踪技术债务。

**特性**:
- 扫描普通代码中的 TODO/FIXME
- 单独列出预期失败测试中的 TODO（功能未实现）和 FIXME（编译器 bug）

**用法**:
```bash
python3 scripts/check_todos.py
```

**输出示例**:
```
📝 Found 20 TODOs:
   lib/std/io.lcy:11   // TODO: 需要更好的实现
   ...

🔴 Found 2 FIXMEs:
   crates/lency_codegen/src/module.rs:100  // FIXME: 临时方案
   ...

📋 预期失败测试 (功能未实现 - 6 个):
   tests/integration/arrays/vec_simple.lcy: print() 不支持 Vec 类型
   ...

🐛 预期失败测试 (需要修复 Bug - 5 个):
   tests/integration/structs/methods.lcy: 方法调用代码生成存在 bug
   ...
```

## 为何需要这些脚本？

Lency 遵循严格的工程标准：
- **可维护性**: 通过限制文件大小，强制进行模块化拆分。
- **代码质量**: 通过强制 Lint 和 Format，保持代码风格一致。
- **可追踪性**: 通过扫描 TODOs，防止遗忘临时代码。
- **回归测试**: 通过 .lcy 集成测试，防止修复 bug 时破坏语言特性。
