# 变量与类型

局部变量使用 `var` 声明，可由初始值推断类型，也可显式标注类型：

```lency
var count = 10
var name: string = "Lency"
var active: bool = true
```

常量使用 `const`：

```lency
const MAX_RETRY = 3
```

基础类型见[基础类型](../types/primitives.md)，可空类型见[可空类型](../types/null-safety.md)。

## 实现说明

- Rust 母体支持完整变量与常量编译链路。
- selfhost 已支持 `var`、`const` 的解析和基础语义约束。
- 是否能够端到端运行，还取决于具体值类型和 selfhost LIR lowering；以 `tests/example/runtime/` 为准。
