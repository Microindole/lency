# Beryl 语言实现路线图 v2.0

> **更新日期**: 2026-01-01  
> **设计哲学**: 简洁 · 规范 · 清晰

---

## ✅ 当前已完成 (Current Status)

### 基础架构
- ✅ 完整的编译器管道 (Lexer → Parser → Sema → Codegen)
- ✅ LLVM 后端代码生成
- ✅ 模块化架构 (遵循开闭原则)

### 类型系统
- ✅ **Phase 0.1**: 类型系统抽象化 (`types/` 模块)
- ✅ **Phase 0.2**: 运算符表驱动 (`operators/` 模块)
- ✅ **Phase 1.1**: String 类型 + 字符串连接
- ✅ **Phase 1.2**: Bool 类型 + 逻辑运算
- ✅ **Phase 1.3**: Float 类型 + 类型提升 (int+float)

**类型提升规则**: `int + float` → `float` (使用 LLVM sitofp)

### 控制流
- ✅ `if/else` 语句
- ✅ `while` 循环 (已实现)
- ✅ `return` 语句

### 其他
- ✅ 函数定义与调用
- ✅ 变量声明 (`var`)
- ✅ 完整的作用域管理
- ✅ 名称解析 (两遍扫描)
- ✅ 类型检查

---

## 🎯 下一阶段目标 (Q1 2026)

### **阶段 2: 控制流扩展** (1-2 周)

#### 2.1 For 循环 ⭐ 优先级：高
**状态**: 未实现

**语法设计**:
```beryl
// C-style for loop
for var i = 0; i < 10; i = i + 1 {
    print(i)
}
```

**实现要点**:
```rust
// AST
Stmt::For {
    init: Option<Box<Stmt>>,        // var i = 0
    condition: Option<Box<Expr>>,   // i < 10
    update: Option<Box<Expr>>,    // i = i + 1
    body: Vec<Stmt>,
}
```

**LLVM IR 结构**:
```llvm
; 初始化
entry:
  %i = alloca i64
  store i64 0, ptr %i

; 条件检查
for.cond:
  %i.val = load i64, ptr %i
  %cond = icmp slt i64 %i.val, 10
  br i1 %cond, label %for.body, label %for.end

; 循环体
for.body:
  ; ... body ...
  br label %for.inc

; 更新
for.inc:
  %i.val2 = load i64, ptr %i
  %i.next = add i64 %i.val2, 1
  store i64 %i.next, ptr %i
  br label %for.cond

; 结束
for.end:
  ; ...
```

**文件修改**:
- `syntax/ast/stmt.rs`: 添加 `Stmt::For`
- `syntax/parser/stmt.rs`: 解析 for 循环
- `sema/type_check.rs`: 类型检查
- `codegen/stmt.rs`: IR 生成

---

#### 2.2 Break/Continue 语句 ⭐ 优先级：高
**状态**: 未实现

**实现要点**:
```rust
// AST
enum Stmt {
    Break,
    Continue,
    // ...
}
```

**LLVM IR**:
```llvm
; break -> br label %loop.end
; continue -> br label %loop.cond (或 for.inc)
```

**挑战**: 需要在codegen中维护循环标签栈

---

#### 2.3 Match 表达式 (简化版) ⭐ 优先级：中
**状态**: 未实现

**语法**:
```beryl
int classify(int code) {
    match code {
        200 => return 1
        404 => return 0
        _   => return -1
    }
}
```

**实现策略**:
- Phase 1: 仅支持整数常量匹配
- 使用 LLVM `switch` 指令
- `_` 映射到 `default`

---

### **阶段 3: 数组与集合** (1.5 周)

#### 3.1 数组类型 ⭐ 优先级：高
**语法设计**:
```beryl
// 固定大小数组
var arr: [int; 5] = [1, 2, 3, 4, 5]

// 访问
var x = arr[0]
arr[1] = 10
```

**实现**:
- LLVM 数组类型: `[5 x i64]`
- 编译期大小检查

---

#### 3.2 动态数组 (Vec) ⭐ 优先级：中
**语法**:
```beryl
var v = vec![1, 2, 3]
v.push(4)
var len = v.length()
```

**需要**:
- 运行时内存管理 (引入 `beryl_runtime`)
- 堆分配 (malloc/realloc)

---

### **阶段 4: 结构体** (2 周)

#### 4.1 基础结构体
```beryl
struct Point {
    int x
    int y
}

Point make_point(int x, int y) {
    var p = Point { x: x, y: y }
    return p
}
```

**实现**:
- LLVM struct 类型
- 字段访问: `getelementptr`

---

