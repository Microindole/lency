# 枚举

## 基本枚举

```lency
enum Color {
    Red,
    Green,
    Blue
}

var c = Color.Red
```

## 带数据的枚举

```lency
enum Shape {
    Circle(int),      // 半径
    Rectangle(int, int)  // 宽, 高
}

var s = Shape.Circle(10)
```

## 模式匹配

```lency
var color = Color.Red

match color {
    case Red => print("red")
    case Green => print("green")
    case Blue => print("blue")
}
```

### 提取数据

```lency
var shape = Shape.Rectangle(10, 20)

match shape {
    case Circle(r) => print(r)
    case Rectangle(w, h) => print(w * h)
}
```

## 泛型枚举

```lency
enum Option<T> {
    Some(T),
    None
}

var opt = Option::<int>.Some(42)

match opt {
    case Some(value) => print(value)
    case None => print("nothing")
}
```

## 标准库枚举

### Option<T>

```lency
import std.core

// 表示可能不存在的值
Option<int> find(Vec<int> v, int target) {
    // ... 返回 Some(index) 或 None
}
```

### Result<T, E>

```lency
import std.core

// 表示可能失败的操作
Result<int, Error> parse(string s) {
    // ... 返回 Ok(value) 或 Err(error)
}
```
