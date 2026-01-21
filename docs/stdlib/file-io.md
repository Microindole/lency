# 文件 I/O

## 内置函数

| 函数 | 签名 | 描述 |
|------|------|------|
| `read_file` | `string read_file(string path)` | 读取文件内容 |
| `write_file` | `void write_file(string path, string content)` | 写入文件 |

## 示例

### 读取文件

```lency
int main() {
    var content = read_file("data.txt")
    print(content)
    return 0
}
```

### 写入文件

```lency
int main() {
    write_file("output.txt", "Hello, World!")
    return 0
}
```

### 复制文件

```lency
int main() {
    var content = read_file("source.txt")
    write_file("dest.txt", content)
    return 0
}
```

## 处理多行文本

```lency
import std.string

int main() {
    var content = read_file("data.csv")
    var lines = split(content, "\n")
    
    for line in lines {
        if len(trim(line)) > 0 {
            print(line)
        }
    }
    
    return 0
}
```
