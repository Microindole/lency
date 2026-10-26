# lency_codegen

Rust 母版的 LLVM codegen，将已完成语义检查和单态化的 AST 降低为 LLVM IR、对象文件或可执行文件所需模块。

## 结构

| 路径 | 职责 |
|---|---|
| `src/context.rs` | codegen 状态与共享上下文 |
| `src/module/` | 模块、类型和函数声明阶段 |
| `src/function.rs` | 函数体生成入口 |
| `src/expr/` | 表达式 lowering |
| `src/stmt/` | 语句和控制流 lowering |
| `src/llvm/` | LLVM 构建辅助层 |
| `src/runtime.rs` | runtime ABI 声明 |
| `src/layout.rs`、`src/types.rs` | 值布局和 LLVM 类型映射 |
| src/nullable.rs | 可空值表示、赋值转换及受保护的解包 |

## 边界

codegen 不应修补前端本应拒绝的类型错误。运行时原语的专用 lowering 必须集中、可测试，并保持语法层为普通调用。

可空引用沿用对象指针，空指针表示没有值；可空标量通过堆上的值单元表示，不能把零或假当成空值，也不能返回局部栈地址。声明、赋值、参数、返回值和字段使用统一的表示转换；空值合并只在非空分支解包。

非空读取细化绑定到具体的变量存储并随作用域保存、恢复；派生数组与集合看到的是细化后的基础类型，左值仍使用原始存储类型。

值单元与结构体沿用现有堆分配方式，自动回收尚未实现；不能据此宣称已有完整的内存管理。
