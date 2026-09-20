# 枚举与匹配

## 声明与构造

```lency
enum Message {
    Quit,
    Text(string),
    Pair(int, string)
}

var message = Pair(1, "ok")
```

## 模式匹配

```lency
var code = match (message) {
    Quit => 0,
    Text(value) => 1,
    Pair(number, text) => number
}
```

支持 wildcard、嵌套 payload 和 guard 的最小形式：

```lency
var code = match (message) {
    Text(value) if (value == "ok") => 1,
    Pair(number, _) => number,
    _ => 0
}
```

## selfhost 状态

前端已覆盖构造器 arity/type、未知 variant、重复 pattern、穷尽性、payload binder、嵌套模式和 guard 条件类型检查。

端到端 lowering 已覆盖 literal、wildcard、guard 和递归 enum payload 的现有回归。复杂组合仍应先添加 `tests/example/runtime/` 用例，不应仅依据 parser/sema 通过就宣称可运行。
