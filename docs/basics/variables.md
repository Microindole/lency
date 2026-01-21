# 变量与类型

## 变量声明

```lency
var x = 10           // 类型推导为 int
var name = "Alice"   // 类型推导为 string
var flag = true      // 类型推导为 bool
```

### 显式类型注解

```lency
int count = 0
string message = "hello"
bool active = false
```

## 基本类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `int` | 64位整数 | `42`, `-100` |
| `float` | 64位浮点数 | `3.14`, `-0.5` |
| `bool` | 布尔值 | `true`, `false` |
| `string` | 字符串 | `"hello"` |
| `void` | 无返回值 | 函数返回类型 |

## 可空类型

默认所有类型都是非空的。使用 `?` 表示可空：

```lency
int? maybe_number = null
string? name = null

// 检查非空
if name != null {
    print(name)  // 智能转型为 string
}
```

## 数组与向量

```lency
// 固定长度数组
int[3] arr = [1, 2, 3]

// 动态向量
var numbers = vec![1, 2, 3]
numbers.push(4)
print(numbers.len())  // 4
```
