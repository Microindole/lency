# Lency 文档

Lency 是一门强调简洁、显式和静态类型的编译语言。仓库同时包含两套实现：

- `crates/`：端到端可用的 Rust 母体编译器，负责提供冻结的 stage0 和参考行为；冻结不表示继续扩展全部语言能力。
- `lencyc/`：使用 Lency 编写的自举编译器，是当前开发主线。

当前目标不是继续扩展 Rust 母体，而是先让 Lency 自举编译器以最小语言子集稳定编译自身。实现状态与规划以[自举状态](./development/status.md)为准。

## 文档导航

### 项目与开发

- [设计原则](./design.md)
- [实现分层与成熟度](./development/implementations.md)
- [自举路线](./development/bootstrap.md)
- [当前状态](./development/status.md)
- [贡献与维护规则](./development/contributing.md)
- [工具与脚本](./tools/scripts.md)

### 语言

- 基础：[变量与类型](./basics/variables.md)、[函数](./basics/functions.md)、[控制流](./basics/control-flow.md)
- 类型：[基础类型](./types/primitives.md)、[结构体](./types/structs.md)、[枚举与匹配](./types/enums.md)、[可空类型](./types/null-safety.md)
- 标准库：[Vec](./stdlib/vec.md)、[字符串](./stdlib/string.md)、[文件 I/O](./stdlib/file-io.md)、[HashMap](./stdlib/hashmap.md)

语言文档以 Rust 母体的已实现行为作为当前参考语义，同时会明确标注 Lency 自举链路尚未支持的部分。不能仅凭 Rust 母体能够编译某段程序，就认定自举编译器也已支持。

在 selfhost 成为独立可用编译器之前，普通 Lency 程序默认使用 Rust 母体的 `lencyc`：

```powershell
cargo run --bin lencyc -- check <input.lcy>
cargo run --bin lencyc -- run <input.lcy>
cargo run --bin lencyc -- build <input.lcy> -o <name>
```

`xtask selfhost-*` 是自举开发和验证入口，不是当前默认的用户编译器入口。

## 最小验证

环境需要 Rust、MSVC/系统链接器和 LLVM 15。配置方法见[工具与脚本](./tools/scripts.md)。

```powershell
cargo build --release -p lency_cli -p lency_runtime
cargo run -p xtask -- check-lency
cargo run -p xtask -- bootstrap-check
```

涉及 Rust 母体修改时再额外运行：

```powershell
cargo run -p xtask -- check-rust
```
