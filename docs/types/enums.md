# 枚举

## 当前可用语法（自举链路）

```lency
enum Status {
    Idle,
    Running,
    Done
}

var s = Running()
```

## 带 payload 的 variant 构造

```lency
enum Message {
    Quit,
    Text(string),
    Pair(int, string)
}

var m = Text("hello")
var p = Pair(1, "ok")
```

## 模式匹配

```lency
var s = Running()

var code = match (s) {
    Idle => 0,
    Running => 1,
    Done => 2
}

// 赋值链作为 match 目标也会触发 enum 语义检查
var code2 = match (s = Running()) {
    Idle => 0,
    Running => 1,
    Done => 2
}
```

## payload 绑定匹配

```lency
enum Message {
    Quit,
    Text(string),
    Pair(int, string)
}

var m = Pair(1, "x")
var code = match (m) {
    Quit => 0,
    Text(s) => 1,
    Pair(a, b) => a
}
```

## 嵌套 payload 模式

```lency
enum Payload {
    Num(int),
    Text(string)
}

enum Message {
    Quit,
    Wrap(Payload)
}

var m = Wrap(Text("x"))
var code = match (m) {
    Quit => 0,
    Wrap(Num(v)) => v,
    Wrap(Text(msg)) => 1
}
```

## 当前语义检查（自举链路）

- `match` 在目标可推断为 enum 时，检查：
  - 重复 pattern（如 `Idle` 写两次）
  - 未知 variant（如 `Paused` 不在 `Status` 内）
  - 穷尽性（无 `_` 且漏分支时报错）
  - payload binder arity（如 `Pair(a)` 对 `Pair(int, string)` 报错）
  - payload binder 类型传播（binder 会在对应 arm 内按 payload 类型参与表达式检查）
  - 赋值链目标（如 `match (s = make_status())`）同样执行未知 variant/穷尽性校验
  - 嵌套 payload 模式（如 `Wrap(Text(msg))`）的 variant 存在性与 arity
- enum variant 构造调用检查：
  - 参数个数（arity）一致
  - 参数类型一致（payload 类型）

> TODO: `match` 的嵌套/复杂模式（例如更深层结构解构）尚未接入。
> FIXME: 自举链路仍存在 `TYPE_UNKNOWN` 兼容路径，复杂组合场景可能把类型错误降级为弱诊断。
