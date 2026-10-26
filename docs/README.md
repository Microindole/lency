# Lency 文档

Lency 是一门强调简洁、显式和静态类型的编译语言。设计方向以[设计原则](./design.md)为准，当前自举阶段和阻塞项见[自举状态](./development/status.md)。

## 从这里开始

普通程序示例使用 Rust 母体编译器。请按以下顺序阅读：

1. [基础语法](./basics/syntax.md)：源码文件、入口、花括号块、分号和注释。
2. [变量与类型](./basics/variables.md)：显式声明、静态推导、常量、可空值和集合。
3. [表达式](./basics/expressions.md)：字面量、运算、调用、成员与索引。
4. [函数](./basics/functions.md)：签名、返回路径、泛型、外部函数与业务错误结果。
5. [控制流](./basics/control-flow.md)：条件、循环、跳转和匹配。
6. [模块](./basics/modules.md)：源码导入与两套前端的差异。

示例会标明自举专用写法；不要将语法可解析等同于自举后端已可运行。实现与设计不一致时，应确认偏差来源，不能直接将偏差写成语言规则。

## 类型与标准库

- 类型：[基础类型](./types/primitives.md)、[结构体](./types/structs.md)、[枚举与匹配](./types/enums.md)、[可空类型](./types/null-safety.md)。
- 标准库：[Vec](./stdlib/vec.md)、[字符串](./stdlib/string.md)、[文件 I/O](./stdlib/file-io.md)、[HashMap](./stdlib/hashmap.md)。

## 项目与开发

- [设计原则](./design.md)
- [实现分层](./development/implementations.md)
- [自举路线](./development/bootstrap.md)
- [当前状态](./development/status.md)
- [贡献与维护规则](./development/contributing.md)
- [工具与脚本](./tools/scripts.md)
- [持续集成](./development/ci.md)

Rust 母体位于 crates，提供普通程序编译入口和自举支撑，目前保持固定。Lency 自举编译器位于 lencyc，是当前开发主线。

普通程序的命令入口：

```powershell
cargo run --bin lencyc -- check <input.lcy>
cargo run --bin lencyc -- run <input.lcy>
cargo run --bin lencyc -- build <input.lcy> -o <name>
```

开发检查入口：

```powershell
cargo run -p xtask -- auto-check
```

阶段收敛检查：

```powershell
cargo run -p xtask -- bootstrap-check
```

环境需要 Rust、LLVM 15 和平台链接器，配置及自举开发命令见工具文档。阶段构建当前仍依赖 Rust LIR 后端与 runtime。
