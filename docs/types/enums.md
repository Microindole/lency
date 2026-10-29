# 枚举与匹配

枚举描述一组明确的情况。一个枚举项可以没有载荷，也可以携带一个或多个指定类型的值；它们不是三个独立类型。

## 母体写法

母体通过类型名限定枚举项，匹配分支带分支关键字。载荷模式中的名称在对应分支绑定载荷值。

```lency
enum Message {
    Quit,
    Text(string),
    Pair(int, string)
}

int main() {
    var message = Message.Pair(2, "ok")
    var code = match message {
        case Message.Quit => 0,
        case Message.Text(value) => 1,
        case Message.Pair(number, text) => number
    }
    return code - 2
}
```

无载荷项直接构造枚举值，有载荷项通过参数提供数据；调用数量和类型必须与声明一致。

## 自举前端写法

当前自举前端将枚举项登记为可调用构造器，匹配分支不写母体的分支关键字。

```lency-selfhost
enum Message {
    Quit,
    Text(string),
    Pair(int, string)
}

int main() {
    var message = Pair(2, "ok")
    var code = match (message) {
        Quit => 0,
        Text(value) => 1,
        Pair(number, text) => number
    }
    return code - 2
}
```

无载荷项也可按空参数构造器调用。自举前端检查载荷类型、构造参数数量、重复模式、未知枚举项与穷尽性，已有递归载荷匹配的运行回归。

自举守卫示例：

```lency-selfhost
enum Message {
    Quit,
    Text(string)
}

int main() {
    var message = Text("ok")
    var code = match (message) {
        Text(value) if (value == "ok") => 1,
        _ => 0
    }
    return code - 1
}
```

守卫条件必须为非可空布尔值，括号不可省略。带守卫的分支不能独自保证穷尽覆盖。

两套前端在构造与分支形式上的差异属于待对齐的实现边界，不据此修改语言设计。母体当前的模式解析器没有自举守卫的对应入口。

## 作为普通业务结果

枚举可以表达可恢复业务结果，调用者通过普通匹配处理；编译器不会因为某个类型或枚举项的名称而自动提供异常、传播或捕获功能。

返回业务结果的完整例子见[函数](../basics/functions.md)，普通匹配入门见[控制流](../basics/control-flow.md)。
