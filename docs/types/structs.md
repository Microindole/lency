# 结构体

## 声明、构造与字段

结构体成员以类型在前声明。构造值时指定字段名和值；字段访问和赋值使用点号。

```lency
struct Pair {
    int left
    int right
}

int main() {
    var pair = Pair { left: 1, right: 2 }
    pair.right = 3
    return pair.left + pair.right - 4
}
```

构造时不能省略必需字段，也不能提供未知字段。字段初始值及后续赋值需要符合声明类型。

## 方法与接收者

实现块为对应类型定义方法，方法体通过接收者访问字段。方法签名沿用类型在前的函数形式。

```lency
struct Pair {
    int left
    int right
}

impl Pair {
    int sum() {
        return this.left + this.right
    }
}

int main() {
    var pair = Pair { left: 1, right: 2 }
    return pair.sum() - 3
}
```

方法按接收者类型解析。其他结构体的同名方法不会使当前类型获得该方法，方法也不会因此变成同名全局函数。方法参数与返回路径遵循[函数](../basics/functions.md)的规则。

## 自举边界

非泛型结构体已有字段读写、普通函数参数和返回值、方法调用以及方法返回结构体的端到端回归。自举 resolver 同时检查本地与导入模块的方法签名。

泛型结构体实例化、泛型实现块和完整 trait 体系尚不属于稳定自举运行子集。当前能够解析对应声明，不等于可以端到端构建。运行回归位于 tests/example/runtime 中的结构体用例。
