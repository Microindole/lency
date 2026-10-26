# 变量与类型

## 声明与静态推导

显式声明将类型放在名称前。推导声明根据初始值确定类型；它不会使变量变成可以随意改变类型的动态变量。两种形式都需要初始值。

```lency
int main() {
    int count = 10
    string name = "Lency"
    bool active = true
    var next = count + 1
    count = next
    print(name)
    return count - 11
}
```

赋值可以改变值，不能任意改变变量类型。例如整数变量不能在下一条赋值中接收字符串。作用域见[基础语法](./syntax.md)。

不采用名称后跟冒号类型的局部变量声明形式。以下仅为不支持的写法：

```text
var count: int = 10
```

## 常量声明的实现差异

自举前端支持常量声明，初始值决定类型，并检查后续重新赋值。

自举前端示例：

```lency-selfhost
int main() {
    const MAX_RETRY = 3
    return MAX_RETRY - 3
}
```

Rust 母体当前具有常量词法标记，但语句解析器没有对应声明分支；不能宣称两套实现均已支持常量编译。该差异不改变常量的语言含义。

## 可空值

普通类型不表示可空。需要表达“可能没有值”时，在类型后加问号，并显式声明可以接收空值的变量。

```lency
int main() {
    int? maybe_count = null
    return 0
}
```

类型后的问号用于构造可空类型，不是错误传播运算符。可空类型的访问和检查边界见[可空类型](../types/null-safety.md)。

## 动态集合与固定数组

动态集合具有元素类型，可以追加元素。空字面量本身不能提供具体元素类型，使用显式类型声明。

```lency
int main() {
    var values = vec[1, 2, 3]
    Vec<int> pending = vec[]
    pending.push(values[0])
    return pending[0] - 1
}
```

固定数组在类型中声明长度，字面量不带动态集合前缀。

```lency
int main() {
    int[3] values = [1, 2, 3]
    return values[0] - 1
}
```

固定数组与动态集合不是同一个类型。数组示例属于母体能力，不能据此推断自举后端也已支持。动态集合的元素检查与自举边界见[Vec](../stdlib/vec.md)。

## 自定义类型

结构体字面量通过字段名指定初始值；声明中的类型和字面量字段必须匹配。

```lency
struct Pair {
    int left
    int right
}

int main() {
    Pair pair = Pair { left: 1, right: 2 }
    pair.right = 3
    return pair.left + pair.right - 4
}
```

完整规则见[结构体](../types/structs.md)和[枚举与匹配](../types/enums.md)。
