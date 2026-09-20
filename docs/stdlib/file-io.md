# 文件与路径

底层 intrinsic：

| 函数 | 用途 |
|---|---|
| `read_file(path)` | 读取完整文件 |
| `write_file(path, content)` | 写入文件 |
| `file_exists(path)` | 判断路径是否存在 |
| `is_dir(path)` | 判断路径是否为目录 |

`lib/std/fs.lcy` 提供 `read_to_string` 和 `write_string` 包装。读取和写入使用当前 `Result` 风格返回类型。

`file_exists` 与 `is_dir` 已有 selfhost runtime 冒烟回归；更完整的路径和 I/O 包装仍以 Rust 母体或标准库源码为主，尚未全部形成 selfhost 端到端保证。
