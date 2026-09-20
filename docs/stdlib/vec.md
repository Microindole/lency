# Vec

`Vec<T>` 是编译器内建集合类型，基础操作由 runtime 提供：

```lency
var values = vec![1, 2, 3]
values.push(4)
var size = values.len()
var first = values.get(0)
values.set(0, 10)
```

`lib/std/collections.lcy` 还定义了 `reverse`、`range`、查找和整数聚合等辅助函数。

## 实现边界

- Rust 母体已有较完整的 Vec 与泛型回归。
- selfhost 编译器内部会使用 vec-backed 数据结构，但这不等于任意 `Vec<T>` 程序都已能由 selfhost 端到端生成。
- generic struct 包装和 impl 方法仍会阻塞标准库中部分抽象；推进时应以具体 runtime 用例为准。
