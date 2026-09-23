# 字符串

常用 intrinsic 包括：

| 函数 | 用途 |
|---|---|
| `len(value)` | 字符串长度 |
| `trim(value)` | 去除首尾空白 |
| `substr(value, start, length)` | 截取子串 |
| `split(value, delimiter)` | 拆分字符串 |
| `join(values, separator)` | 连接字符串 |
| `char_to_string(code)` | 字符码转字符串 |

`lib/std/str.lcy` 在这些基础能力上实现 `starts_with`、`ends_with`、`contains`、`replace_*`、大小写转换等函数。

字符串索引按 UTF-8 字节偏移返回 `int`。负索引或超出字节长度属于程序错误，Rust 母体会立即 panic，不返回哨兵值。

这些名称在源码和 AST 中都是普通函数调用；Rust 母体只在集中 lowering 边界把对应全局函数符号映射到 runtime。

Rust 母体对这些接口的覆盖更完整。selfhost 当前只保证已有自举和 runtime 回归实际经过的调用路径；新增字符串能力时应同时检查 LIR intrinsic 映射和 runtime ABI。
