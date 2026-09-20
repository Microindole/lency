# HashMap

当前稳定入口主要是 runtime 函数，而不是结构体包装：

```lency
var map = hashmap_int_new()
hashmap_int_insert(map, 1, 10)
var exists = hashmap_int_contains(map, 1)
var value = hashmap_int_get(map, 1)
```

字符串键使用对应的 `hashmap_string_*` 函数。

Rust 母体有相关集成测试。selfhost 仍受 generic struct、impl method 和 member lowering 限制，因此不要把标准库源码中存在的包装类型视为已具备端到端支持。
