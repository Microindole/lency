# 文件 I/O

## 推荐接口（std.fs）

当前建议优先使用 `std.fs` 提供的包装函数：

| 函数 | 签名 | 描述 |
|------|------|------|
| `read_to_string` | `string! read_to_string(string path)` | 读取文件全部内容 |
| `write_string` | `void! write_string(string path, string content)` | 写入字符串到文件 |

> 说明：底层 intrinsic 仍是 `read_file` / `write_file`，`std.fs` 对其做了语义包装。

## 基础示例

```lency
import std.fs

int main() {
    var read_res = read_to_string("data.txt")
    match read_res {
        case Result.Ok(content) => print(content)
        case Result.Err(_) => print("read failed")
    }

    var write_res = write_string("output.txt", "Hello, Lency!")
    match write_res {
        case Result.Ok(_) => print("write ok")
        case Result.Err(_) => print("write failed")
    }

    return 0
}
```

## 复制文件

```lency
import std.fs

int main() {
    var src = read_to_string("source.txt")
    match src {
        case Result.Ok(content) => {
            var w = write_string("dest.txt", content)
            match w {
                case Result.Ok(_) => return 0
                case Result.Err(_) => return 2
            }
        }
        case Result.Err(_) => return 1
    }
}
```

## 当前边界

`file_exists` / `is_dir` 已有最小 runtime 映射，但 `std.fs` 中更完整的路径能力仍在持续补齐。
