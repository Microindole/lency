# 字符串操作

## 内置函数

| 函数 | 签名 | 描述 |
|------|------|------|
| `len` | `int len(string s)` | 返回字符串长度 |
| `trim` | `string trim(string s)` | 去除首尾空白 |
| `split` | `Vec<string> split(string s, string delim)` | 按分隔符拆分 |
| `join` | `string join(Vec<string> parts, string sep)` | 用分隔符连接 |
| `substr` | `string substr(string s, int start, int len)` | 提取子串 |
| `char_to_string` | `string char_to_string(int code)` | 字符码转字符串 |

## 示例

```lency
import std.string

int main() {
    var s = "  hello world  "
    
    // 基本操作
    print(len(s))           // 15
    print(trim(s))          // "hello world"
    
    // 拆分与连接
    var parts = split("a,b,c", ",")  // ["a", "b", "c"]
    var joined = join(parts, "-")     // "a-b-c"
    
    // 子串
    var sub = substr("hello", 0, 2)   // "he"
    
    // 字符操作
    var c = s[0]                      // 获取字符码 (int)
    var ch = char_to_string(65)       // "A"
    
    return 0
}
```

## 字符串辅助函数

`lib/std/string.lcy` 提供更多函数：

```lency
import std.string

// 检查
bool is_empty(string s)
bool starts_with(string s, string prefix)
bool ends_with(string s, string suffix)
bool contains(string s, string sub)

// 转换
string to_upper(string s)   // "hello" -> "HELLO"
string to_lower(string s)   // "HELLO" -> "hello"
string reverse(string s)    // "abc" -> "cba"

// 填充
string pad_left(string s, int len, string pad)
string pad_right(string s, int len, string pad)

// 替换
string replace_first(string s, string old, string new_str)
string replace_all(string s, string old, string new_str)

// 查找
int index_of(string s, string sub)  // 返回 -1 表示未找到
```