#### 4.2 方法 (Method)
```beryl
struct Point {
    int x
    int y
    
    int distance(Point other) {
        var dx = self.x - other.x
        var dy = self.y - other.y
        return dx * dx + dy * dy
    }
}
```

---

### **阶段 5: 空安全系统** (3 周) 🌟 核心特性

#### 5.1 可空类型 `T?`
```beryl
string? find_user(int id) {
    if id == 1 {
        return "Alice"
    }
    return null
}
```

**实现**:
- AST: `Type::Nullable(Box<Type>)`
- LLVM: `{i1, T}` (bool + value)
- 或 Option-like tagged union

---

#### 5.2 智能类型转换
```beryl
var user = find_user(1)  // 类型: string?

if user != null {
    print(user)  // 这里 user 是 string (非空)
}
```

**实现**: 控制流分析 (Flow Analysis)

---

### **阶段 6: 模块系统** (1.5 周)

#### 6.1 基础导入
```beryl
// math/calc.brl
pub int add(int a, int b) {
    return a + b
}

// main.brl
import math.calc

int main() {
    return calc.add(1, 2)
}
```

**设计**:
- 文件 = 模块
- 目录 = 包
- `pub` 控制可见性

---

### **阶段 7: 泛型** (3 周)

#### 7.1 泛型结构体
```beryl
struct Box<T> {
    T value
}
```

**策略**: 单态化 (Monomorphization)
- 编译期生成特化版本
- 零运行时开销

---

## 📅 时间表 (Timeline)

| 阶段 | 工作量 | 开始日期 | 预计完成 |
|------|--------|----------|----------|
| **Phase 2**: 控制流 | 1-2周 | 2026-01-02 | 2026-01-16 |
| **Phase 3**: 数组 | 1.5周 | 2026-01-17 | 2026-01-27 |
| **Phase 4**: 结构体 | 2周 | 2026-01-28 | 2026-02-11 |
| **Phase 5**: 空安全 | 3周 | 2026-02-12 | 2026-03-05 |
| **Phase 6**: 模块 | 1.5周 | 2026-03-06 | 2026-03-17 |
| **Phase 7**: 泛型 | 3周 | 2026-03-18 | 2026-04-08 |

**总计**: 约 12-15 周 (3-4 个月)

---

## 🎯 MVP 定义 (v0.5)

**包含特性**:
- ✅ 4种基础类型 (int, float, bool, string)
- ✅ 类型提升
- ✅ 控制流 (if, while, for, break, continue)
- ⬜ 数组
- ⬜ 结构体
- ⬜ 基础模块系统

**MVP 时间**: 约 5-6 周 (Phase 2-4)

---

## 🌟 v1.0 定义

**核心卖点**: 空安全系统

**包含特性**:
- ✅ MVP 所有功能
- ⬜ **可空类型 `T?`**
- ⬜ **智能类型转换**
- ⬜ 泛型 (基础版)
- ⬜ 完整模块系统

**v1.0 时间**: 约 12-15 周

---

## 🚀 立即可开始的任务 (Next Sprint)

### Sprint 1: For 循环实现 (3-5 天)
1. AST 定义 (`Stmt::For`)
2. Parser 实现
3. 类型检查
4. Codegen (LLVM IR)
5. 测试用例

**验收标准**:
```beryl
int sum(int n) {
    var total = 0
    for var i = 0; i < n; i = i + 1 {
        total = total + i
    }
    return total
}
```

### Sprint 2: Break/Continue (2-3 天)
1. AST 定义
2. Parser 实现
3. Codegen (管理循环标签栈)
4. 测试用例

---

## 📊 代码质量目标

- ✅ 无文件超过 500 行 (已达成)
- ✅ 模块化架构 (已达成)
- ✅ 所有测试通过 (43/43)
- ⬜ 测试覆盖率 > 70%
- ⬜ 完整的错误消息

---

## 🔧 技术债务 (Technical Debt)

### 需要解决
1. **内存管理**: 字符串连接有内存泄漏 (malloc 未 free)
   - **解决方案**: Phase 5 引入 GC 或 RAII
   
2. **错误报告**: 错误消息需要更友好
   - **解决方案**: 使用 Ariadne 库

3. **优化**: 目前无优化pass
   - **解决方案**: Phase 后期引入 LLVM 优化 pass

### 可接受延后
- 字符串插值 (`"Hello {name}"`)
- 类型别名 (`type UserId = int`)
- 枚举类型

---

## 📚 参考资料

- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)
- [Rust Compiler Internals](https://rustc-dev-guide.rust-lang.org/)
- [Kotlin Null Safety](https://kotlinlang.org/docs/null-safety.html)

---

**文档结束** | 下一步：开始 Phase 2.1 For 循环实现
