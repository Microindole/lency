# 工具脚本指南

Lency 项目的脚本位于 `scripts/` 目录，用于统一质量门禁与自举链路验证。
平台实现已分层：
- `scripts/linux/`：Linux/macOS 的 `.sh` 实现。
- `scripts/win/`：Windows 的 `.ps1` 实现。
主检查逻辑已收敛到 `xtask`：
- `cargo run -p xtask -- auto-check`
- `cargo run -p xtask -- check-rust`
- `cargo run -p xtask -- check-lency`

推荐：
- 日常开发默认使用 `auto-check`，根据 Git 改动范围自动决定跑 `check-rust`、`check-lency` 或两者都跑。

## 1. Rust 侧检查入口

脚本: `scripts/linux/run_checks.sh`
Windows 对应: `scripts/win/run_checks.ps1`

固定执行顺序：
1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -D warnings`
3. `cargo test`
4. `scripts/check_file_size.py --scope rust`
5. `scripts/check_todos.py --scope rust`
6. `scripts/check_banned_patterns.py --scope rust`

用法：
```bash
cargo run -p xtask -- check-rust
```
```bash
./scripts/linux/run_checks.sh
```
```powershell
.\scripts\win\run_checks.ps1
```

约束：
- 不接受参数。
- 除 TODO 扫描外，任一步骤失败都会直接退出。

## 2. Lency 自举检查入口

脚本: `scripts/linux/run_lency_checks.sh`
Windows 对应: `scripts/win/run_lency_checks.ps1`

固定执行内容：
1. 构建 Rust 宿主编译器（`lency_cli` + `lency_runtime`）
2. 元检查（TODO/FIXME、文件大小、Lency 命名规范）
3. 入口语法检查（`--check-only` 可用时）
4. 编译并运行 `tests/example/selfhost/driver/test_entry.lcy`
5. 编译并运行 `lencyc/driver/main.lcy`
6. 校验主流程 AST 产物
7. 自举 LIR 回归样例校验
8. Rust `.lir -> LLVM -> executable` 冒烟
9. 一键构建脚本冒烟（`lency_selfhost_build.sh`）
10. 一键运行脚本冒烟（`lency_selfhost_run.sh`，含参数透传）
11. runtime builtin 映射回归（`int_to_string` 等）

用法：
```bash
cargo run -p xtask -- check-lency
```
```bash
./scripts/linux/run_lency_checks.sh
```
```powershell
.\scripts\win\run_lency_checks.ps1
```

约束：
- 不接受参数。

## 3. 自举一键构建

主入口: `cargo run -p xtask -- selfhost-build ...`
脚本: `scripts/linux/lency_selfhost_build.sh` / `scripts/win/lency_selfhost_build.ps1`

作用：`.lcy -> self-host emit-lir -> Rust build executable`

用法：
```bash
cargo run -p xtask -- selfhost-build <input.lcy> [-o output] [--out-dir DIR] [--check-only] [--release]
```
```bash
./scripts/linux/lency_selfhost_build.sh <input.lcy> [-o output] [--out-dir DIR] [--check-only] [--release]
```
```powershell
.\scripts\win\lency_selfhost_build.ps1 <input.lcy> [-o output] [--out-dir DIR] [--check-only] [--release]
```

## 4. 自举一键运行

主入口: `cargo run -p xtask -- selfhost-run ...`
脚本: `scripts/linux/lency_selfhost_run.sh` / `scripts/win/lency_selfhost_run.ps1`

作用：`.lcy -> self-host build -> run`，支持参数透传和期望退出码校验。

用法：
```bash
cargo run -p xtask -- selfhost-run <input.lcy> [--release] [--out-dir DIR] [--expect-exit N] [--] [program args...]
```
```bash
./scripts/linux/lency_selfhost_run.sh <input.lcy> [--release] [--out-dir DIR] [--expect-exit N] [--] [program args...]
```
```powershell
.\scripts\win\lency_selfhost_run.ps1 <input.lcy> [--release] [--out-dir DIR] [--expect-exit N] [--] [program args...]
```

## 5. 其他辅助脚本

- `scripts/linux/run_lcy_tests.sh` / `scripts/win/run_lcy_tests.ps1`: `.lcy` 集成测试入口；现已纳入 `xtask check-rust` 主流程。
- `scripts/check_file_size.py`: 文件规模检查。
- `scripts/check_todos.py`: TODO/FIXME 扫描。
- `scripts/check_banned_patterns.py`: 禁用模式扫描。
- `scripts/check_lencyc_meta.py`: Lency 命名与结构元规则检查。
- `scripts/check_commit_messages.py`: CI 提交信息校验（逐条提交检查，支持 `push`/`pull_request` 事件范围）。
- `scripts/win/setup-dev.ps1`: Windows 开发环境初始化（自动探测 LLVM 15，设置 `LLVM_SYS_150_PREFIX`）。
- `scripts/linux/setup-dev.sh`: Linux/macOS 开发环境初始化（自动探测 LLVM 15，设置 `LLVM_SYS_150_PREFIX`）。

说明：
- `tests/example/` 已按用途分层为 `lir/`、`runtime/`、`parser/`、`modules/`、`selfhost/`，新增回归用例应放入对应子目录。

## 6. 跨平台 LLVM 环境初始化

`inkwell` 依赖 LLVM 15。项目不再在仓库中硬编码 `LLVM_SYS_150_PREFIX`，请通过脚本或手动环境变量设置。

Windows（PowerShell）：
```powershell
.\scripts\win\setup-dev.ps1
.\scripts\win\setup-dev.ps1 -Persist
.\scripts\win\setup-dev.ps1 -LlvmPrefix D:\tools\llvm-15.0.7-custom -Persist
```

Linux/macOS（bash/zsh）：
```bash
./scripts/linux/setup-dev.sh
./scripts/linux/setup-dev.sh --persist
./scripts/linux/setup-dev.sh --llvm-prefix /usr/lib/llvm-15 --persist
```

说明：
- 以上脚本只在你手动执行时生效，不会在 clone 后自动运行。
- 脚本会校验 `llvm-config --version` 必须为 `15.x`。
- 运行完成后建议执行 `cargo build -v` 验证。

## 7. 提交信息校验

脚本: `scripts/check_commit_messages.py`

作用：
- 校验一次 `push` 或一次 `PR` 范围内的每一条提交，而不是只看最新一条。
- 支持两类提交主题：
1. Conventional Commit 风格：`<type>: <subject>` 或 `<type>(scope): <subject>`
2. GitHub 合并提交：`Merge pull request #... from ...` / `Merge branch '...' ...` / `Merge remote-tracking branch '...' ...` / `Merge tag '...' of ...`

当前允许 `type`：
- `feat`、`fix`、`style`、`refactor`、`build`、`ci`、`docs`、`perf`、`test`、`chore`

输出说明：
- 校验日志为中英文双语，包含失败提交序号（第 N 条）、SHA、提交主题、允许格式与示例，便于直接回改。

用法（本地手动调试）：
```bash
GITHUB_EVENT_NAME=workflow_dispatch python3 scripts/check_commit_messages.py
```
