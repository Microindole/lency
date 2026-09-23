# 持续集成

CI 以自举链路为主门禁，而不是只验证 Rust workspace 能否通过单元测试。

## 平台矩阵

| 平台 | 架构 | Runner | 验证范围 |
|---|---|---|---|
| Linux | x86_64 | `ubuntu-24.04` | Rust 全量检查、selfhost、LIR、runtime |
| Linux | ARM64 | `ubuntu-24.04-arm` | selfhost、LIR、runtime |
| Windows | x86_64 | `windows-2025` | Rust 单元测试、selfhost、LIR、runtime |
| macOS | ARM64 | `macos-15` | Rust 单元测试、selfhost、LIR、runtime |

Windows ARM64 暂不属于支持矩阵。Windows 工具链依赖项目发布的
`llvm-15.0.7-custom-windows-x86_64-msvc-x86only.zip`；在提供并验证 ARM64 LLVM
工具链之前，不创建只验证 Rust 表面编译的 Windows ARM64 job。

## 工作流职责

- `tests.yml`：按变更范围运行 Rust 门禁和多平台 Lency 端到端门禁。
- `bootstrap.yml`：验证 stage2/stage3 LIR 收敛；影响编译器、自举源码、runtime 或测试的 PR 必须运行。
- `lints.yml`：格式、Clippy、Rustdoc 和仓库静态规则。
- `workflow-lint.yml`：GitHub Actions、shell 和 PowerShell 语法检查。
- `audit.yml`：Cargo 依赖审计，按依赖变更和每周计划运行。
- `release.yml`：构建 Linux x86_64/ARM64、Windows x86_64、macOS ARM64 包，并对解包产物执行 build/run smoke test。

## LLVM 约束

Linux runner 固定为 Ubuntu 24.04，避免 `ubuntu-latest` 迁移导致 LLVM 15 包突然不可用。
macOS 使用 Homebrew `llvm@15` 的动态 prefix。Windows 使用仓库内 composite action，按固定
SHA-256 校验并缓存定制 LLVM 归档。

## 本地对应命令

```powershell
cargo run -p xtask -- check-rust
cargo run -p xtask -- check-lency
cargo run -p xtask -- bootstrap-check
```

`cargo run -p xtask -- auto-check` 的 scope 规则应与 CI 保持一致。Rust host 或 `xtask`
变化同时影响 Rust 与 Lency 自举链路，因此会运行两组检查。
