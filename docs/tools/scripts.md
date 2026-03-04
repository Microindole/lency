# 工具脚本指南

Lency 项目的脚本位于 `scripts/` 目录，用于统一质量门禁与自举链路验证。

## 1. Rust 侧检查入口

脚本: `scripts/run_checks.sh`

固定执行顺序：
1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -D warnings`
3. `cargo test`
4. `scripts/check_file_size.py --scope rust`
5. `scripts/check_todos.py --scope rust`
6. `scripts/check_banned_patterns.py --scope rust`

用法：
```bash
./scripts/run_checks.sh
```

约束：
- 不接受参数。
- 除 TODO 扫描外，任一步骤失败都会直接退出。

## 2. Lency 自举检查入口

脚本: `scripts/run_lency_checks.sh`

固定执行内容：
1. 构建 Rust 宿主编译器（`lency_cli` + `lency_runtime`）
2. 元检查（TODO/FIXME、文件大小、Lency 命名规范）
3. 入口语法检查（`--check-only` 可用时）
4. 编译并运行 `lencyc/driver/test_entry.lcy`
5. 编译并运行 `lencyc/driver/main.lcy`
6. 校验主流程 AST 产物
7. 自举 LIR 回归样例校验
8. Rust `.lir -> LLVM -> executable` 冒烟
9. 一键构建脚本冒烟（`lency_selfhost_build.sh`）
10. 一键运行脚本冒烟（`lency_selfhost_run.sh`，含参数透传）
11. runtime builtin 映射回归（`int_to_string` 等）

用法：
```bash
./scripts/run_lency_checks.sh
```

约束：
- 不接受参数。

## 3. 自举一键构建

脚本: `scripts/lency_selfhost_build.sh`

作用：`.lcy -> self-host emit-lir -> Rust build executable`

用法：
```bash
./scripts/lency_selfhost_build.sh <input.lcy> [-o output] [--out-dir DIR] [--check-only] [--release]
```

## 4. 自举一键运行

脚本: `scripts/lency_selfhost_run.sh`

作用：`.lcy -> self-host build -> run`，支持参数透传和期望退出码校验。

用法：
```bash
./scripts/lency_selfhost_run.sh <input.lcy> [--release] [--out-dir DIR] [--expect-exit N] [--] [program args...]
```

## 5. 其他辅助脚本

- `scripts/run_lcy_tests.sh`: 独立 `.lcy` 集成测试入口（不在 `run_checks.sh` 主流程中）。
- `scripts/check_file_size.py`: 文件规模检查。
- `scripts/check_todos.py`: TODO/FIXME 扫描。
- `scripts/check_banned_patterns.py`: 禁用模式扫描。
- `scripts/check_lencyc_meta.py`: Lency 命名与结构元规则检查。
