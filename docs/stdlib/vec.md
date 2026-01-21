# Vec 动态数组

## 创建

```lency
// 字面量创建
var numbers = vec![1, 2, 3]

// 空向量（需要类型注解）
Vec<int> empty = vec![]
```

## 方法

| 方法 | 描述 |
|------|------|
| `len()` | 返回元素数量 |
| `push(item)` | 添加元素到末尾 |
| `get(index)` | 获取指定索引的元素 |
| `set(index, value)` | 设置指定索引的值 |

## 示例

```lency
int main() {
    var v = vec![1, 2, 3]
    
    print(v.len())      // 3
    print(v.get(0))     // 1
    
    v.push(4)
    v.set(0, 10)
    
    print(v.get(0))     // 10
    print(v.len())      // 4
    
    return 0
}
```

## 遍历

```lency
import std.collections

var items = vec![1, 2, 3, 4, 5]
var iter = vec_iter(items)
var sum = 0

var opt = iter.next()
while opt != null {
    match opt {
        case Some(value) => sum = sum + value
        case None => break
    }
    opt = iter.next()
}
print(sum)  // 15
```

## 泛型

Vec 支持任意类型：

```lency
Vec<string> names = vec!["Alice", "Bob"]
Vec<Vec<int>> matrix = vec![vec![1, 2], vec![3, 4]]
```
