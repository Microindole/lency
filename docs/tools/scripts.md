# 工具与脚本

## 环境

编译器依赖 Rust、系统链接器和 LLVM 15。`inkwell` 通过 `LLVM_SYS_150_PREFIX` 定位 LLVM。

Windows：

```powershell
.\scripts\win\setup-dev.ps1 -LlvmPrefix D:\tools\llvm-15.0.7-custom -Persist
```

Linux/macOS：

```bash
./scripts/linux/setup-dev.sh --llvm-prefix /usr/lib/llvm-15 --persist
```

脚本会校验 `llvm-config --version` 为 15.x。设置永久环境变量后，应重新打开终端。

## 主检查入口

| 命令 | 用途 |
|---|---|
| `cargo run -p xtask -- check-lency` | 当前主线：selfhost 构建、语义、LIR 和 runtime 回归 |
| `cargo run -p xtask -- bootstrap-check` | stage1/stage2/stage3 自举收敛 |
| `cargo run -p xtask -- check-rust` | Rust 母体格式、Clippy、测试和 `.lcy` 集成用例 |
| `cargo run -p xtask -- auto-check` | 按 Git 未提交文件选择检查范围 |

首次搭建环境应显式运行 `check-lency`；修改 Rust 母体时再运行 `check-rust`。干净工作区的 `auto-check` 会回退到 `check-lency`。

## 普通程序使用 Rust 母体

selfhost 尚未成为默认用户编译器。当前检查、运行和构建普通 Lency 程序使用：

```powershell
cargo run -p lency_cli -- check <input.lcy>
cargo run -p lency_cli -- run <input.lcy>
cargo run -p lency_cli -- build <input.lcy> -o <name>
```

## selfhost 构建与运行

```powershell
cargo run -p xtask -- selfhost-build <input.lcy> [-o output] [--out-dir DIR] [--check-only] [--release]
cargo run -p xtask -- selfhost-run <input.lcy> [--release] [--out-dir DIR] [--expect-exit N] [--] [program args...]
```

平台包装脚本位于：

- `scripts/linux/lency_selfhost_build.sh`
- `scripts/linux/lency_selfhost_run.sh`
- `scripts/win/lency_selfhost_build.ps1`
- `scripts/win/lency_selfhost_run.ps1`

## 测试与维护脚本

- `scripts/*/run_lcy_tests.*`：Rust 母体 `.lcy` 集成测试。
- `scripts/check_lencyc_meta.py`：selfhost 源码结构和命名检查。
- `scripts/check_file_size.py`、`scripts/check_dir_density.py`：规模预警。
- `scripts/check_todos.py`：TODO/FIXME 扫描。
- `scripts/check_banned_patterns.py`：禁用模式检查。
- `scripts/check_commit_messages.py`：Conventional Commit 格式检查。

测试目录分工见[自举路线](../development/bootstrap.md)。
