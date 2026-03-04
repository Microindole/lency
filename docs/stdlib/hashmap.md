# HashMap

当前 HashMap 主要通过 runtime 提供的函数接口使用。

## int 键 HashMap

```lency
var map = hashmap_int_new()
hashmap_int_insert(map, 1, 10)
var has = hashmap_int_contains(map, 1)
var v = hashmap_int_get(map, 1)
var n = hashmap_int_len(map)
```

## string 键 HashMap

```lency
var map = hashmap_string_new()
hashmap_string_insert(map, "k", 42)
var has = hashmap_string_contains(map, "k")
var v = hashmap_string_get(map, "k")
var n = hashmap_string_len(map)
```

## 当前边界

- 现阶段推荐直接使用上述函数接口。
- 结构体封装版本仍受 codegen 路径限制，相关待办保留在标准库实现中。
