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

字符访问包装遵循相同边界规则，越界不会返回负一。大小写转换只处理 ASCII，反转按字节操作，不提供 Unicode 字素反转语义。

这些名称在源码和 AST 中都是普通函数调用；Rust 母体只在集中 lowering 边界把对应全局函数符号映射到 runtime。

Rust 母体对这些接口的覆盖更完整。selfhost 当前只保证已有自举和 runtime 回归实际经过的调用路径；新增字符串能力时应同时检查 LIR intrinsic 映射和 runtime ABI。

## 内存归属

内置查询借用输入，不销毁字符串。内置裁剪、截取、连接、格式化和字符转换返回新分配字符串，调用者负责释放；拆分结果需要分别释放子串和容器。字符串字面量不可释放，详见[手动内存管理](../basics/memory.md)。

标准字符串模块中返回字符串的函数始终返回独立分配，包括空结果、重复次数为零、未找到替换目标和无需填充的分支。调用者释放返回值，输入不被销毁。布尔与位置查询在内部释放其临时子串。普通与优化构建的重复调用回归会核对运行时分配余额。
