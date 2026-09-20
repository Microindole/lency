# 控制流

## 条件与循环

```lency
if value > 0 {
    print("positive")
} else {
    print("not positive")
}

var index = 0
while index < 10 {
    index = index + 1
}
```

支持 `break`、`continue` 和 `return`。selfhost parser 也能解析 `for`，内部按当前实现降为循环结构；端到端使用前应有对应 runtime 回归。

## match

```lency
enum Status {
    Idle,
    Running,
    Done
}

var status = Running()
var code = match (status) {
    Idle => 0,
    Running => 1,
    Done => 2
}
```

payload、嵌套模式和 guard 见[枚举与匹配](../types/enums.md)。
