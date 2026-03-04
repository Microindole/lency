# Lency 语言设计规范 (Ver 2.1)

> 更新: 2026-03-04
> 状态: 规范持续维护中（实现分为 Rust 主编译器与 Lency 自举编译器两条链路）

## 1. 核心哲学 (Philosophy)

Lency 是一门实用主义的工业级语言，目标是在 C 的结构感与 Python 的开发效率之间取得平衡。

四大支柱:
- Crystal Clear (清晰如晶): 拒绝隐式转换与不透明魔法。
- Safety by Default (默认安全): 空值语义必须显式。
- Structure over Style (结构至上): 使用 C 系 `{}` 结构，减少样板噪音。
- Simplicity First (简洁优先): 显式优于隐式，组合优于继承。

---

## 2. 基础语法 (Syntax)

### 2.1 变量与常量

```lency
var count = 10
var name: string = "Lency"
const PI = 3.14159
```

实现状态:
- Rust 主编译器: 已支持核心变量语法。
- Lency 自举编译器: 已支持 `var` 声明解析；`const` 暂未进入自举主线。

### 2.2 函数 (Functions)

```lency
int add(int a, int b) {
    return a + b
}

void log(string msg) {
    print(msg)
}
```

实现状态:
- Rust 主编译器: 已具备函数语义与代码生成能力。
- Lency 自举编译器: 函数声明解析仍在收尾，当前优先保障语句与表达式主链路可运行。

### 2.3 控制流 (Control Flow)

```lency
if x > 10 {
    print("Large")
} else {
    print("Small")
}

while x > 0 {
    x = x - 1
}

for i = 0; i < 10; i = i + 1 {
    print(i)
}
```

实现状态:
- Rust 主编译器: 已支持主流控制流。
- Lency 自举编译器: `if/while/for/break/continue/return` 已支持，`for` 采用反糖到 `while`。

---

## 3. 类型系统 (Type System)

### 3.1 空安全 (Null Safety)

```lency
string s = "Hello"
string? maybe = null
```

实现状态:
- Rust 主编译器: 空安全与相关检查可用。
- Lency 自举编译器: 当前以最小语义闭环为目标，类型系统仍在增量补齐。

### 3.2 结构体、泛型、枚举、Trait

这是语言规范目标能力，Rust 主编译器链路支持度更高；Lency 自举编译器当前不把该组特性作为近期里程碑优先项。

---

## 4. 错误处理 (Error Handling)

采用 `Result` 风格而非 `try-catch`。

实现状态:
- Rust 主编译器: 诊断系统与错误传播链路已可用。
- Lency 自举编译器: 语义诊断仍以最小文本报错为主，后续会统一收敛到更完整的诊断模型。

---

## 5. 编译器实现分层

### 5.1 Rust 主编译器

`lency_cli` + `lency_driver` + `lency_syntax` + `lency_sema` + `lency_monomorph` + `lency_codegen` + `lency_runtime` + `lency_diagnostics`。

### 5.2 Lency 自举编译器

当前重点目录: `lencyc/`。

当前阶段已打通最小链路:
- Read -> Lex -> Parse -> Resolve -> Emit(AST/LIR)
- 通过 `scripts/run_lency_checks.sh` 做回归闭环

---

## 6. 文件扩展名

`.lcy`

---

## 7. 当前阶段约束 (2026-03-04)

- 规范与实现要分层描述，禁止再用单一“完成百分比”掩盖链路差异。
- Sprint 当前主线是语义增量（Sema），不是继续堆 Parser 文案。
- 文档中涉及阶段计划时，只允许引用 `prompt/sprint/status.md` 的当前状态，不再维护过期冲刺计划副本。
