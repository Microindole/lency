# 控制流

## if 语句

```lency
if condition {
    // 条件为 true 时执行
}

if x > 0 {
    print("positive")
} else if x < 0 {
    print("negative")
} else {
    print("zero")
}
```

## while 循环

```lency
var i = 0
while i < 10 {
    print(i)
    i = i + 1
}
```

## for 循环

```lency
// C 风格 for
for var i = 0; i < 10; i = i + 1 {
    print(i)
}

// for-in 遍历
var items = vec![1, 2, 3]
for item in items {
    print(item)
}
```

## match 表达式

```lency
enum Color {
    Red,
    Green,
    Blue
}

var c = Color.Red
var name = match c {
    case Red => "红色"
    case Green => "绿色"
    case Blue => "蓝色"
}
```

### 带数据的枚举匹配

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

## 循环控制

```lency
while true {
    if done {
        break     // 退出循环
    }
    if skip {
        continue  // 跳过本次迭代
    }
}
```
