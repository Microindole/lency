# Lency 语言设计规范 (Ver 2.0)

> **更新**: 2026-01-16  
> **状态**: 开发中 (65% 完成)

## 1. 核心哲学 (Philosophy)

Lency 是一门 **"实用主义的工业级语言"**。它的设计目标是在 C 语言的结构感与 Python 的开发效率之间找到黄金平衡点。

**四大支柱**:
- **Crystal Clear (清晰如晶)**: 代码意图一目了然。拒绝隐式转换，拒绝复杂的元编程魔法。
- **Safety by Default (默认安全)**: 所有的引用默认不可为空 (Non-nullable)。空值必须显式处理。
- **Structure over Style (结构至上)**: 采用 C 系的大括号 `{}` 结构，但在语句末尾摒弃分号 `;` (除非一行多句)，减少视觉噪音。
- **Simplicity First (简洁优先)**: 组合优于继承，显式优于隐式。

---

## 2. 基础语法 (Syntax)

### 2.1 变量与常量 ✅

采用 `var` 进行类型推导，支持显式类型标注。

```lency
// 自动推导为 int
var count = 10 

// 显式类型
var name: string = "Lency"

// 常量（规划中）
const PI = 3.14159
```

**实现状态**: ✅ 完成

### 2.2 函数 (Functions) ✅

抛弃 `func/fn` 关键字，回归 C 系的直观。

```lency
// 返回值类型写在前面
int add(int a, int b) {
    return a + b
}

// 无返回值
void log(string msg) {
    print(msg)
}

// 泛型函数
T max<T>(T a, T b) {
    if a > b {
        return a
    }
    return b
}
```

**实现状态**: ✅ 完成

### 2.3 控制流 (Control Flow) ✅

没有括号包裹条件，强制使用大括号。

```lency
if x > 10 {
    print("Large")
} else {
    print("Small")
}

while x > 0 {
    x = x - 1
}

for i in 0..10 {
    print(i)
}

// Match 表达式
match status {
    200 => print("OK"),
    404 => print("Not Found"),
    _   => print("Unknown")
}
```

**实现状态**: ✅ 完成 (for-range 部分完成)

---

## 3. 类型系统 (Type System)

### 3.1 空安全 (Null Safety) ✅

这是 Lency 最核心的特性之一。

```lency
string s = "Hello" // 永远不可能是 null

string? maybe = null // 显式可空

// 安全访问
if maybe != null {
    print(maybe.length) // 智能转换
}

// Elvis 操作符
var len = maybe?.length  // 返回 int?
var len2 = maybe?.length ?? 0  // 提供默认值
```

**实现状态**: ✅ 完成（智能类型转换、Elvis、空值合并）

### 3.2 结构体与泛型 ✅

采用单态化泛型 (Monomorphization)，零运行时开销。

```lency
struct Box<T> {
    T value
}

impl<T> Box<T> {
    T get() {
        return this.value
    }
    
    void set(T v) {
        this.value = v
    }
}

var intBox = Box<int> { value: 10 }
var val = intBox.get()
```

**实现状态**: ✅ 完成（泛型 struct、impl、方法调用）

### 3.3 枚举与模式匹配 ✅

```lency
enum Status {
    Ok,
    Error,
    Pending
}

// 泛型枚举（部分支持）
enum Option<T> {
    Some(T),
    None
}

match opt {
    Some(val) => print(val),
    None => print("empty")
}
```

**实现状态**: ✅ 基础枚举，⚠️ 泛型枚举有限制

### 3.4 Trait 系统 ✅

```lency
trait Hash {
    int hash()
}

impl Hash for int {
    int hash() {
        return this
    }
}

trait Comparable<T> {
    bool greater_than(T other)
}
```

**实现状态**: ✅ 完成

---

## 4. 错误处理 (Error Handling)

拒绝 Try-Catch 这种破坏控制流的机制。使用 Result 模式。

```lency
// ! 表示可能出错
string! read_file(string path) {
    // ...
}

var result = read_file("data.txt")
// 需要手动处理错误（当前实现）
```

**实现状态**: ⚠️ 语法支持，Result 类型待完善

---

## 5. 内存管理 (Memory)

### 5.1 内存模型 ⚠️

- **当前**: 手动管理 + LLVM 优化
- **计划**: Boehm GC 或引用计数
- **未来**: 所有权系统（学习 Rust）

**实现状态**: ⚠️ 基础实现，GC 待集成

---

## 6. 标准库 (Standard Library)

### 6.1 已实现模块 ✅

**std/core** - 核心功能
- print, assert
- 类型转换 (int_to_string, parse_int)

**std/string** - 字符串处理
- trim, split, join, substr
- repeat, pad_left, pad_right
- starts_with, ends_with, replace

**std/collections** - 集合类型
- Vec<T> (动态数组)
- HashMap (整数键)

**std/io** - 文件 I/O
- read_file, write_file
- file_exists, is_dir

**std/math** - 数学函数
- abs, max, min, clamp
- pow, sqrt (规划中)
- PI, E 常量

**lib/test** - 测试工具
- assert_eq, assert_true
- test_passed, test_failed

### 6.2 规划中模块 📋

- std/result - Result<T, E> 辅助
- std/option - Option<T> 辅助
- lib/json - JSON 解析
- lib/http - HTTP 客户端

---

## 7. 编译器架构

```
lency_cli      # CLI 入口
lency_driver   # 编译驱动
  ├─ lency_syntax      # 词法+语法 ✅
  ├─ lency_sema        # 语义分析 ✅
  ├─ lency_monomorph   # 泛型单态化 ⚠️ 待重构
  ├─ lency_codegen     # LLVM 代码生成 ✅
  └─ lency_runtime     # 运行时库 ✅

lency_diagnostics # 统一诊断 ⚠️ 待实现
```

**详见**: [assets/roadmap.md](file:///home/indolyn/beryl/assets/roadmap.md)

---

## 8. 文件扩展名

`.lcy`

---

## 9. 下一步开发

**Sprint 14 - 架构重构**:
- 迁移单态化到独立模块
- 实现统一诊断系统

**Sprint 15 - 泛型增强**:
- Result<T, E> 完整支持
- Option<T> 完整支持

**Sprint 16 - 标准库**:
- JSON 解析
- HTTP 基础

详见 [roadmap.md](file:///home/indolyn/beryl/assets/roadmap.md)