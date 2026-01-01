# 工具脚本指南

Beryl 项目包含一套实用脚本，位于 `scripts/` 目录下，用于维护代码质量和提高开发效率。

## 1. 运行所有检查

**脚本**: `scripts/run_checks.sh`

这是提交代码前推荐运行的主脚本。它会按顺序执行以下检查：
1. `cargo fmt` (格式化检查)
2. `cargo clippy` (Lint 检查)
3. `cargo test` (单元测试)
4. `check_file_size.py` (文件大小检查)
5. `check_todos.py` (扫描 TODOs)

**用法**:
```bash
./scripts/run_checks.sh
```

如果任何步骤失败（TODO 检查除外），脚本将以非零状态码退出。

## 2. 文件大小检查

**脚本**: `scripts/check_file_size.py`

用于确保源文件不会变得过大，难以维护。

**配置**:
- **警告阈值**: 300 行
- **错误阈值**: 500 行

**用法**:
```bash
python3 scripts/check_file_size.py
```

## 3. TODO 扫描

**脚本**: `scripts/check_todos.py`

扫描代码库中的 `TODO`, `FIXME`, `XXX` 标记，帮助跟踪技术债务。

**用法**:
```bash
python3 scripts/check_todos.py
```

## 为何需要这些脚本？

Beryl 遵循严格的工程标准：
- **可维护性**: 通过限制文件大小，强制进行模块化拆分。
- **代码质量**: 通过强制 Lint 和 Format，保持代码风格一致。
- **可追踪性**: 通过扫描 TODOs，防止遗忘临时代码。
