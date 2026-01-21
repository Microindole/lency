# Null 安全

Lency 默认所有类型都是非空的。使用 `?` 后缀表示可空类型。

## 可空类型声明

```lency
int value = 42          // 非空，不能赋值 null
int? maybe = null       // 可空，可以赋值 null
int? also = 42          // 可空，也可以有值
```

## 安全操作符

### 安全导航 `?.`

```lency
struct User {
    string? name
}

var user = get_user()
var len = user?.name?.len()  // 如果任一为 null，返回 null
```

### Elvis 操作符 `??`

```lency
string? name = null
var display = name ?? "Anonymous"  // 如果 null，使用默认值
```

## 智能转型

在 `if` 条件检查后，编译器自动将可空类型转为非空：

```lency
int? x = get_value()

if x != null {
    // 这里 x 自动转型为 int（非空）
    print(x + 1)
}
```

## 类型兼容性

```lency
int a = 10
int? b = a      // ✅ 非空可以赋值给可空

int? c = null
int d = c       // ❌ 编译错误：可空不能直接赋值给非空
```
