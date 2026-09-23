# 变量与类型

局部变量使用 `var` 从初始值推断类型；需要显式类型时采用与 C/C++ 一致的类型在前写法：

```lency
var count = 10
string name = "Lency"
bool active = true
```

常量使用 `const`：

```lency
const MAX_RETRY = 3
```

基础类型见[基础类型](../types/primitives.md)，可空类型见[可空类型](../types/null-safety.md)。

## 实现说明

- Rust 母体支持完整变量与常量编译链路。
- selfhost 已支持 `var`、`const` 以及类型在前的显式局部变量，并会保留显式类型供 resolver 与 LIR lowering 使用。空 `vec[]` 无法推断元素类型时必须写成 `Vec<T> values = vec[]`。
- 是否能够端到端运行，还取决于具体值类型和 selfhost LIR lowering；以 `tests/example/runtime/` 为准。
