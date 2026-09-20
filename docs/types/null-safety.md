# 可空类型

默认类型为非空；`T?` 明确表示值可以为 `null`：

```lency
int value = 42
int? maybe_value = null

struct Boxed {
    int id
}

Boxed? keep(Boxed? value) {
    return value
}
```

## selfhost 状态

- lexer/parser 支持 `null` 和 `T?`。
- resolver 已对基础类型和自定义类型的 nullable 签名做最小一致性检查。
- literal `null` 可参与当前 match 子集。
- 安全导航、空值合并和完整流敏感智能转型尚未作为稳定 selfhost 能力记录，不应在依赖它们前只参考 Rust 母体行为。

对应 parser 回归位于 `tests/example/parser/custom_nullable_*.lcy`。
