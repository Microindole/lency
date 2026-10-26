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

## Rust 母体边界

可空结构体经过非空判断后可以读写字段，空值合并会按需计算默认结构体。非空值参与空值合并不会额外执行默认表达式。

母体的可空整数、浮点数和布尔值具有一致的存储、参数、返回值与字段表示。零与假都是有效值，不等于没有值；非空检查后可以使用基础值，空值合并仅在左侧为空时执行默认表达式。

```lency
int? keep(int? value) {
    return value
}

int main() {
    int? missing = null
    int? zero = keep(0)
    if (missing ?? 9) != 9 { return 1 }
    return zero ?? 9
}
```

安全导航读取标量字段时，空接收者得到没有值，非空接收者保留字段的真实值。可空标量之间的相等比较检查值和空状态，不比较内部值单元的地址。不兼容的基础类型不能仅因可空标记就获得比较能力。同名局部变量也不会继承外层绑定的非空细化。

这些规则不提供错误包装、错误传播或异常捕获。编译器内部的值表示不改变类型在前的函数签名，也不要求源码手动构造包装。

## selfhost 状态

- lexer/parser 支持 `null` 和 `T?`。
- resolver 已对基础类型和自定义类型的 nullable 签名做最小一致性检查。
- literal `null` 可参与当前 match 子集。
- 安全导航、空值合并和完整流敏感智能转型尚未作为稳定 selfhost 能力记录，不应在依赖它们前只参考 Rust 母体行为。

对应 parser 回归位于 `tests/example/parser/custom_nullable_*.lcy`。
